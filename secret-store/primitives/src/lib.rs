#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::vec::Vec;

pub mod acl_storage;
pub mod key_server_set;
pub mod service;

/// Any entrity is identified by this id.
pub type EntityId = sp_core::H512;

/// Requester id.
pub type RequesterId = sp_core::H160;
///
pub type Address = sp_core::H160;
/// Server key id.
pub type KeyServerId = sp_core::H512;

/// Network address type.
pub type NetworkAddress = Vec<u8>;

///
pub type ServerKeyId = sp_core::H256;

///
pub type ServerKeyPublic = sp_core::H512;

///
pub type CommonPoint = sp_core::H512;

///
pub type EncryptedPoint = sp_core::H512;

pub type DecryptedSecret = sp_core::H512;

pub type DocumentKeyShadow = Vec<u8>;

pub fn into_author_address(entity: EntityId) -> Address {
	unimplemented!()
}

/*/// Key Server Set state.
#[derive(Debug, PartialEq)]
pub struct KeyServerSetSnapshot {
	/// Current set of key servers.
	pub current_set: Vec<(KeyServerId, NetworkAddress)>,
	/// New set of key servers.
	pub new_set: Vec<(KeyServerId, NetworkAddress)>,
	/// Current migration data.
	pub migration: Option<KeyServerSetMigration>,
}

/// Key server set migration.
#[derive(Debug, PartialEq)]
pub struct KeyServerSetMigration {
	/// Migration id.
	pub id: MigrationId,
	/// Migration set of key servers. It is the new_set at the moment of migration start.
	pub set: Vec<(KeyServerId, NetworkAddress)>,
	/// Master node of the migration process.
	pub master: KeyServerId,
	/// Is migration confirmed by this node?
	pub is_confirmed: bool,
}*/

/*pub mod acl_storage;
pub mod key_server_set;

/// TODO: these types MUST BE taken from secret-store package
pub mod externals {
	use std::net::SocketAddr;

	/// H256 type.
	pub type H256 = [u8; 32];

	/// Secp256k1 public key type.
	pub type Public = [u8; 64];

	/// Secp256k1 secret key type.
	pub type Secret = [u8; 32];

	/// Ethereum address type.
	pub type Address = [u8; 20];

	/// Server key id.
	pub type ServerKeyId = [u8; 32];

	/// Key Server id.
	pub type NodeId = Public;

	/// Secret store error type.
	pub enum Error {}

	/// Key Server Set state.
	#[derive(Default, Debug, Clone, PartialEq)]
	pub struct KeyServerSetSnapshot {
		/// Current set of key servers.
		pub current_set: BTreeMap<NodeId, SocketAddr>,
		/// New set of key servers.
		pub new_set: BTreeMap<NodeId, SocketAddr>,
		/// Current migration data.
		pub migration: Option<KeyServerSetMigration>,
	}

	/// Key server set migration.
	#[derive(Default, Debug, Clone, PartialEq)]
	pub struct KeyServerSetMigration {
		/// Migration id.
		pub id: H256,
		/// Migration set of key servers. It is the new_set at the moment of migration start.
		pub set: BTreeMap<NodeId, SocketAddr>,
		/// Master node of the migration process.
		pub master: NodeId,
		/// Is migration confirmed by this node?
		pub is_confirmed: bool,
	}

	/// ACL storage of Secret Store
	pub trait AclStorage: Send + Sync {
		/// Check if requestor can access document with hash `document`
		fn check(&self, requester: Address, document: &ServerKeyId) -> Result<bool, Error>;
	}

	/// Key Server Set
	pub trait KeyServerSet: Send + Sync {
		/// Is this node currently isolated from the set?
		fn is_isolated(&self) -> bool;
		/// Get server set state.
		fn snapshot(&self) -> KeyServerSetSnapshot;
		/// Start migration.
		fn start_migration(&self, migration_id: H256);
		/// Confirm migration.
		fn confirm_migration(&self, migration_id: H256);
	}
}
*/