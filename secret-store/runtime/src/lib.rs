#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::*;

mod blockchain_storage;
mod entity_id_storage;
mod document_key_shadow_retrieval;
mod document_key_storing;
mod key_server_set;
mod key_server_set_storage;
mod mock;
mod server_key_generation;
mod server_key_retrieval;
mod service;
mod utils;

use frame_support::{StorageMap, traits::Currency, decl_module, decl_event, decl_storage, ensure};
use frame_system::{self as system, ensure_signed};
use ss_primitives::{
	KeyServerId, EntityId, NetworkAddress,
	ServerKeyId, ServerKeyPublic, Address,
	key_server_set::{KeyServerSetSnapshot, MigrationId as MigrationIdT},
	CommonPoint, EncryptedPoint,
	service::{ServiceTask, ServiceResponse},
};
use document_key_shadow_retrieval::{
	DocumentKeyShadowRetrievalRequest,
	DocumentKeyShadowRetrievalPersonalData,
	DocumentKeyShadowRetrievalService,
};
use document_key_storing::{DocumentKeyStoreRequest, DocumentKeyStoreService};
use server_key_generation::{ServerKeyGenerationRequest, ServerKeyGenerationService};
use server_key_retrieval::{ServerKeyRetrievalRequest, ServerKeyRetrievalService};
use key_server_set_storage::KeyServer;
use utils::KeyServersMask;

/*

	TODO: every request must have ID and all KS responses should come with this ID
	instead of ServerKeyId. Or:
	1) new_request1
	2) response to request1 from KS1 => request completed
	3) new_request2 for the same key
	4) response to request1 from KS2 => request2 completed

*/

pub type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// The module configuration trait
pub trait Trait: frame_system::Trait {
	/// They overarching event type.
	type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

	/// The currency type used for paying services.
	type Currency: Currency<Self::AccountId>;
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn deposit_event() = default;

		/// Claim given id.
		pub fn claim_id(origin, id: EntityId) {
			ensure!(
				!<ClaimedBy<T>>::exists(&id),
				"Id is already claimed",
			);

			let origin = ensure_signed(origin)?;
			ensure!(
				!<ClaimedId<T>>::exists(&origin),
				"Account has already claimed an id",
			);

			<ClaimedBy<T>>::insert(id, origin.clone());
			<ClaimedId<T>>::insert(origin, id);
		}

/*		/// Change key server set owner.
		pub fn change_owner(origin, new_owner: T::AccountId) {
			KeyServerSetWithMigration::<T>::change_owner(origin, new_owner)?;
		}*/

		/// Complete initialization.
		pub fn complete_initialization(origin) {
			key_server_set::<T>().complete_initialization(origin)?;
		}

		/// Add key server to the set.
		pub fn add_key_server(origin, id: KeyServerId, network_address: NetworkAddress) {
			key_server_set::<T>().add_key_server(origin, id, network_address)?;
		}

		/// Update key server in the set.
		pub fn update_key_server(origin, id: KeyServerId, network_address: NetworkAddress) {
			key_server_set::<T>().update_key_server(origin, id, network_address)?;
		}

		/// Remove key server from the set.
		pub fn remove_key_server(origin, id: KeyServerId) {
			key_server_set::<T>().remove_key_server(origin, id)?;
		}

		/// Start migration.
		pub fn start_migration(origin, migration_id: MigrationIdT) {
			key_server_set::<T>().start_migration(origin, migration_id)?;
		}

		/// Confirm migration.
		pub fn confirm_migration(origin, migration_id: MigrationIdT) {
			key_server_set::<T>().confirm_migration(origin, migration_id)?;
		}

		/// Generate server key.
		pub fn generate_server_key(origin, id: ServerKeyId, threshold: u8) {
			ServerKeyGenerationService::<T>::generate(origin, id, threshold)?;
		}

		/// Publish key server response for service request.
		pub fn service_response(origin, response: ServiceResponse) {
			match response {
				ServiceResponse::ServerKeyGenerated(key_id, key) =>
					ServerKeyGenerationService::<T>::on_generated(origin, key_id, key)?,
				ServiceResponse::ServerKeyGenerationFailed(key_id) =>
					ServerKeyGenerationService::<T>::on_generation_error(origin, key_id)?,
				_ => unimplemented!("TODO"),
			}
		}
/*
		/// Generate server key.
		pub fn generate_server_key(origin, id: ServerKeyId, threshold: u8) {
			ServerKeyGenerationService::<T>::generate(origin, id, threshold)?;
		}

		/// Called when generation is reported by key server.
		pub fn server_key_generated(origin, id: ServerKeyId, server_key_public: ServerKeyPublic) {
			ServerKeyGenerationService::<T>::on_generated(origin, id, server_key_public)?;
		}

		/// Called when generation error is reported by key server.
		pub fn server_key_generation_error(origin, id: ServerKeyId) {
			ServerKeyGenerationService::<T>::on_generation_error(origin, id)?;
		}

		/// Retrieve server key.
		pub fn retrieve_server_key(origin, id: ServerKeyId) {
			ServerKeyRetrievalService::<T>::retrieve(origin, id)?;
		}

		/// Called when generation is reported by key server.
		pub fn server_key_retrieved(origin, id: ServerKeyId, server_key_public: ServerKeyPublic, threshold: u8) {
			ServerKeyRetrievalService::<T>::on_retrieved(origin, id, server_key_public, threshold)?;
		}

		/// Called when generation error is reported by key server.
		pub fn server_key_retrieval_error(origin, id: ServerKeyId) {
			ServerKeyRetrievalService::<T>::on_retrieval_error(origin, id)?;
		}

		/// Store document key.
		pub fn store_document_key(origin, id: ServerKeyId, common_point: CommonPoint, encrypted_point: EncryptedPoint) {
			DocumentKeyStoreService::<T>::store(origin, id, common_point, encrypted_point)?;
		}

		/// Called when store is reported by key server.
		pub fn document_key_stored(origin, id: ServerKeyId) {
			DocumentKeyStoreService::<T>::on_stored(origin, id)?;
		}

		/// Called when store error is reported by key server.
		pub fn document_key_store_error(origin, id: ServerKeyId) {
			DocumentKeyStoreService::<T>::on_store_error(origin, id)?;
		}*/
/*
		/// Allow key operations for given requester.
		pub fn grant_key_access(origin, key: ServerKeyId, requester: Address) {
			let origin = ensure_signed(origin)?;
			ensure!(
				<KeyAccessRights<T>>::exists(&key, &origin),
				"Access to key is denied",
			);

			<KeyAccessRights<T>>::insert(&key, &requester, &());
		}

		/// Deny key operations for given requester.
		pub fn deny_key_access(origin, key: ServerKeyId, requester: Address) {
			let origin = ensure_signed(origin)?;
			ensure!(
				<KeyAccessRights<T>>::exists(&key, &origin),
				"Access to key is denied",
			);

			<KeyAccessRights<T>>::remove(&key, &requester, &());
		}

		/// Set requesters who are allowed to perform operations with given key.
		pub fn change_key_access(origin, key: ServerKeyId, requesters: Vec<Address>) {
			let origin = ensure_signed(origin)?;
			ensure!(
				<KeyAccessRights<T>>::exists(&key, &origin),
				"Access to key is denied",
			);

			<KeyAccessRights<T>>::remove_prefix(&key);
			requesters.for_each(|requester| <KeyAccessRights<T>>::insert(&key, &requester, &()));
		}
*/
	}
}

//<T> where <T as frame_system::Trait>::AccountId
decl_event!(
	pub enum Event {
		/// Key server set: key server added to the new set.
		KeyServerAdded(KeyServerId),
		/// Key server set: key server added to the new set.
		KeyServerRemoved(KeyServerId),
		/// Key server set: key server address has been updated.
		KeyServerUpdated(KeyServerId),
		/// Key server set: migration has started.
		MigrationStarted,
		/// Key server set: migration has completed.
		MigrationCompleted,

		/// 
		ServerKeyGenerationRequested(ServerKeyId, Address, u8),
		///
		ServerKeyGenerated(ServerKeyId, ServerKeyPublic),
		///
		ServerKeyGenerationError(ServerKeyId),

		/// 
		ServerKeyRetrievalRequested(ServerKeyId),
		///
		ServerKeyRetrieved(ServerKeyId, ServerKeyPublic),
		///
		ServerKeyRetrievalError(ServerKeyId),

		///
		DocumentKeyStoreRequested(ServerKeyId, Address, CommonPoint, EncryptedPoint),
		///
		DocumentKeyStored(ServerKeyId),
		///
		DocumentKeyStoreError(ServerKeyId),

		/// TODO: needs to be verified by the key server
		DocumentKeyShadowRetrievalRequested(ServerKeyId, EntityId, ServerKeyPublic),
		///
		DocumentKeyCommonRetrieved(ServerKeyId, EntityId, CommonPoint, u8),
		///
		DocumentKeyPersonalRetrievalRequested(ServerKeyId, ServerKeyPublic),
		///
		DocumentKeyShadowRetrievalError(ServerKeyId, EntityId),
		///
		DocumentKeyPersonalRetrieved(ServerKeyId, EntityId, Vec<u8>, Vec<u8>),
	}
);

decl_storage! {
	trait Store for Module<T: Trait> as SecretStore {
		pub Owner get(owner) config(): T::AccountId;
		ClaimedId get(claimed_address): map T::AccountId => Option<EntityId>;
		ClaimedBy get(claimed_by): map EntityId => Option<T::AccountId>;

		IsInitialized: bool;
		CurrentSetChangeBlock: <T as frame_system::Trait>::BlockNumber;

		CurrentKeyServers: linked_map KeyServerId => Option<KeyServer>;
		MigrationKeyServers: linked_map KeyServerId => Option<KeyServer>;
		NewKeyServers: linked_map KeyServerId => Option<KeyServer>;
		MigrationId: Option<(MigrationIdT, KeyServerId)>;
		MigrationConfirmations: map KeyServerId => ();

		pub ServerKeyGenerationFee get(server_key_generation_fee) config(): BalanceOf<T>;
		ServerKeyGenerationRequestsKeys: Vec<ServerKeyId>;
		ServerKeyGenerationRequests: map ServerKeyId
			=> Option<ServerKeyGenerationRequest<<T as frame_system::Trait>::BlockNumber>>;
		ServerKeyGenerationResponses: double_map ServerKeyId, twox_128(ServerKeyPublic) => u8;

		pub ServerKeyRetrievalFee get(server_key_retrieval_fee) config(): BalanceOf<T>;
		ServerKeyRetrievalRequestsKeys: Vec<ServerKeyId>;
		ServerKeyRetrievalRequests: map ServerKeyId
			=> Option<ServerKeyRetrievalRequest<<T as frame_system::Trait>::BlockNumber>>;
		ServerKeyRetrievalResponses: double_map ServerKeyId, twox_128(ServerKeyPublic) => u8;
		ServerKeyRetrievalThresholdResponses: double_map ServerKeyId, twox_128(u8) => u8;

		pub DocumentKeyStoreFee get(document_key_store_fee) config(): BalanceOf<T>;
		DocumentKeyStoreRequestsKeys: Vec<ServerKeyId>;
		DocumentKeyStoreRequests: map ServerKeyId
			=> Option<DocumentKeyStoreRequest<<T as frame_system::Trait>::BlockNumber>>;
		DocumentKeyStoreResponses: double_map ServerKeyId, twox_128(()) => u8;

		pub DocumentKeyShadowRetrievalFee get(document_key_shadow_retrieval_fee) config(): BalanceOf<T>;
		DocumentKeyShadowRetrievalRequestsKeys: Vec<(ServerKeyId, EntityId)>;
		DocumentKeyShadowRetrievalRequests: map (ServerKeyId, EntityId)
			=> Option<DocumentKeyShadowRetrievalRequest<<T as frame_system::Trait>::BlockNumber>>;
		DocumentKeyShadowRetrievalCommonResponses:
			double_map (ServerKeyId, EntityId),
			twox_128((CommonPoint, u8)) => u8;
		DocumentKeyShadowRetrievalPersonalResponses:
			double_map (ServerKeyId, EntityId),
			twox_128((KeyServersMask, Vec<u8>)) => DocumentKeyShadowRetrievalPersonalData;
	}
	add_extra_genesis {
		config(is_initialization_completed): bool;
		config(key_servers): Vec<(KeyServerId, NetworkAddress)>;
		build(|config|
			key_server_set::<T>()
				.fill(
					&config.key_servers,
					config.is_initialization_completed,
				).expect("invalid key servers set in configuration")
		)
	}
}

impl<T: Trait> Module<T> {
	/// Get snapshot of key servers set state.
	pub fn key_server_set_snapshot(key_server: KeyServerId) -> KeyServerSetSnapshot {
		key_server_set::<T>().snapshot(key_server)
	}

	/// Return count of pending service tasks.
	pub fn service_tasks_count() -> u32 {
		unimplemented!()
	}

	/// Return pending task by index.
	pub fn service_task(index: u32) -> Option<ServiceTask> {
		unimplemented!()
	}

	/// Check if server key generation response is required from given key server.
	pub fn is_server_key_generation_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool {
		ServerKeyGenerationService::<T>::is_response_required(key_server, key)
	}

	/// Check if server key retrieval response is required from given key server.
	pub fn is_server_key_retrieval_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool {
		ServerKeyRetrievalService::<T>::is_response_required(key_server, key)
	}

	/// Check if document key store response is required from given key server.
	pub fn is_document_key_store_response_required(key_server: KeyServerId, key: ServerKeyId) -> bool {
		DocumentKeyStoreService::<T>::is_response_required(key_server, key)
	}
}


pub(crate) type KeyServerSet<T> = key_server_set::KeyServerSetWithMigration<
	blockchain_storage::RuntimeStorage<T>,
	entity_id_storage::RuntimeStorage<T>,
	key_server_set_storage::RuntimeStorageWithMigration<T>,
>;

pub(crate) fn key_server_set<T: Trait>() -> KeyServerSet<T> {
	key_server_set::KeyServerSetWithMigration::with_storage(Default::default(), Default::default(), Default::default())
}

pub fn resolve_entity_id<T: Trait>(origin: &T::AccountId) -> Result<EntityId, &'static str> {
	let origin_id = ClaimedId::<T>::get(origin);
	match origin_id {
		Some(id) => Ok(id),
		None => Err("No associated id for this account"),
	}
}
