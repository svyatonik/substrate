use codec::{Decode, Encode};
use sp_core::H256;
use sp_std::vec::Vec;
use crate::KeyServerId;

/// Migration id.
pub type MigrationId = H256;

/// Key server node address.
pub type KeyServerAddress = Vec<u8>;

/// Key server set snapshot.
#[derive(Decode, Encode, PartialEq)]
pub struct KeyServerSetSnapshot {
	/// Current set of key servers.
	pub current_set: Vec<(KeyServerId, KeyServerAddress)>,
	/// New set of key servers.
	pub new_set: Vec<(KeyServerId, KeyServerAddress)>,
	/// Current migration data.
	pub migration: Option<KeyServerSetMigration>,
}

/// Key server set migration.
#[derive(Decode, Encode, PartialEq)]
pub struct KeyServerSetMigration {
	/// Migration id.
	pub id: MigrationId,
	/// Migration set of key servers. It is the new_set at the moment of migration start.
	pub set: Vec<(KeyServerId, KeyServerAddress)>,
	/// Master node of the migration process.
	pub master: KeyServerId,
	/// Is migration confirmed by this node?
	pub is_confirmed: bool,
}

sp_api::decl_runtime_apis! {
	/// Runtime API that backs the key server set.
	pub trait KeyServerSetWithMigrationRuntimeApi {
		/// Get server set state.
		fn snapshot(key_server: KeyServerId) -> KeyServerSetSnapshot;
	}
}

#[cfg(feature = "std")]
impl Into<crate::secret_store::KeyServerSetSnapshot> for KeyServerSetSnapshot {
	fn into(self) -> crate::secret_store::KeyServerSetSnapshot {
		crate::secret_store::KeyServerSetSnapshot {
			current_set: self.current_set.into_iter()
				.map(|ks| (ks.0.into(), ks.1.into()))
				.collect(),
			new_set: self.new_set.into_iter()
				.map(|ks| (ks.0.into(), ks.1.into()))
				.collect(),
			migration: self.migration.map(|migration| crate::secret_store::KeyServerSetMigration {
				id: migration.id.into(),
				set: migration.set.into_iter()
					.map(|ks| (ks.0.into(), ks.1.into()))
					.collect(),
				master: migration.master.into(),
				is_confirmed: migration.is_confirmed,
			}),
		}
	}
}
