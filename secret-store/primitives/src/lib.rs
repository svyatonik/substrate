#![cfg_attr(not(feature = "std"), no_std)]

mod key_servers_mask;

pub mod acl_storage;
pub mod key_server_set;
pub mod service;

pub use key_servers_mask::KeyServersMask;

/// Server key id.
pub type ServerKeyId = sp_core::H256;
/// Entity address.
pub type EntityId = sp_core::H160;
/// Key server address.
pub type KeyServerId = sp_core::H160;


/*///
pub type ServerKeyId = sp_core::H256;

///
pub type ServerKeyPublic = sp_core::H512;

///
pub type CommonPoint = sp_core::H512;

///
pub type EncryptedPoint = sp_core::H512;

pub type DecryptedSecret = sp_core::H512;

pub type DocumentKeyShadow = Vec<u8>;

*/