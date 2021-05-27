use crate::{
    _xpc_type_array, _xpc_type_bool, _xpc_type_dictionary, _xpc_type_double, _xpc_type_int64,
    _xpc_type_s, _xpc_type_string, _xpc_type_uint64, xpc_get_type, xpc_object_t, xpc_type_get_name,
    xpc_type_t, _xpc_type_mach_send, _xpc_type_mach_recv,
};

use crate::objects::xpc_error::XPCError;
use crate::objects::xpc_error::XPCError::ValueError;
use crate::objects::xpc_object::XPCObject;
use std::ffi::CStr;

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
   All (most?) _xpc_type_* from bindings.rs

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
    pub static ref Dictionary: XPCType =
        unsafe { (&_xpc_type_dictionary as *const _xpc_type_s).into() };
    pub static ref Int64: XPCType = unsafe { (&_xpc_type_int64 as *const _xpc_type_s).into() };
    pub static ref UInt64: XPCType = unsafe { (&_xpc_type_uint64 as *const _xpc_type_s).into() };
    pub static ref Double: XPCType = unsafe { (&_xpc_type_double as *const _xpc_type_s).into() };
    pub static ref String: XPCType = unsafe { (&_xpc_type_string as *const _xpc_type_s).into() };
    pub static ref Bool: XPCType = unsafe { (&_xpc_type_bool as *const _xpc_type_s).into() };
    pub static ref Array: XPCType = unsafe { (&_xpc_type_array as *const _xpc_type_s).into() };
    pub static ref MachSend: XPCType = unsafe { (&_xpc_type_mach_send as *const _xpc_type_s).into() };
    pub static ref MachRecv: XPCType = unsafe { (&_xpc_type_mach_recv as *const _xpc_type_s).into() };
}

/// Runtime type check for XPC object. I do not know if possible/advantageous to represent
/// HashMap<&str, XPCObject<T>> if T were heterogeneous? Box<dyn XPCObject>?
pub fn check_xpc_type(object: &XPCObject, xpc_type: &XPCType) -> Result<(), XPCError> {
    let XPCObject(_, obj_type) = object;
    if *obj_type == *xpc_type {
        return Ok(());
    }

    let obj_str = unsafe {
        let XPCType(ptr) = object.xpc_type();
        CStr::from_ptr(xpc_type_get_name(ptr))
            .to_string_lossy()
            .to_string()
    };

    let req_str = unsafe {
        let XPCType(ptr) = xpc_type;
        CStr::from_ptr(xpc_type_get_name(*ptr))
            .to_string_lossy()
            .to_string()
    };

    Err(ValueError(format!("Cannot get {} as {}", obj_str, req_str)))
}
