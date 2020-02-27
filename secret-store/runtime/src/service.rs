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

//! Common methods for all service.

use frame_system::ensure_signed;
use frame_support::{traits::{Currency, ExistenceRequirement}, StorageValue, StorageMap, StorageDoubleMap, StorageLinkedMap};
use ss_primitives::{KeyServerId, KeyServersMask};
use sp_std::vec::Vec;
use super::{
	BalanceOf,
	Trait,
	ClaimedId, ClaimedBy,
	CurrentKeyServers, CurrentSetChangeBlock,
};
use codec::{/*Codec, */Decode, Encode, EncodeLike};

/*/// Map of request key => request details.
pub struct RequestDetails<RequestKey, Request, List, Map>(std::marker::PhantomData<(RequestKey, Request, List, Map)>);

impl<RequestKey: PartialEq + Codec, Request: Codec, List: StorageValue<Vec<RequestKey>, Query=Vec<RequestKey>>, Map: StorageMap<RequestKey, Option<Request>, Query=Option<Request>>> RequestDetails<RequestKey, Request, List, Map> {
	/// Remove given request from the storage.
	pub fn xxx(request: &RequestKey) {
		Map::remove(request);
		List::mutate(|list| {
			let index = list.iter().position(|lrequest| lrequest == request);
			if let Some(index) = index {
				list.swap_remove(index);
			}
		})
	}
}*/

/// Map of response => number of key servers that have supported this response.
pub struct ResponsesSupport<RequestKey, Response, Map>(sp_std::marker::PhantomData<(RequestKey, Response, Map)>);

impl<RequestKey: EncodeLike, Response: EncodeLike, Map: StorageDoubleMap<RequestKey, Response, u8, Query=u8>> ResponsesSupport<RequestKey, Response, Map> {
	/// Increase support of given response by one. Returns new support
	pub fn support(request: &RequestKey, response: &Response) -> u8 {
		Map::mutate(request, response, |responses_count| {
			*responses_count = *responses_count + 1;
			*responses_count
		})
	}

	/// Clear all known responses.
	pub fn reset(request: &RequestKey) {
		Map::remove_prefix(request);
	}
}

/// The structure contains some meta fields that are describing actual responses from key servers.
#[derive(Decode, Encode)]
pub struct Responses<BlockNumber> {
	/// Number of block when servers set has been changed last time.
	/// This whole structure is valid when this value stays the same.
	/// Once this changes, all previous responses are erased.
	pub key_servers_change_block: BlockNumber,
	/// If bit is set, in this mask, this means that corresponding key server has already voted
	/// for some response (we do not care about exact response).
	pub responded_key_servers_mask: KeyServersMask,
	/// Number of key servers that have responded to request (number of ones in responded_key_servers_mask).
	pub responded_key_servers_count: u8,
	/// Maximal support of single response.
	pub max_response_support: u8,
}

/// How's response is supported by the current key server set.
#[derive(Debug, PartialEq)]
pub enum ResponseSupport {
	/// The response is not yet confirmed. More key servers should support this response
	/// to make it confirmed.
	Unconfirmed,
	/// The response is confirmed by required number of key servers.
	Confirmed,
	/// Key servers are unable to agree on supporting any response.
	Impossible,
}

/// Implementation of key server set with migration support
pub struct SecretStoreService<T>(sp_std::marker::PhantomData<T>);

impl<T: Trait> SecretStoreService<T> {
	/// Creates new responses structure.
	pub fn new_responses() -> Responses<<T as frame_system::Trait>::BlockNumber> {
		Responses {
			key_servers_change_block: CurrentSetChangeBlock::<T>::get(),
			responded_key_servers_mask: Default::default(),
			responded_key_servers_count: 0,
			max_response_support: 0,
		}
	}

	/// Return number of key servers in the current set.
	pub fn key_servers_count() -> Result<u8, &'static str> {
		Ok(CurrentKeyServers::enumerate().count() as u8) // TODO: optimize?
	}

	///
	pub fn key_server_index_from_origin(origin: T::Origin) -> Result<u8, &'static str> {
		let origin = ensure_signed(origin)?;
		let origin_id = ClaimedId::<T>::get(&origin).ok_or("the caller has not claimed any id")?;
		Self::key_server_index_from_id(origin_id)
	}

	///
	pub fn key_server_index_from_id(id: KeyServerId) -> Result<u8, &'static str> {
		CurrentKeyServers::get(id)
			.map(|ks| ks.index)
			.ok_or("the caller is not a key server")
	}

	/// Deposit equal share of amount to each of key servers.
	pub fn collect_service_fee(origin: &T::AccountId, fee: BalanceOf<T>) -> Result<(), &'static str> {
		let key_servers_ids = CurrentKeyServers::enumerate().map(|(id, _)| id).collect::<Vec<_>>();
		let key_servers_count = key_servers_ids.len() as u8;
		let key_server_fee_share = fee / key_servers_count.into();
		let mut fee_rest = fee;
		for i in 0..key_servers_ids.len() - 1 {
			let key_server_id = key_servers_ids[i];
			let key_server_account = ClaimedBy::<T>::get(&key_server_id).ok_or("key server has not claimed id")?;
			T::Currency::transfer(origin, &key_server_account, key_server_fee_share, ExistenceRequirement::AllowDeath)?;
			fee_rest -= key_server_fee_share;
		}

		let key_server_id = key_servers_ids[key_servers_ids.len() - 1];
		let key_server_account = ClaimedBy::<T>::get(&key_server_id).ok_or("key server has not claimed id")?;
		T::Currency::transfer(origin, &key_server_account, fee_rest, ExistenceRequirement::AllowDeath)?;

		Ok(())
	}

	///
	pub fn insert_response<RequestKey, Response, Map>(
		key_server_index: u8,
		threshold: u8,
		responses: &mut Responses<<T as frame_system::Trait>::BlockNumber>,
		request: &RequestKey,
		response: &Response,
	) -> Result<ResponseSupport, &'static str> where
		RequestKey: EncodeLike,
		Response: EncodeLike,
		Map: StorageDoubleMap<RequestKey, Response, u8, Query=u8>,
	{
		// early return (this is the only fn that can fail here)
		let key_servers_count = Self::key_servers_count()?;

		// check that servers set is still the same (and all previous responses are valid)
		let key_servers_change_block = CurrentSetChangeBlock::<T>::get();
		if responses.responded_key_servers_count == 0 {
			responses.key_servers_change_block = key_servers_change_block;
		} else if responses.key_servers_change_block != key_servers_change_block {
			responses.key_servers_change_block = key_servers_change_block;
			responses.responded_key_servers_mask = Default::default();
			responses.responded_key_servers_count = 0;
			responses.max_response_support = 0;
			ResponsesSupport::<RequestKey, Response, Map>::reset(request);
		}

		// check if key server has already responded
		let key_server_mask = KeyServersMask::from_index(key_server_index);
		let updated_responded_key_servers_mask = responses.responded_key_servers_mask.union(key_server_mask);
		if updated_responded_key_servers_mask == responses.responded_key_servers_mask {
			return Ok(ResponseSupport::Unconfirmed);
		}

		// insert response
		let response_support = ResponsesSupport::<RequestKey, Response, Map>::support(request, response);
		responses.responded_key_servers_mask = updated_responded_key_servers_mask;
		responses.responded_key_servers_count = responses.responded_key_servers_count + 1;
		if response_support >= responses.max_response_support {
			responses.max_response_support = response_support;

			// check if passed response has received enough support
			if threshold <= response_support - 1 {
				return Ok(ResponseSupport::Confirmed);
			}
		}

		// check if max confirmation CAN receive enough support
		let key_servers_left = key_servers_count - responses.responded_key_servers_count;
		if threshold > responses.max_response_support + key_servers_left - 1 {
			return Ok(ResponseSupport::Impossible);
		}

		Ok(ResponseSupport::Unconfirmed)
	}

	/// Returns true if resonse is required.
	pub fn is_response_required(
		key_server: KeyServerId,
		responses: &Responses<<T as frame_system::Trait>::BlockNumber>
	) -> bool {
		let key_server_index = match SecretStoreService::<T>::key_server_index_from_id(key_server) {
			Ok(key_server_index) => key_server_index,
			Err(_) => return false,
		};

		let key_servers_change_block = CurrentSetChangeBlock::<T>::get();
		key_servers_change_block != responses.key_servers_change_block
			|| !responses.responded_key_servers_mask.is_set(key_server_index)
	}
}
