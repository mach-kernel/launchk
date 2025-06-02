use crate::{
    _xpc_type_array, _xpc_type_bool, _xpc_type_dictionary, _xpc_type_double, _xpc_type_fd,
    _xpc_type_int64, _xpc_type_mach_recv, _xpc_type_mach_send, _xpc_type_null, _xpc_type_s,
    _xpc_type_shmem, _xpc_type_string, _xpc_type_uint64, xpc_get_type, xpc_object_t,
    xpc_type_get_name, xpc_type_t,
};

use crate::object::xpc_error::XPCError;
use crate::object::xpc_error::XPCError::ValueError;
use crate::object::xpc_object::XPCObject;
use std::ffi::CStr;
use std::fmt;
use std::ptr::null;

#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct XPCType(pub xpc_type_t);

impl XPCType {
    pub fn as_ptr(&self) -> xpc_type_t {
        let XPCType(t) = self;
        *t
    }
}

unsafe impl Send for XPCType {}
unsafe impl Sync for XPCType {}

impl From<xpc_object_t> for XPCType {
    fn from(value: xpc_object_t) -> XPCType {
        XPCType(unsafe { xpc_get_type(value) })
    }
}

impl From<XPCObject> for XPCType {
    fn from(value: XPCObject) -> XPCType {
        value.xpc_type()
    }
}

impl From<*const _xpc_type_s> for XPCType {
    fn from(value: *const _xpc_type_s) -> Self {
        XPCType(value)
    }
}

impl fmt::Display for XPCType {
    /// Use xpc_copy_description to show as a string, for
    /// _xpc_type_dictionary contents are shown!
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let XPCType(t) = self;

        if *t == null() {
            return write!(f, "NULL");
        }

        write!(f, "{}", unsafe {
            CStr::from_ptr(xpc_type_get_name(*t)).to_string_lossy()
        })
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
    pub static ref MachSend: XPCType =
        unsafe { (&_xpc_type_mach_send as *const _xpc_type_s).into() };
    pub static ref MachRecv: XPCType =
        unsafe { (&_xpc_type_mach_recv as *const _xpc_type_s).into() };
    pub static ref Fd: XPCType = unsafe { (&_xpc_type_fd as *const _xpc_type_s).into() };
    pub static ref Shmem: XPCType = unsafe { (&_xpc_type_shmem as *const _xpc_type_s).into() };
    pub static ref Null: XPCType = unsafe { (&_xpc_type_null as *const _xpc_type_s).into() };
}

/// Runtime type check for XPC object.
pub fn check_xpc_type(object: &XPCObject, requested: &XPCType) -> Result<(), XPCError> {
    if object.xpc_type() == *requested {
        return Ok(());
    }

    Err(ValueError(format!(
        "Cannot get {} as {}",
        object.xpc_type(),
        requested
    )))
}
