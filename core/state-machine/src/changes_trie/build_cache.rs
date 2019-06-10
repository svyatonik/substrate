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

//! Changes tries build cache.

use std::collections::{HashMap, HashSet};

/// Changes trie build cache.
///
/// Helps to avoid read of changes tries from the database when digest block
/// is built. It holds changed keys for every block (changes trie root) that
/// could be referenced by future digest items. For digest entries it also holds
/// keys covered by this digest.
///
/// Entries are pruned from the cache once digest block that is using this entry
/// is inserted (because digest block will includes all keys from this entry).
/// When there's a fork, entries are pruned when first changes trie is inserted.
pub struct BuildCache<H, N> {
	/// Map of block (implies changes true) number => changes trie root.
	roots_by_number: HashMap<N, H>,
	/// Map of changes trie root =>
	changed_keys: HashMap<H, HashSet<Vec<u8>>>,
}

/// The data that has been cached during changes trie building.
#[derive(Debug, PartialEq)]
pub struct CachedBuildData<H, N> {
	trie_root: H,
	digest_input_blocks: Vec<N>,
	changed_keys: HashSet<Vec<u8>>,
}

/// The data (without changes trie root) that has been cached during changes trie building.
#[derive(Debug, PartialEq)]
pub(crate) struct IncompleteCachedBuildData<N> {
	digest_input_blocks: Vec<N>,
	changed_keys: HashSet<Vec<u8>>,
}

impl<H, N> BuildCache<H, N>
	where
		N: Eq + ::std::hash::Hash,
		H: Eq + ::std::hash::Hash,
{
	/// Create new changes trie build cache.
	pub fn new() -> Self {
		BuildCache {
			roots_by_number: HashMap::new(),
			changed_keys: HashMap::new(),
		}
	}

	/// Get cached changed keys for changes trie with given root.
	pub fn get(&self, root: &H) -> Option<&HashSet<Vec<u8>>> {
		self.changed_keys.get(&root)
	}

	/// Insert data into cache.
	pub fn insert(&mut self, data: CachedBuildData<H, N>) {
		self.changed_keys.insert(data.trie_root, data.changed_keys);

		for digest_input_block in data.digest_input_blocks {
			let digest_input_block_hash = self.roots_by_number.remove(&digest_input_block);
			if let Some(digest_input_block_hash) = digest_input_block_hash {
				self.changed_keys.remove(&digest_input_block_hash);
			}
		}
	}
}

impl<N> IncompleteCachedBuildData<N> {
	/// Create new cached data.
	pub(crate) fn new() -> Self {
		IncompleteCachedBuildData {
			digest_input_blocks: Vec::new(),
			changed_keys: HashSet::new(),
		}
	}

	/// Complete cached data with computed changes trie root.
	pub(crate) fn complete<H>(self, trie_root: H) -> CachedBuildData<H, N> {
		CachedBuildData {
			trie_root,
			digest_input_blocks: self.digest_input_blocks,
			changed_keys: self.changed_keys,
		}
	}

	/// Called for digest entries only. Set numbers of blocks that are superseded
	/// by this new entry.
	pub(crate) fn set_digest_input_blocks(&mut self, digest_input_blocks: Vec<N>) {
		self.digest_input_blocks = digest_input_blocks;
	}

	/// Insert changed keys into cached data.
	pub(crate) fn insert<I: IntoIterator<Item=Vec<u8>>>(&mut self, changed_keys: I) {
		self.changed_keys.extend(changed_keys)
	}
}
