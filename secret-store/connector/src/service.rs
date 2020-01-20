use std::sync::Arc;
use log::warn;
use parking_lot::RwLock;
use sp_core::twox_128;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, NumberFor, ProvideRuntimeApi, One},
};
use ss_primitives::key_server_set::KeyServerSetWithMigrationRuntimeApi;

/// Transaction pool that can accept service transactions.
pub trait KeyServerSetTransactionPool<Block: BlockT> {
	/// Submit start migration transaction.
	fn start_migration(&self, at: &BlockId<Block>, migration_id: ss_primitives::key_server_set::MigrationId);
	/// Submit confitm migration transaction.
	fn confirm_migration(&self, at: &BlockId<Block>, migration_id: ss_primitives::key_server_set::MigrationId);
}

/// On chain service.
pub struct OnChainService<Block: BlockT, Client, Api, Pool> {
	/// Something that provides access to the state storage.
	state_storage: Arc<Client>,
	/// Something that provides key server set runtime API.
	runtime: Arc<Api>,
	/// Transaction pool reference.
	pool: Arc<Pool>,
	/// Mutable data reference.
	data: RwLock<OnChainServiceData<NumberFor<Block>, Block::Hash>>,
}

/// Mutable data of key server set connector.
struct OnChainServiceData<N, H> {
	/// Best block.
	best: (N, H),
	/// Last finalized block.
	finalized: (N, H),
	/// Number of last finalized block when the `new_requests()` has been called last time.
	prev_finalized: (N, H),
}

impl<Block, Client, Api, Pool> ss_primitives::secret_store::Service
for
	OnChainService<Block, Client, Api, Pool>
where
	Block: BlockT,
	Api: ProvideRuntimeApi,
	Api::Api: ss_primitives::service::ServiceRuntimeApi<Block>,
	Pool: KeyServerSetTransactionPool<Block>,
{
	fn new_requests(&self) -> Box<dyn Iterator<Item = ss_primitives::secret_store::ServiceTask>> {
		unimplemented!()
	}

	fn pending_requests(&self) -> Box<dyn Iterator<Item = ss_primitives::secret_store::ServiceTask>> {
		unimplemented!()
	}

	fn publish_response(&self, response: ss_primitives::secret_store::ServiceResponse) {
		unimplemented!()
	}
}

struct NewRequests<Block: BlockT> {
	current_block: NumberFor<Block>,
	last_finalized_block: NumberFor<Block>,
//	current_block_events: Option<Vec<Event>>,
}

impl<Block: BlockT> Iterator for NewRequests<Block> {
	type Item = ss_primitives::secret_store::ServiceTask;

	fn next(&mut self) -> Option<Self::Item> {
/*		loop {
			if let Some(next_task) = self.next_task_iter.next() {
				return Some(next_task);
			}

			if self.current_block > self.last_finalized_block {
				return None;
			}

			self.next_task_iter = NewRequestsFromBlock::new(self.current_block);
			self.current_block = self.current_block + One::one();
		}*/
		unimplemented!()
	}
}
/*
struct NewRequestsFromBlock<Block: BlockT> {
	current_task: u32,
	last_task: u32,
}

impl<Block: BlockT> NewRequestsFromBlock<Block> {
	pub fn new<Client>(block: BlockId<Block>, state_storage: Arc<Client>) -> Self {
		let state = state_storage.state_at(block);
		let events = match state {
			Ok(state) => state.storage(system_events_storage_key()),
			Err(error) => {},
		};
	}
}

impl<Block: BlockT> Iterator for NewRequestsFromBlock {
	type Item = ss_primitives::secret_store::ServiceTask;

	fn next(&mut self) -> Option<Self::Item> {
		unimplemented!()
	}
}

fn system_events_storage_key() -> Vec<u8> {
	// https://github.com/scs/substrate-api-client/blob/4bb9f7dd792be6bf69fac5386b1fe054550689ba/src/utils.rs#L23
	twox_128("System Events".as_bytes())
}
*/