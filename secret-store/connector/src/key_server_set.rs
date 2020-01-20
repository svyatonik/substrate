use std::sync::Arc;
use log::warn;
use parking_lot::RwLock;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, NumberFor, ProvideRuntimeApi, SimpleArithmetic},
};
use ss_primitives::key_server_set::KeyServerSetWithMigrationRuntimeApi;

/// Number of blocks before the same-migration transaction (be it start or confirmation) will be retried.
const TRANSACTION_RETRY_INTERVAL_BLOCKS: u32 = 30;

/// Transaction pool that can accept key server set transactions.
pub trait KeyServerSetTransactionPool<Block: BlockT> {
	/// Submit start migration transaction.
	fn start_migration(&self, at: &BlockId<Block>, migration_id: ss_primitives::key_server_set::MigrationId);
	/// Submit confitm migration transaction.
	fn confirm_migration(&self, at: &BlockId<Block>, migration_id: ss_primitives::key_server_set::MigrationId);
}

/// Key server set with migration support.
pub struct OnChainKeyServerSetWithMigration<Block: BlockT, Api, Pool> {
	/// Something that provides key server set runtime API.
	runtime: Arc<Api>,
	/// Transaction pool reference.
	pool: Arc<Pool>,
	/// ID of this key server.
	self_id: ss_primitives::KeyServerId,
	/// Mutable data reference.
	data: RwLock<OnChainKeyServerSetWithMigrationData<NumberFor<Block>, Block::Hash>>,
}

/// Mutable data of key server set connector.
struct OnChainKeyServerSetWithMigrationData<N, H> {
	/// Best block.
	best: (N, H),
	/// Last finalized block.
	finalized: (N, H),
	/// Key server set snapshot at last finalized block.
	snapshot: ss_primitives::secret_store::KeyServerSetSnapshot,
	/// Previous start migration transaction (if has been sent).
	start_migration_tx: Option<PreviousMigrationTransaction<N, H>>,
	/// Previous confirm migration transaction (if has been sent).
	confirm_migration_tx: Option<PreviousMigrationTransaction<N, H>>,
}

/// Previous migration-related transaction.
struct PreviousMigrationTransaction<N, H> {
	/// ID of migration process.
	migration_id: ss_primitives::secret_store::MigrationId,
	/// Best block when transaction has been sent.
	block: (N, H),
}

impl<Block: BlockT, Api, Pool> OnChainKeyServerSetWithMigration<Block, Api, Pool> where
	Block: BlockT,
	Api: ProvideRuntimeApi,
	Api::Api: ss_primitives::key_server_set::KeyServerSetWithMigrationRuntimeApi<Block>,
	Pool: KeyServerSetTransactionPool<Block>,
{
	/// Create new key server set that is backed by the provided runtime.
	pub fn new(
		runtime: Arc<Api>,
		pool: Arc<Pool>,
		self_id: ss_primitives::KeyServerId,
	) -> Self {
		OnChainKeyServerSetWithMigration {
			runtime,
			pool,
			self_id,
			data: RwLock::new(OnChainKeyServerSetWithMigrationData {
				best: (0.into(), Default::default()),
				finalized: (0.into(), Default::default()), // TODO: actual best + finalized
				snapshot: Default::default(),
				start_migration_tx: None,
				confirm_migration_tx: None,
			}),
		}
	}

	/// Called when new best block is inserted.
	pub fn on_best_block_updated(&mut self, block: (NumberFor<Block>, Block::Hash)) {
		self.data.write().best = block;
	}

	/// Called when new block is finalized.
	pub fn on_block_finalized(&mut self, block: (NumberFor<Block>, Block::Hash)) {
		// TODO: probably read on demand?
		let snapshot = match self.runtime.runtime_api().snapshot(&BlockId::Hash(block.1), self.self_id.clone()) {
			Ok(snapshot) => snapshot.into(),
			Err(error) => {
				warn!(target: "secretstore_net", "Error reading key server set snapshot from contract: {:?}", error);
				return;
			}
		};

		let mut data = self.data.write();
		data.snapshot = snapshot;
		data.finalized = block;
	}
}

impl<Block, Api, Pool> ss_primitives::secret_store::KeyServerSet
for
	OnChainKeyServerSetWithMigration<Block, Api, Pool>
where
	Block: BlockT,
	Api: ProvideRuntimeApi,
	Api::Api: ss_primitives::key_server_set::KeyServerSetWithMigrationRuntimeApi<Block>,
	Pool: KeyServerSetTransactionPool<Block>,
{
	fn is_isolated(&self) -> bool {
		!self.data.read().snapshot.current_set.contains_key(&self.self_id.into())
	}

	fn snapshot(&self) -> ss_primitives::secret_store::KeyServerSetSnapshot {
		self.data.read().snapshot.clone()
	}

	fn start_migration(&self, migration_id: ss_primitives::secret_store::MigrationId) {
		let mut data = self.data.write();

		// check if we need to send start migration transaction
		let best_block = data.best.clone();
		if !update_last_transaction_block(&best_block, &migration_id, &mut data.start_migration_tx) {
			return;
		}

		// send transaction
		self.pool.start_migration(&BlockId::Hash(best_block.1), migration_id);
	}

	fn confirm_migration(&self, migration_id: ss_primitives::secret_store::MigrationId) {
		let mut data = self.data.write();

		// check if we need to send confirm migration transaction
		let best_block = data.best.clone();
		if !update_last_transaction_block(&best_block, &migration_id, &mut data.confirm_migration_tx) {
			return;
		}

		// send transaction
		self.pool.confirm_migration(&BlockId::Hash(best_block.1), migration_id);
	}
}

fn update_last_transaction_block<N: Clone + SimpleArithmetic, H: Clone>(
	best: &(N, H),
	migration_id: &ss_primitives::key_server_set::MigrationId,
	previous_transaction: &mut Option<PreviousMigrationTransaction<N, H>>,
) -> bool {
	match previous_transaction.as_ref() {
		// no previous transaction => send immediately
		None => (),
		// previous transaction has been sent for other migration process => send immediately
		Some(tx) if tx.migration_id != *migration_id => (),
		// if we have sent the same type of transaction recently => do nothing (hope it will be mined eventually)
		// if we have sent the same transaction some time ago =>
		//   assume that our tx queue was full
		//   or we didn't have enough eth fot this tx
		//   or the transaction has been removed from the queue (and never reached any miner node)
		// if we have restarted after sending tx => assume we have never sent it
		Some(tx) => {
			if tx.block.0 > best.0 || best.0.clone() - tx.block.0.clone() < TRANSACTION_RETRY_INTERVAL_BLOCKS.into() {
				return false;
			}
		},
	}

	*previous_transaction = Some(PreviousMigrationTransaction {
		migration_id: migration_id.clone(),
		block: (best.0.clone(), best.1.clone()),
	});

	true
}
