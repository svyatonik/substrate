// TODO: get pair from KeyStorePtr???

use std::sync::Arc;
use futures01::{prelude::*, sync::mpsc};
use futures::{
	compat::Compat,
	FutureExt as _, TryFutureExt as _,
	StreamExt as _, TryStreamExt as _,
	future::{select, Either}
};
use codec::Encode;
use node_primitives::{AccountId, Block, BlockId, Index};
use node_runtime::{VERSION, Call, SecretStoreCall, UncheckedExtrinsic};
use sp_core::{H256, crypto::Pair};
use sp_runtime::{OpaqueExtrinsic, traits::{Block as BlockT, IdentifyAccount, ProvideRuntimeApi}};
use sp_transaction_pool::TransactionPool;
use frame_system_rpc_runtime_api::AccountNonceApi;
use log::{trace, warn};

struct SecretStoreTransactionPool<Pool, Api> {
	pool: Arc<Pool>,
	runtime: Arc<Api>,
	executor: sc_service::TaskExecutor,
	account: AccountId,
	account_pair: sp_core::sr25519::Pair,
	genesis_hash: H256,
}

impl<Pool, Api> SecretStoreTransactionPool<Pool, Api>
	where
		Pool: TransactionPool<Block = Block>,
		Pool::Error: 'static,
		Api: ProvideRuntimeApi,
		Api::Api: frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>,
{
	/// Submit transaction.
	fn submit_transaction(&self, at: &BlockId, call: Call) -> Result<(), String> {
		let raw_payload = node_runtime::SignedPayload::from_raw(
			call,
			(
				frame_system::CheckVersion::<node_runtime::Runtime>::new(),
				frame_system::CheckGenesis::<node_runtime::Runtime>::new(),
				frame_system::CheckEra::<node_runtime::Runtime>::from(sp_runtime::generic::Era::Immortal),
				frame_system::CheckNonce::<node_runtime::Runtime>::from(
					self.runtime.runtime_api().account_nonce(at, self.account.clone())
						.map_err(|err| format!("{:?}", err))?,
				),
				frame_system::CheckWeight::<node_runtime::Runtime>::new(),
				pallet_transaction_payment::ChargeTransactionPayment::<node_runtime::Runtime>::from(0),
				Default::default(),
			),
			(
				VERSION.spec_version as u32,
				self.genesis_hash,
				self.genesis_hash,
				(),
				(),
				(),
				(),
			),
		);

		let signature = raw_payload.using_encoded(|payload| self.account_pair.sign(payload));
		let signer: sp_runtime::MultiSigner = self.account_pair.public().into();
		let (function, extra, _) = raw_payload.deconstruct();

		let transaction = node_runtime::UncheckedExtrinsic::new_signed(
			function,
			signer.into_account().into(),
			signature.into(),
			extra,
		);

		let submit_future = self.pool
			.submit_one(at, OpaqueExtrinsic(transaction.encode()))
			.map(move |_| Ok::<(), ()>(()));

		self.executor.execute(Box::new(submit_future.compat()));

		Ok(())
	}
}

impl<Pool, Api> substrate_secret_store_connector::key_server_set::KeyServerSetTransactionPool<Block> for SecretStoreTransactionPool<Pool, Api>
	where
		Pool: TransactionPool<Block = Block>,
		Pool::Error: 'static,
		Api: ProvideRuntimeApi,
		Api::Api: frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Index>,
{
	fn start_migration(
		&self,
		at: &BlockId,
		migration_id: ss_primitives::key_server_set::MigrationId,
	) {
		let call = Call::SecretStore(SecretStoreCall::start_migration(migration_id));
		if let Err(error) = self.submit_transaction(at, call) {
			warn!(target: "secretstore_net", "Error submitting start migration transaction: {:?}", error);
		}
	}

	fn confirm_migration(
		&self,
		at: &BlockId,
		migration_id: ss_primitives::key_server_set::MigrationId,
	) {
		let call = Call::SecretStore(SecretStoreCall::confirm_migration(migration_id));
		if let Err(error) = self.submit_transaction(at, call) {
			warn!(target: "secretstore_net", "Error submitting start migration transaction: {:?}", error);
		}
	}
}
