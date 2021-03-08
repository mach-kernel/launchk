use crate::objects::xpc_type::XPCType;
use crate::{
    mach_port_t, xpc_bool_create, xpc_copy_description, xpc_double_create, xpc_int64_create,
    xpc_mach_send_create, xpc_object_t, xpc_release, xpc_string_create, xpc_uint64_create,
};
use std::ffi::{CStr, CString};
use std::ptr::null_mut;
use std::sync::Arc;

use std::fmt;

#[derive(Clone, PartialEq, Eq)]
pub struct XPCObject(pub Arc<xpc_object_t>, pub XPCType);

unsafe impl Send for XPCObject {}
unsafe impl Sync for XPCObject {}

impl XPCObject {
    pub fn new(value: xpc_object_t) -> Self {
        value.into()
    }

    pub fn xpc_type(&self) -> XPCType {
        let XPCObject(_, xpc_type) = self;
        *xpc_type
    }

    pub fn as_ptr(&self) -> xpc_object_t {
        let XPCObject(object_ptr, _) = self;
        **object_ptr
    }
}

impl Default for XPCObject {
    fn default() -> Self {
        Self(Arc::new(null_mut()), XPCType(null_mut()))
    }
}

impl fmt::Display for XPCObject {
    /// Use xpc_copy_description to get an easy snapshot of a dictionary
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let XPCObject(arc, _) = self;

        if **arc == null_mut() {
            write!(f, "XPCObject is NULL")
        } else {
            let xpc_desc = unsafe { xpc_copy_description(**arc) };
            let cstr = unsafe { CStr::from_ptr(xpc_desc) };
            write!(f, "{}", cstr.to_string_lossy())
        }
    }
}

impl From<xpc_object_t> for XPCObject {
    fn from(value: xpc_object_t) -> Self {
        XPCObject(Arc::new(value), value.into())
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

impl From<f64> for XPCObject {
    /// Create XPCObject via xpc_double_create
    fn from(value: f64) -> Self {
        unsafe { XPCObject::new(xpc_double_create(value)) }
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

/// Cloning an XPC object will clone the underlying Arc -- we will
/// call xpc_release() only if we are the last valid reference
/// (and underlying data is not null)
///
/// NOTE: If using with obj-c blocks crate (blocks::ConcreteBlock),
/// make sure to invoke xpc_retain() to avoid the Obj-C runtime from
/// releasing your xpc_object_t after the block leaves scope. This
/// drop trait will then cause a segfault!
///
/// TODO: Is there a way to check if an xpc_release() was already invoked?
impl Drop for XPCObject {
    fn drop(&mut self) {
        let XPCObject(arc, _) = self;
        if **arc == null_mut() {
            return;
        }

        let refs = Arc::strong_count(arc);

        if refs <= 1 {
            unsafe { xpc_release(**arc) }
        }
    }
}
