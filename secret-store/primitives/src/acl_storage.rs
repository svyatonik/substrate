use crate::{RequesterId, ServerKeyId};

sp_api::decl_runtime_apis! {
	/// API for checking key access rights.
	pub trait AclStorageRuntimeApi {
		/// Check if requestor can perform operations that are involving key
		/// with the given ID.
		fn check(requester: RequesterId, key: ServerKeyId) -> bool;
	}
}
