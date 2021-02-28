use crate::{
    mach_port_t, xpc_dictionary_create, xpc_get_type, xpc_int64_create, xpc_object_t, xpc_release,
    xpc_type_t, xpc_uint64_create,
};

use crate::object::xpc_object::XPCObject;
use std::collections::HashMap;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
/// Newtype for xpc_type_t
pub struct XPCType(pub xpc_type_t);

unsafe impl Send for XPCType {}
unsafe impl Sync for XPCType {}

impl From<xpc_object_t> for XPCType {
    fn from(value: xpc_object_t) -> XPCType {
        XPCType(unsafe { xpc_get_type(value) })
    }
}

impl From<XPCObject> for XPCType {
    fn from(value: XPCObject) -> XPCType {
        let XPCObject(_, xpc_type) = value;
        xpc_type
    }
}

// TODO: Why is this broken? Even with lazy_static!
// impl From<_xpc_type_s> for XPCType {
//     /// Mostly for use with externs like _xpc_type_dictionary
//     /// from bindgen
//     fn from(value: _xpc_type_s) -> XPCType {
//         let xpc_type: xpc_type_t = &value;
//         XPCType(xpc_type)
//     }
// }
// pub static ref Dictionary: XPCType = unsafe { _xpc_type_dictionary.into() };

lazy_static! {
    pub static ref Dictionary: XPCType = unsafe {
        let empty: HashMap<&str, XPCObject> = HashMap::new();
        XPCObject::from(empty).xpc_type()
    };
    pub static ref Int64: XPCType = XPCObject::from(0 as i64).into();
    pub static ref UInt64: XPCType = XPCObject::from(0 as u64).into();
    pub static ref String: XPCType = XPCObject::from("").into();
    pub static ref Bool: XPCType = XPCObject::from(true).into();
    pub static ref MachPort: XPCType = XPCObject::from(0 as mach_port_t).into();
}
