use codec::{Encode, Decode};
use sp_std::vec::Vec;
use crate::{
	KeyServerId, ServerKeyId, EntityId,
};

/// Service contract task.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ServiceTask {
	/// Generate server key (server_key_id, author, threshold).
	GenerateServerKey(ServerKeyId, EntityId, u8),
	/// Retrieve server key (server_key_id).
	RetrieveServerKey(ServerKeyId),
/*	/// Store document key (server_key_id, author, common_point, encrypted_point).
	StoreDocumentKey(ServerKeyId, RequesterId, CommonPoint, EncryptedPoint),
	/// Retrieve common data of document key (server_key_id, requester).
	RetrieveShadowDocumentKeyCommon(ServerKeyId, RequesterId),
	/// Retrieve personal data of document key (server_key_id, requester).
	RetrieveShadowDocumentKeyPersonal(ServerKeyId, RequesterId),*/
}

sp_api::decl_runtime_apis! {
	/// Service runtime API.
	pub trait SecretStoreServiceApi {
		///
		fn server_key_generation_tasks(begin: u32, end: u32) -> Vec<ServiceTask>;
		/// Check if server key generation response is required from given key server.
		fn is_server_key_generation_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool;

		///
		fn server_key_retrieval_tasks(begin: u32, end: u32) -> Vec<ServiceTask>;
		/// Check if server key retrieval response is required from given key server.
		fn is_server_key_retrieval_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool;
	}
}
/*

		/// Return count of pending service tasks.
		fn pending_tasks_count() -> u32;
		/// Return pending task by index.
		fn pending_task(index: u32) -> Option<ServiceTask>;


		/// Check if server key retrieval response is required from given key server.
		fn is_server_key_retrieval_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool;
		/// Check if document key store response is required from given key server.
		fn is_document_key_store_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool;
*/