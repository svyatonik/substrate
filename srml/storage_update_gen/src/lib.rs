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

#![cfg_attr(not(feature = "std"), no_std)]

use srml_support::{StorageValue, storage, decl_module, decl_storage};

/// The module configuration trait
pub trait Trait: system::Trait {
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		fn on_finalize() {
			let unique_range_len = Self::unique_range_length();
			let updates_per_block = Self::updates_per_block();

			let mut unique_range_index = Self::unique_range_index();
			if unique_range_index == unique_range_len {
				unique_range_index = 0;
			}

			for i in 0..updates_per_block {
				let unique_range_index_le = unique_range_index.to_le_bytes();
				let i_le = i.to_le_bytes();
				let storage_key = [
					unique_range_index_le[0],
					unique_range_index_le[1],
					unique_range_index_le[2],
					unique_range_index_le[3],
					i_le[0],
					i_le[1],
					i_le[2],
					i_le[3],
				];
				let storage_value = &storage_key;
				storage::unhashed::put_raw(&storage_key, storage_value);
			}

			<Self as Store>::UniqueRangeIndex::put(unique_range_index + 1);
		}
	}
}

decl_storage! {
	trait Store for Module<T: Trait> as Timestamp {
		/// Blocks range where all updates should be unique.
		///
		/// 1 means that every block will have the same keys updated.
		/// 1000 meanse that all storage updates in first 1_000 blocks will be unique.
		UniqueRangeLength get(unique_range_length): u32 = 1_000;
		/// Number of storage updates for each block. Restarts when finished.
		UpdatesPerBlock get(updates_per_block): u32 = 5_000;

		/// Current index in UniqueRange.
		UniqueRangeIndex get(unique_range_index): u32 = 0;
	}
}

impl<T: Trait> Module<T> {
}
