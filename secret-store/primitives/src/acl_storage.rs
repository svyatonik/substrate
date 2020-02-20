use crate::{EntityId, ServerKeyId};

sp_api::decl_runtime_apis! {
	/// API for checking key access rights.
	pub trait SecretStoreAclApi {
		/// Check if requestor can perform operations that are involving key
		/// with the given ID.
		fn check(requester: EntityId, key: ServerKeyId) -> bool;
	}
}
