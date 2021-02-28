use crate::{xpc_dictionary_create, xpc_get_type, xpc_type_t, xpc_int64_create, xpc_uint64_create, xpc_object_t, xpc_release, mach_port_t};
use std::ptr::{null, null_mut};
use crate::object::xpc_object::XPCObject;
use std::collections::HashMap;
use crate::object::xpc_value::XPCValue;
use std::marker::PhantomData;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
/// Newtype for xpc_type_t
pub struct XPCType(pub xpc_type_t);

unsafe impl Send for XPCType {}
unsafe impl Sync for XPCType {}

lazy_static! {
    pub static ref Dictionary: XPCType =
        unsafe {
            let empty: HashMap<&str, XPCObject> = HashMap::new();
            XPCObject::from(empty).xpc_type()
        };

    pub static ref Int64: XPCType = XPCObject::from(0 as i64).xpc_type();
    pub static ref UInt64: XPCType = XPCObject::from(0 as u64).xpc_type();
    pub static ref String: XPCType = XPCObject::from("").xpc_type();
    pub static ref Bool: XPCType = XPCObject::from(true).xpc_type();
    pub static ref MachPort: XPCType = XPCObject::from(0 as mach_port_t).xpc_type();
}
