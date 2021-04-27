use std::collections::HashMap;

use crate::{mach_port_t, xpc_get_type, xpc_object_t, xpc_type_t, _xpc_type_array, xpc_array_create, _xpc_type_s, _xpc_type_int64, _xpc_type_string, _xpc_type_uint64, _xpc_type_double, _xpc_type_bool, _xpc_type_dictionary};

use crate::objects::xpc_object::XPCObject;
use std::ptr::null_mut;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

impl From<*const _xpc_type_s> for XPCType {
    fn from(value: *const _xpc_type_s) -> Self {
        XPCType(value)
    }
}

/*
   All _xpc_type_* from bindings.rs

   pub static _xpc_bool_true: _xpc_bool_s;
   pub static _xpc_bool_false: _xpc_bool_s;
   pub static _xpc_type_int64: _xpc_type_s;
   pub static _xpc_type_uint64: _xpc_type_s;
   pub static _xpc_type_double: _xpc_type_s;
   pub static _xpc_type_date: _xpc_type_s;
   pub static _xpc_type_data: _xpc_type_s;
   pub static _xpc_type_string: _xpc_type_s;
   pub static _xpc_type_uuid: _xpc_type_s;
   pub static _xpc_type_fd: _xpc_type_s;
   pub static _xpc_type_shmem: _xpc_type_s;
   pub static _xpc_type_array: _xpc_type_s;
   pub static _xpc_type_dictionary: _xpc_type_s;
   pub static _xpc_type_error: _xpc_type_s;
*/

lazy_static! {
    pub static ref Dictionary: XPCType = unsafe { (&_xpc_type_dictionary as *const _xpc_type_s).into() };
    pub static ref Int64: XPCType = unsafe { (&_xpc_type_int64 as *const _xpc_type_s).into() };
    pub static ref UInt64: XPCType = unsafe { (&_xpc_type_uint64 as *const _xpc_type_s).into() };
    pub static ref Double: XPCType = unsafe { (&_xpc_type_double as *const _xpc_type_s).into() };
    pub static ref String: XPCType = unsafe { (&_xpc_type_string as *const _xpc_type_s).into() };
    pub static ref Bool: XPCType = unsafe { (&_xpc_type_bool as *const _xpc_type_s).into() };
    pub static ref Array: XPCType = unsafe { (&_xpc_type_array as *const _xpc_type_s).into() };
}
