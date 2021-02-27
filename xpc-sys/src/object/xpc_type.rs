use crate::{xpc_dictionary_create, xpc_get_type, xpc_type_t};
use std::ptr::{null, null_mut};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
/// Newtype for xpc_type_t
pub struct XPCType(pub xpc_type_t);

unsafe impl Send for XPCType {}
unsafe impl Sync for XPCType {}

lazy_static! {
    pub static ref Dictionary: XPCType =
        unsafe { XPCType(xpc_get_type(xpc_dictionary_create(null(), null_mut(), 0))) };
}
