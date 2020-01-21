use codec::{Encode, Decode};
use sp_std::vec::Vec;
use crate::{
	KeyServerId, ServerKeyId, RequesterId, CommonPoint,
	EncryptedPoint, DecryptedSecret, DocumentKeyShadow, ServerKeyPublic,
};

/// Service contract task.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ServiceTask {
	/// Generate server key (server_key_id, author, threshold).
	GenerateServerKey(ServerKeyId, RequesterId, u8),
	/// Retrieve server key (server_key_id).
	RetrieveServerKey(ServerKeyId),
	/// Store document key (server_key_id, author, common_point, encrypted_point).
	StoreDocumentKey(ServerKeyId, RequesterId, CommonPoint, EncryptedPoint),
	/// Retrieve common data of document key (server_key_id, requester).
	RetrieveShadowDocumentKeyCommon(ServerKeyId, RequesterId),
	/// Retrieve personal data of document key (server_key_id, requester).
	RetrieveShadowDocumentKeyPersonal(ServerKeyId, RequesterId),
}

/// Service contract response.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum ServiceResponse {
	/// Server key is generated.
	ServerKeyGenerated(ServerKeyId, ServerKeyPublic),
	/// Server key generation has failed.
	ServerKeyGenerationFailed(ServerKeyId),
	/// Server key retrieved.
	ServerKeyRetrieved(ServerKeyId, ServerKeyPublic, u8),
	/// Server key retrieval has failed.
	ServerKeyRetrievalFailed(ServerKeyId),
	/// Document key stored.
	DocumentKeyStored(ServerKeyId),
	/// Document key store has failed.
	DocumentKeyStoreFailed(ServerKeyId),
	/// Document key common part retrieved.
	DocumentKeyCommonRetrieved(ServerKeyId, RequesterId, CommonPoint, u8),
	/// Document key personal part retrieved.
	DocumentKeyPersonalRetrieved(ServerKeyId, RequesterId, Vec<KeyServerId>, DecryptedSecret, DocumentKeyShadow),
	/// Document key retrieval has failed.
	DocumentKeyRetrievalFailed(ServerKeyId, RequesterId),
}

sp_api::decl_runtime_apis! {
	/// Service runtime API.
	pub trait ServiceRuntimeApi {
		/// Return count of pending service tasks.
		fn pending_tasks_count() -> u32;
		/// Return pending task by index.
		fn pending_task(index: u32) -> Option<ServiceTask>;

		/// Check if server key generation response is required from given key server.
		fn is_server_key_generation_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool;
		/// Check if server key retrieval response is required from given key server.
		fn is_server_key_retrieval_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool;
		/// Check if document key store response is required from given key server.
		fn is_document_key_store_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool;
	}
}
