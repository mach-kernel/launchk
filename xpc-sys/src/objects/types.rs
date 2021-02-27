use crate::{
    mach_port_t, xpc_bool_create, xpc_copy_description, xpc_dictionary_create, xpc_get_type,
    xpc_int64_create, xpc_mach_send_create, xpc_object_t, xpc_pipe_t, xpc_release,
    xpc_string_create, xpc_type_t, xpc_uint64_create,
};
use std::ffi::{CStr, CString};
use std::fmt;
use std::ptr::{null, null_mut};

use std::sync::Arc;

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

#[derive(Clone, PartialEq, Eq)]
/// Newtype for xpc_object_t
pub struct XPCObject(pub Arc<xpc_object_t>);

unsafe impl Send for XPCObject {}
unsafe impl Sync for XPCObject {}

#[repr(transparent)]
#[derive(Clone, PartialEq, Eq)]
/// Newtype for xpc_pipe_t
pub struct XPCPipe(pub xpc_pipe_t);

unsafe impl Send for XPCPipe {}
unsafe impl Sync for XPCPipe {}

impl XPCObject {
    pub fn new(value: xpc_object_t) -> Self {
        value.into()
    }

    pub fn get_type(&self) -> XPCType {
        let XPCObject(object_ptr) = self;
        unsafe { XPCType(xpc_get_type(**object_ptr)) }
    }

    pub fn as_ptr(&self) -> xpc_object_t {
        let XPCObject(object_ptr) = self;
        **object_ptr
    }
}

impl fmt::Display for XPCObject {
    /// Use xpc_copy_description to get an easy snapshot of a dictionary
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let XPCObject(arc) = self;

        if **arc == null_mut() {
            write!(f, "XPCObject is NULL")
        } else {
            let xpc_desc = unsafe { xpc_copy_description(**arc) };
            let cstr = unsafe { CStr::from_ptr(xpc_desc) };
            write!(f, "{}", cstr.to_string_lossy())
        }
    }
}

impl From<i64> for XPCObject {
    /// Create XPCObject via xpc_int64_create
    fn from(value: i64) -> Self {
        unsafe { XPCObject::new(xpc_int64_create(value)) }
    }
}

impl From<u64> for XPCObject {
    /// Create XPCObject via xpc_uint64_create
    fn from(value: u64) -> Self {
        unsafe { XPCObject::new(xpc_uint64_create(value)) }
    }
}

impl From<mach_port_t> for XPCObject {
    /// Create XPCObject via xpc_uint64_create
    fn from(value: mach_port_t) -> Self {
        unsafe { XPCObject::new(xpc_mach_send_create(value)) }
    }
}

impl From<bool> for XPCObject {
    /// Create XPCObject via xpc_bool_create
    fn from(value: bool) -> Self {
        unsafe { XPCObject::new(xpc_bool_create(value)) }
    }
}

impl From<&str> for XPCObject {
    /// Create XPCObject via xpc_string_create
    fn from(slice: &str) -> Self {
        let cstr = CString::new(slice).unwrap();
        unsafe { XPCObject::new(xpc_string_create(cstr.into_boxed_c_str().as_ptr())) }
    }
}

impl From<xpc_object_t> for XPCObject {
    fn from(value: xpc_object_t) -> Self {
        XPCObject(Arc::new(value))
    }
}

/// TODO: If there is more than one XPCObject with the same pointer
/// value and it is dropped -- then that pointer gets released.
impl Drop for XPCObject {
    fn drop(&mut self) {
        let XPCObject(arc) = self;
        if **arc == null_mut() {
            println!("I'm null!");
            return;
        }

        let refs = Arc::strong_count(arc);
        println!("{} I have {} refs", **arc as u64, refs);
        if refs <= 1 {
            unsafe { xpc_release(**arc) }
        }
    }
}
