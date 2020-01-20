// Copyright 2018-2019 Parity Technologies (UK) Ltd.
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

//! Test utilities

#![cfg(test)]

use std::collections::BTreeMap;
use codec::Encode;
use sp_runtime::Perbill;
use sp_runtime::testing::Header;
use sp_runtime::traits::{IdentityLookup, BlakeTwo256};
use sp_core::{H256, Blake2Hasher};
use frame_support::{impl_outer_origin, impl_outer_event, parameter_types};
use crate::GenesisConfig;
use super::*;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TestRuntime;

mod secret_store {
	pub use crate::Event;
}

impl_outer_event! {
	pub enum TestEvent for TestRuntime {
		pallet_balances<T>, secret_store,
	}
}

impl_outer_origin!{
	pub enum Origin for TestRuntime {}
}

/*impl_outer_dispatch! {
	pub enum Call for Test where origin: Origin {
		balances::Balances,
	}
}*/

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

impl frame_system::Trait for TestRuntime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Call = ();
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = TestEvent;
	type BlockHashCount = BlockHashCount;
	type MaximumBlockWeight = MaximumBlockWeight;
	type MaximumBlockLength = MaximumBlockLength;
	type AvailableBlockRatio = AvailableBlockRatio;
	type Version = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 0;
	pub const TransferFee: u64 = 0;
	pub const CreationFee: u64 = 0;
	pub const TransactionBaseFee: u64 = 1;
	pub const TransactionByteFee: u64 = 0;
}

impl pallet_balances::Trait for TestRuntime {
	type Balance = u64;
	type OnNewAccount = ();
	type OnFreeBalanceZero = ();
	type Event = TestEvent;
	type TransferPayment = ();
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type TransferFee = TransferFee;
	type CreationFee = CreationFee;
}

impl Trait for TestRuntime {
	type Event = TestEvent;
	type Currency = pallet_balances::Module<Self>;
}

pub const OWNER: u64 = 1;
pub const REQUESTER1: u64 = 2;
pub const REQUESTER2: u64 = 3;
pub const KEY_SERVER0: u64 = 100;
pub const KEY_SERVER1: u64 = 101;
pub const KEY_SERVER2: u64 = 102;
pub const KEY_SERVER3: u64 = 103;
pub const KEY_SERVER4: u64 = 104;

pub const KEY_SERVER0_ID: KeyServerId = [KEY_SERVER0 as u8; 32];
pub const KEY_SERVER1_ID: KeyServerId = [KEY_SERVER1 as u8; 32];
pub const KEY_SERVER2_ID: KeyServerId = [KEY_SERVER2 as u8; 32];
pub const KEY_SERVER3_ID: KeyServerId = [KEY_SERVER3 as u8; 32];
pub const KEY_SERVER4_ID: KeyServerId = [KEY_SERVER4 as u8; 32];

pub fn ordered_set(set: Vec<(KeyServerId, NetworkAddress)>) -> Vec<(KeyServerId, NetworkAddress)> {
	set.into_iter().collect::<BTreeMap<_, _>>().into_iter().collect()
}

pub fn default_key_server_set() -> Vec<(KeyServerId, NetworkAddress)> {
	vec![
		(KEY_SERVER0_ID, KEY_SERVER0_ID.to_vec()),
		(KEY_SERVER1_ID, KEY_SERVER1_ID.to_vec()),
	]
}

pub fn default_key_server_set3() -> Vec<(KeyServerId, NetworkAddress)> {
	vec![
		(KEY_SERVER0_ID, KEY_SERVER0_ID.to_vec()),
		(KEY_SERVER1_ID, KEY_SERVER1_ID.to_vec()),
		(KEY_SERVER2_ID, KEY_SERVER2_ID.to_vec()),
	]
}

pub fn default_key_server_set5() -> Vec<(KeyServerId, NetworkAddress)> {
	vec![
		(KEY_SERVER0_ID, KEY_SERVER0_ID.to_vec()),
		(KEY_SERVER1_ID, KEY_SERVER1_ID.to_vec()),
		(KEY_SERVER2_ID, KEY_SERVER2_ID.to_vec()),
		(KEY_SERVER3_ID, KEY_SERVER3_ID.to_vec()),
		(KEY_SERVER4_ID, KEY_SERVER4_ID.to_vec()),
	]
}

fn initialize(
	is_initialization_completed: bool,
	key_server_set: Vec<(KeyServerId, NetworkAddress)>,
) -> sp_io::TestExternalities {
	let mut t = system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap();
	let config = GenesisConfig::<TestRuntime> {
		owner: OWNER,
		is_initialization_completed,
		key_servers: key_server_set,
		server_key_generation_fee: 1_000_000,
		server_key_retrieval_fee: 1_000_000,
		document_key_store_fee: 1_000_000,
		document_key_shadow_retrieval_fee: 1_000_000,
	};
	config.assimilate_storage(&mut t).unwrap();
	let config = pallet_balances::GenesisConfig::<TestRuntime> {
		balances: vec![
			(OWNER, 10_000_000),
			(REQUESTER1, 10_000_000),
		],
		vesting: vec![],
	};
	config.assimilate_storage(&mut t).unwrap();

	t.top.insert(ClaimedId::<TestRuntime>::hashed_key_for(&OWNER), [OWNER as u8; 32].encode());
	t.top.insert(ClaimedId::<TestRuntime>::hashed_key_for(&REQUESTER1), [REQUESTER1 as u8; 32].encode());
	t.top.insert(ClaimedId::<TestRuntime>::hashed_key_for(&KEY_SERVER0), KEY_SERVER0_ID.encode());
	t.top.insert(ClaimedId::<TestRuntime>::hashed_key_for(&KEY_SERVER1), KEY_SERVER1_ID.encode());
	t.top.insert(ClaimedId::<TestRuntime>::hashed_key_for(&KEY_SERVER2), KEY_SERVER2_ID.encode());
	t.top.insert(ClaimedId::<TestRuntime>::hashed_key_for(&KEY_SERVER3), KEY_SERVER3_ID.encode());
	t.top.insert(ClaimedId::<TestRuntime>::hashed_key_for(&KEY_SERVER4), KEY_SERVER4_ID.encode());
	t.top.insert(ClaimedBy::<TestRuntime>::hashed_key_for(&[OWNER as u8; 32]), OWNER.encode());
	t.top.insert(ClaimedBy::<TestRuntime>::hashed_key_for(&[REQUESTER1 as u8; 32]), REQUESTER1.encode());
	t.top.insert(ClaimedBy::<TestRuntime>::hashed_key_for(&KEY_SERVER0_ID), KEY_SERVER0.encode());
	t.top.insert(ClaimedBy::<TestRuntime>::hashed_key_for(&KEY_SERVER1_ID), KEY_SERVER1.encode());
	t.top.insert(ClaimedBy::<TestRuntime>::hashed_key_for(&KEY_SERVER2_ID), KEY_SERVER2.encode());
	t.top.insert(ClaimedBy::<TestRuntime>::hashed_key_for(&KEY_SERVER3_ID), KEY_SERVER3.encode());
	t.top.insert(ClaimedBy::<TestRuntime>::hashed_key_for(&KEY_SERVER4_ID), KEY_SERVER4.encode());

	t.into()
}

pub fn empty_initialization() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<TestRuntime>().unwrap().into()
}

pub fn basic_initialization() -> sp_io::TestExternalities {
	initialize(false, default_key_server_set())
}

pub fn default_initialization() -> sp_io::TestExternalities {
	initialize(true, default_key_server_set())
}

pub fn default_initialization_with_three_servers() -> sp_io::TestExternalities {
	initialize(true, default_key_server_set3())
}

pub fn default_initialization_with_five_servers() -> sp_io::TestExternalities {
	initialize(true, default_key_server_set5())
}
