use codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;
use crate::KeyServerId;

/// Migration id.
pub type MigrationId = H256;

/// Opaque key server network address type.
pub type KeyServerNetworkAddress = Vec<u8>;

/// Key server set snapshot.
#[derive(Decode, Encode, PartialEq, RuntimeDebug)]
pub struct KeyServerSetSnapshot {
	/// Current set of key servers.
	pub current_set: Vec<(KeyServerId, KeyServerNetworkAddress)>,
	/// New set of key servers.
	pub new_set: Vec<(KeyServerId, KeyServerNetworkAddress)>,
	/// Current migration data.
	pub migration: Option<KeyServerSetMigration>,
}

/// Key server set migration.
#[derive(Decode, Encode, PartialEq, RuntimeDebug)]
pub struct KeyServerSetMigration {
	/// Migration id.
	pub id: MigrationId,
	/// Migration set of key servers. It is the new_set at the moment of migration start.
	pub set: Vec<(KeyServerId, KeyServerNetworkAddress)>,
	/// Master node of the migration process.
	pub master: KeyServerId,
	/// Is migration confirmed by this node?
	pub is_confirmed: bool,
}

sp_api::decl_runtime_apis! {
	/// Runtime API that backs the key server set.
	pub trait SecretStoreKeyServerSetApi {
		/// Get server set state.
		fn snapshot(key_server: KeyServerId) -> KeyServerSetSnapshot;
	}
}
