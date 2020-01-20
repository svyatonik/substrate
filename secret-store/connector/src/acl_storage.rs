use std::sync::Arc;
use log::warn;
use parking_lot::RwLock;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT, NumberFor, ProvideRuntimeApi},
};
use ss_primitives::acl_storage::AclStorageRuntimeApi;

/// On-chain ACL storage implementation.
pub struct OnChainAclStorage<Block: BlockT, Api> {
	/// Something that provides key server set runtime API.
	runtime: Arc<Api>,
	/// Mutable data reference.
	data: RwLock<OnChainAclStorageMutableData<NumberFor<Block>, Block::Hash>>,
}

/// Mutable data of ACL storage connector.
struct OnChainAclStorageMutableData<N, H> {
	/// Last finalized block.
	finalized: (N, H),
}

impl<Block: BlockT, Api> OnChainAclStorage<Block, Api> where
	Block: BlockT,
	Api: ProvideRuntimeApi,
	Api::Api: ss_primitives::acl_storage::AclStorageRuntimeApi<Block>,
{
	/// Create new ACL storage that is backed by the provided runtime.
	pub fn new(runtime: Arc<Api>,) -> Self {
		OnChainAclStorage {
			runtime,
			data: RwLock::new(OnChainAclStorageMutableData {
				finalized: (0.into(), Default::default()), // TODO: actual best + finalized
			})
		}
	}

	/// Called when new block is finalized.
	pub fn on_block_finalized(&mut self, block: (NumberFor<Block>, Block::Hash)) {
		self.data.write().finalized = block;
	}
}

impl<Block, Api> ss_primitives::secret_store::AclStorage for OnChainAclStorage<Block, Api> where
	Block: BlockT,
	Api: ProvideRuntimeApi,
	Api::Api: ss_primitives::acl_storage::AclStorageRuntimeApi<Block>,
{
	fn check(
		&self,
		requester: ss_primitives::secret_store::RequesterId,
		document: ss_primitives::secret_store::ServerKeyId,
	) -> Result<bool, String> {
		let block = self.data.read().finalized.clone();
		match self.runtime.runtime_api().check(&BlockId::Hash(block.1), requester.into(), document.into()) {
			Ok(check_result) => Ok(check_result),
			Err(error) => {
				let error = format!("{:?}", error);
				warn!(target: "secretstore_net", "Error checking key access rights: {}", error);
				Err(error)
			}
		}
	}
}
