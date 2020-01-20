// Copyright 2017-2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

//! Contains actual implementation of all public/private module methods
//! for storing document keys.

use codec::{Encode, Decode};
use frame_support::{StorageValue, StorageMap, StorageDoubleMap, ensure};
use ss_primitives::{EntityId, ServerKeyId, KeyServerId, CommonPoint, EncryptedPoint};
use frame_system::ensure_signed;
use crate::service::{Responses, ResponseSupport, SecretStoreService};
use super::{
	Trait, Module, Event,
	DocumentKeyStoreFee,
	DocumentKeyStoreRequests, DocumentKeyStoreRequestsKeys,
	DocumentKeyStoreResponses,
	resolve_entity_id,
};

/// Maximal number of active requests in the queue.
const MAX_REQUESTS: u64 = 8;

/// Structure that describes document key store request with responses meta.
#[derive(Decode, Encode)]
pub struct DocumentKeyStoreRequest<Number> {
	/// The author of this request. It must be the same author as in the
	/// server key generation request.
	pub author: EntityId,
	/// Common point of the document key.
	pub common_point: CommonPoint,
	/// Encrypted point of the document key.
	pub encrypted_point: EncryptedPoint,
	/// Responses metadata.
	pub responses: Responses<Number>,
}

/// Implementation of document key storing service.
pub struct DocumentKeyStoreService<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait> DocumentKeyStoreService<T> {
	/// Request storing of new document key.
	pub fn store(
		origin: T::Origin,
		id: ServerKeyId,
		common_point: CommonPoint,
		encrypted_point: EncryptedPoint,
	) -> Result<(), &'static str> {
		// limit number of requests in the queue
		ensure!(
			(DocumentKeyStoreRequestsKeys::decode_len()? as u64) < MAX_REQUESTS,
			"Too many active requests. Try later",
		);

		// check if there are no pending request for the same key
		ensure!(
			!DocumentKeyStoreRequests::<T>::exists(id),
			"The same request is already queued",
		);

		// collect service fee
		let origin = ensure_signed(origin)?;
		let fee = DocumentKeyStoreFee::<T>::get();
		SecretStoreService::<T>::collect_service_fee(&origin, fee)?;

		// insert request to the queue
		let author = resolve_entity_id::<T>(&origin)?;
		let request = DocumentKeyStoreRequest {
			author: author.clone(),
			common_point: common_point.clone(),
			encrypted_point: encrypted_point.clone(),
			responses: SecretStoreService::<T>::new_responses(),
		};
		DocumentKeyStoreRequests::<T>::insert(id, request);
		DocumentKeyStoreRequestsKeys::append(sp_std::iter::once(&id))?;

		// emit event
		Module::<T>::deposit_event(Event::DocumentKeyStoreRequested(id, author, common_point, encrypted_point));

		Ok(())
	}

	/// Called when storing is reported by key server.
	pub fn on_stored(
		origin: T::Origin,
		id: ServerKeyId,
	) -> Result<(), &'static str> {
		// check if this request is active (the tx could arrive when request is already inactive)
		let mut request = match DocumentKeyStoreRequests::<T>::get(id) {
			Some(request) => request,
			None => return Ok(()),
		};

		// insert response (we're waiting for responses from all authorities here)
		let key_servers_count = SecretStoreService::<T>::key_servers_count()?;
		let key_server_index = SecretStoreService::<T>::key_server_index_from_origin(origin)?;
		let response_support = SecretStoreService::<T>::insert_response::<_, _, DocumentKeyStoreResponses>(
			key_server_index,
			key_servers_count - 1,
			&mut request.responses,
			&id,
			&(),
		)?;

		// check if response is confirmed
		match response_support {
			ResponseSupport::Unconfirmed => {
				DocumentKeyStoreRequests::<T>::insert(id, request);
			},
			ResponseSupport::Confirmed => {
				// we do not need this request anymore
				delete_request::<T>(&id);

				// emit event
				Module::<T>::deposit_event(Event::DocumentKeyStored(id));
			},
			ResponseSupport::Impossible => unreachable!("we're receiving the same response from all servers; qed"),
		}

		Ok(())
	}

	/// Called when error occurs during document key storing.
	pub fn on_store_error(
		origin: T::Origin,
		id: ServerKeyId,
	) -> Result<(), &'static str> {
		// check that it is reported by the key server
		let _ = SecretStoreService::<T>::key_server_index_from_origin(origin)?;

		// check if this request is active (the tx could arrive when request is already inactive)
		let _request = match DocumentKeyStoreRequests::<T>::get(id) {
			Some(request) => request,
			None => return Ok(()),
		};

		// any error in key generation is fatal, because we need all key servers to participate in generation
		// => delete request and fire event
		delete_request::<T>(&id);

		Module::<T>::deposit_event(Event::DocumentKeyStoreError(id));
		Ok(())
	}

	/// Returns true if response from given key server is required to complete request.
	pub fn is_response_required(
		key_server: KeyServerId,
		id: ServerKeyId,
	) -> bool {
		DocumentKeyStoreRequests::<T>::get(&id)
			.map(|request| SecretStoreService::<T>::is_response_required(
				key_server,
				&request.responses,
			))
			.unwrap_or(false)
	}
}

fn delete_request<T: Trait>(request: &ServerKeyId) {
	DocumentKeyStoreResponses::remove_prefix(request);
	DocumentKeyStoreRequests::<T>::remove(request);
	DocumentKeyStoreRequestsKeys::mutate(|list| {
		let index = list.iter().position(|lrequest| lrequest == request);
		if let Some(index) = index {
			list.swap_remove(index);
		}
	});
}

#[cfg(test)]
mod tests {
	use crate::mock::*;
	use super::*;

	#[test]
	fn should_accept_document_key_store_request() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();

			// check that event has been emitted
			assert!(
				frame_system::Module::<TestRuntime>::events().into_iter()
					.find(|e| e.event == Event::DocumentKeyStoreRequested(
						[32; 32],
						[REQUESTER1 as u8; 32],
						vec![21],
						vec![42],
					).into())
					.is_some(),
			);
		});
	}

	#[test]
	fn should_reject_document_key_store_request_when_fee_is_not_paid() {
		default_initialization().execute_with(|| {
			// REQUESTER2 has no enough funds
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER2),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap_err();
		});
	}

	#[test]
	fn should_reject_document_key_store_request_when_limit_reached() {
		default_initialization().execute_with(|| {
			// make MAX_REQUESTS requests
			for i in 0..MAX_REQUESTS {
				DocumentKeyStoreService::<TestRuntime>::store(
					Origin::signed(REQUESTER1),
					[i as u8; 32],
					vec![21],
					vec![42],
				).unwrap();
			}

			// and now try to push new request so that there will be more than a limit requests
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[MAX_REQUESTS as u8; 32],
				vec![21],
				vec![42],
			).unwrap_err();
		});
	}

	#[test]
	fn should_reject_duplicated_document_key_store_request() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();

			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap_err();
		});
	}

	#[test]
	fn should_publish_document_key_store_confirmation() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();
			let events_count = frame_system::Module::<TestRuntime>::events().len();

			// response from key server 1 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER0),
				[32; 32],
			).unwrap();
			// => no new events generated
			assert_eq!(
				events_count,
				frame_system::Module::<TestRuntime>::events().len(),
			);

			// response from key server 2 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER1),
				[32; 32],
			).unwrap();
			// => new event is generated
			assert_eq!(
				events_count + 1,
				frame_system::Module::<TestRuntime>::events().len(),
			);
			assert!(
				frame_system::Module::<TestRuntime>::events().into_iter()
					.find(|e| e.event == Event::DocumentKeyStored([32; 32]).into())
					.is_some(),
			);

			// and then another response from key server 2 is received (and ignored without error)
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER1),
				[32; 32],
			).unwrap();
		});
	}

	#[test]
	fn should_not_accept_store_confirmation_from_non_key_server() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();

			// response from key server 3 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER3),
				[32; 32],
			).unwrap_err();
		});
	}

	#[test]
	fn should_not_publish_generated_key_when_receiving_responses_from_same_key_server() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();
			let events_count = frame_system::Module::<TestRuntime>::events().len();

			// response from key server 1 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER1),
				[32; 32],
			).unwrap();

			// another response from key server 1 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER1),
				[32; 32],
			).unwrap();

			// check that key is not published
			assert_eq!(
				events_count,
				frame_system::Module::<TestRuntime>::events().len(),
			);
		});
	}

	#[test]
	fn should_raise_store_error_when_at_least_one_server_has_responded_with_error() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();
			let events_count = frame_system::Module::<TestRuntime>::events().len();

			// error from key server 1 is received
			DocumentKeyStoreService::<TestRuntime>::on_store_error(
				Origin::signed(KEY_SERVER0),
				[32; 32],
			).unwrap();

			// check that store error is published
			assert_eq!(
				events_count + 1,
				frame_system::Module::<TestRuntime>::events().len(),
			);
			assert!(
				frame_system::Module::<TestRuntime>::events().into_iter()
					.find(|e| e.event == Event::DocumentKeyStoreError([32; 32]).into())
					.is_some(),
			);
		});
	}

	#[test]
	fn should_fail_if_store_error_is_reported_by_non_key_server() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();

			// error from REQUESTER1 is received
			DocumentKeyStoreService::<TestRuntime>::on_store_error(
				Origin::signed(REQUESTER1),
				[32; 32],
			).unwrap_err();
		});
	}

	#[test]
	fn should_not_raise_store_error_if_no_active_request() {
		default_initialization().execute_with(|| {
			// error from key server 1 is received
			DocumentKeyStoreService::<TestRuntime>::on_store_error(
				Origin::signed(KEY_SERVER0),
				[32; 32],
			).unwrap();

			assert_eq!(
				0,
				frame_system::Module::<TestRuntime>::events().len(),
			);
		});
	}

	#[test]
	fn should_return_if_store_response_is_required() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();

			// response from all key servers is required
			assert!(DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER0_ID, [32; 32]));
			assert!(DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER1_ID, [32; 32]));

			// response from key server 1 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER0),
				[32; 32],
			).unwrap();

			// response from key server 2 is required
			assert!(!DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER0_ID, [32; 32]));
			assert!(DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER1_ID, [32; 32]));

			// response from key server 2 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER1),
				[32; 32],
			).unwrap();

			// no responses are required
			assert!(!DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER0_ID, [32; 32]));
			assert!(!DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER1_ID, [32; 32]));
		});
	}

	#[test]
	fn should_reset_existing_responses_when_key_server_set_changes() {
		default_initialization().execute_with(|| {
			// ask to store document key
			DocumentKeyStoreService::<TestRuntime>::store(
				Origin::signed(REQUESTER1),
				[32; 32],
				vec![21],
				vec![42],
			).unwrap();

			// response from key server 1 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER0),
				[32; 32],
			).unwrap();

			// response from key server 1 is not required anymore
			assert!(!DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER0_ID, [32; 32]));

			// let's simulate migration
			crate::CurrentSetChangeBlock::<TestRuntime>::put(100);

			// response from key server 2 is received
			DocumentKeyStoreService::<TestRuntime>::on_stored(
				Origin::signed(KEY_SERVER1),
				[32; 32],
			).unwrap();

			// response from key server 1 is required again
			assert!(DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER0_ID, [32; 32]));
			assert!(!DocumentKeyStoreService::<TestRuntime>::is_response_required(KEY_SERVER1_ID, [32; 32]));
		});
	}
}
