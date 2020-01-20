use std::collections::HashMap;
use sp_core::H256;

/// Requester id.
pub type RequesterId = H256;

/// Server key id.
pub type ServerKeyId = H256;

/// Server key.
pub type ServerKey = Vec<u8>;

/// Key server node id.
pub type KeyServerId = H256;

/// Key server node address.
pub type KeyServerAddress = Vec<u8>;

/// Migration id.
pub type MigrationId = H256;

/// Common point of document key.
pub type CommonPoint = Vec<u8>;

/// Encrypted point of document key.
pub type EncryptedPoint = Vec<u8>;

///
pub type DecrypedSecret = Vec<u8>;

///
pub type DocumentKeyShadow = Vec<u8>;

/// Key server set snapshot.
#[derive(Clone, Debug, Default)]
pub struct KeyServerSetSnapshot {
	/// Current set of key servers.
	pub current_set: HashMap<KeyServerId, KeyServerAddress>,
	/// New set of key servers.
	pub new_set: HashMap<KeyServerId, KeyServerAddress>,
	/// Current migration data.
	pub migration: Option<KeyServerSetMigration>,
}

/// Key server set migration.
#[derive(Clone, Debug, Default)]
pub struct KeyServerSetMigration {
	/// Migration id.
	pub id: MigrationId,
	/// Migration set of key servers. It is the new_set at the moment of migration start.
	pub set: HashMap<KeyServerId, KeyServerAddress>,
	/// Master node of the migration process.
	pub master: KeyServerId,
	/// Is migration confirmed by this node?
	pub is_confirmed: bool,
}

/// Service contract task.
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceTask {
	/// Generate server key (server_key_id, author, threshold).
	GenerateServerKey(ServerKeyId, RequesterId, usize),
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
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceResponse {
	/// Server key is generated.
	ServerKeyGenerated(ServerKeyId, ServerKey),
	/// Server key generation has failed.
	ServerKeyGenerationFailed(ServerKeyId),
	/// Server key retrieved.
	ServerKeyRetrieved(ServerKeyId, ServerKey, usize),
	/// Server key retrieval has failed.
	ServerKeyRetrievalFailed(ServerKeyId),
	/// Document key stored.
	DocumentKeyStored(ServerKeyId),
	/// Document key store has failed.
	DocumentKeyStoreFailed(ServerKeyId),
	/// Document key common part retrieved.
	DocumentKeyCommonRetrieved(ServerKeyId, RequesterId, CommonPoint, usize),
	/// Document key personal part retrieved.
	DocumentKeyPersonalRetrieved(ServerKeyId, RequesterId, Vec<KeyServerId>, DecrypedSecret, DocumentKeyShadow),
	/// Document key retrieval has failed.
	DocumentKeyRetrievalFailed(ServerKeyId, RequesterId),
}

/// Key server set API.
pub trait KeyServerSet {
	/// Is this node currently isolated from the set?
	fn is_isolated(&self) -> bool;
	/// Get server set state.
	fn snapshot(&self) -> KeyServerSetSnapshot;
	/// Start migration.
	fn start_migration(&self, migration_id: MigrationId);
	/// Confirm migration.
	fn confirm_migration(&self, migration_id: MigrationId);
}

/// ACL storage of Secret Store.
pub trait AclStorage {
	/// Check if requestor can access document with hash `document`
	fn check(&self, requester: RequesterId, document: ServerKeyId) -> Result<bool, String>;
}

/// Service API.
pub trait Service {
	/// Return iterator of requests that have appeard since last call.
	fn new_requests(&self) -> Box<dyn Iterator<Item = ServiceTask>>;
	/// Return iterator of pending requests.
	fn pending_requests(&self) -> Box<dyn Iterator<Item = ServiceTask>>;
	/// Publish secret key server response.
	fn publish_response(&self, response: ServiceResponse);
}
