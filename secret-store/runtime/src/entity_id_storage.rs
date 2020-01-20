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

use frame_support::StorageMap;
use frame_system::ensure_signed;
use sp_std::marker::PhantomData;
use ss_primitives::KeyServerId;
use crate::{ClaimedId, Trait};

/// Entity id related data storage.
pub(crate) trait Storage {
	/// Block number type.
	type EntityId;

	/// Resolve entity id into key server id.
	fn resolve_key_server_id(&self, id: Self::EntityId) -> Result<KeyServerId, &'static str>;
}

/// The storage of single key server set.
pub(crate) struct RuntimeStorage<T>(PhantomData<T>);

impl<T: Trait> Storage for RuntimeStorage<T> {
	type EntityId = T::Origin;

	fn resolve_key_server_id(&self, id: Self::EntityId) -> Result<KeyServerId, &'static str> {
		let account = ensure_signed(id)?;
		let id = ClaimedId::<T>::get(account);
		match id {
			Some(id) => Ok(id),
			None => Err("No associated id for this account"),
		}
	}
}

impl<T> Default for RuntimeStorage<T> {
	fn default() -> Self {
		RuntimeStorage(Default::default())
	}
}
