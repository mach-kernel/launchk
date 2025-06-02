use libc::c_int;
use std::collections::HashMap;

use crate::object::xpc_type::XPCType;
use crate::{xpc_array_append_value, xpc_array_create, xpc_bool_create, xpc_copy, xpc_copy_description, xpc_dictionary_create, xpc_dictionary_set_value, xpc_double_create, xpc_fd_create, xpc_int64_create, xpc_mach_recv_create, xpc_mach_send_create, xpc_object_t, xpc_release, xpc_retain, xpc_string_create, xpc_uint64_create};
use libc::mach_port_t;
use std::ffi::{CStr, CString};
use std::os::unix::prelude::RawFd;
use std::ptr::{null, null_mut};

use crate::object::xpc_type;
use crate::object::xpc_type::check_xpc_type;
use std::fmt;
use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub struct XPCObject(xpc_object_t, pub XPCType);

unsafe impl Send for XPCObject {}
unsafe impl Sync for XPCObject {}

pub type XPCHashMap = HashMap<String, Arc<XPCObject>>;

impl XPCObject {
    pub unsafe fn from_raw(value: xpc_object_t) -> XPCObject {
        Self::new(value)
    }

    pub unsafe fn from_raw_retain(value: xpc_object_t) -> XPCObject {
        Self::new(xpc_retain(value))
    }

    fn new(value: xpc_object_t) -> Self {
        let obj = Self(value, value.into());

        log::info!(
            "XPCObject new ({:p}, {}, {})",
            value,
            obj.xpc_type(),
            obj.get_refs()
                .map(|(r, xr)| format!("refs {} xrefs {}", r, xr))
                .unwrap_or("refs ???".to_string()),
        );

        obj
    }

    pub fn xpc_type(&self) -> XPCType {
        let XPCObject(_, xpc_type) = self;
        *xpc_type
    }

    /// Get underlying xpc_object_t pointer
    pub fn as_ptr(&self) -> xpc_object_t {
        let XPCObject(object_ptr, _) = self;
        *object_ptr
    }

    /// Should (?) return a deep copy of the underlying object
    /// https://developer.apple.com/documentation/xpc/1505584-xpc_copy
    pub fn xpc_copy(xpc_object: xpc_object_t) -> Self {
        let clone = unsafe { xpc_copy(xpc_object) };
        Self::new(clone)
    }

    /// Attempt to safely get refcounts (segfault for others?)
    fn get_refs(&self) -> Option<(c_int, c_int)> {
        for t in &[*xpc_type::UInt64, *xpc_type::Int64, *xpc_type::Bool] {
            check_xpc_type(self, t).err()?;
        }

        unsafe {
            Some((
                Self::read_refs(self.as_ptr()),
                Self::read_xrefs(self.as_ptr()),
            ))
        }
    }

    /// Read ref count (base + 0x0C). The count is incremented and
    /// decremented with calls to xpc_release and xpc_retain.
    unsafe fn read_refs(xpc_object: xpc_object_t) -> c_int {
        let refs: *const c_int = xpc_object as *const _;
        *refs.offset(3)
    }

    /// Read xref count (base + 0x08). The count is incremented and
    /// decremented with calls to xpc_release and xpc_retain.
    unsafe fn read_xrefs(xpc_object: xpc_object_t) -> c_int {
        let xrefs: *const c_int = xpc_object as *const _;
        *xrefs.offset(2)
    }
}

impl fmt::Display for XPCObject {
    /// Use xpc_copy_description to show as a string, for
    /// _xpc_type_dictionary contents are shown!
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let XPCObject(ptr, _) = self;

        if *ptr == null_mut() {
            write!(f, "{:?} xpc_object_t is NULL", self)
        } else {
            let xpc_desc = unsafe { xpc_copy_description(*ptr) };
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

impl From<f64> for XPCObject {
    /// Create XPCObject via xpc_double_create
    fn from(value: f64) -> Self {
        unsafe { XPCObject::new(xpc_double_create(value)) }
    }
}

/// Enum used for selecting between _xpc_type_mach_send and _xpc_type_mach_recv
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MachPortType {
    Send,
    Recv,
}

impl From<(MachPortType, mach_port_t)> for XPCObject {
    /// Create XPCObject via xpc_mach_send_create or xpc_mach_recv_create
    fn from((mpt, value): (MachPortType, mach_port_t)) -> Self {
        let xpc_object = unsafe {
            match mpt {
                MachPortType::Send => xpc_mach_send_create(value),
                MachPortType::Recv => xpc_mach_recv_create(value),
            }
        };

        XPCObject::new(xpc_object)
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
        unsafe { XPCObject::new(xpc_string_create(cstr.as_ptr())) }
    }
}

impl<O: Into<XPCObject>> From<Vec<O>> for XPCObject {
    /// Create XPCObject via xpc_array_create
    fn from(value: Vec<O>) -> Self {
        let xpc_array = unsafe { xpc_array_create(null_mut(), 0) };
        for object in value {
            unsafe { xpc_array_append_value(xpc_array, object.into().as_ptr()) }
        }

        XPCObject::new(xpc_array)
    }
}

impl From<String> for XPCObject {
    /// Create XPCObject via xpc_string_create
    fn from(value: String) -> Self {
        let cstr = CString::new(value).unwrap();
        unsafe { XPCObject::new(xpc_string_create(cstr.as_ptr())) }
    }
}

impl<R: AsRef<XPCObject>> From<R> for XPCObject {
    /// Use xpc_copy() to copy out of refs.
    /// https://developer.apple.com/documentation/xpc/1505584-xpc_copy?language=objc
    fn from(other: R) -> Self {
        Self::xpc_copy(other.as_ref().as_ptr())
    }
}

impl From<RawFd> for XPCObject {
    /// Box fd in an XPC object which "behaves like dup()", allowing
    /// to close after wrapping.
    fn from(value: RawFd) -> Self {
        log::info!("Making FD from {}", value);
        unsafe { XPCObject::new(xpc_fd_create(value)) }
    }
}

impl<S> From<HashMap<S, Arc<XPCObject>>> for XPCObject
where
    S: Into<String>,
{
    /// Creates a XPC dictionary
    ///
    /// Values must be Arc<XPCObject> but can encapsulate any
    /// valid xpc_object_t
    fn from(message: HashMap<S, Arc<XPCObject>>) -> Self {
        let dict = unsafe { xpc_dictionary_create(null(), null_mut(), 0) };

        for (k, v) in message {
            unsafe {
                let as_str: String = k.into();
                let cstr = CString::new(as_str).unwrap();
                xpc_dictionary_set_value(dict, cstr.as_ptr(), v.as_ptr());
            }
        }

        unsafe { XPCObject::from_raw(dict) }
    }
}

impl Drop for XPCObject {
    /// Release XPC object when dropped
    /// https://developer.apple.com/documentation/xpc/1505851-xpc_release
    fn drop(&mut self) {
        let XPCObject(ptr, _) = &self;

        if *ptr == null_mut() {
            log::info!("XPCObject xpc_object_t is NULL, not calling xpc_release()");
            return;
        }

        log::info!(
            "XPCObject drop ({:p}, {}, {})",
            *ptr,
            &self.xpc_type(),
            &self
                .get_refs()
                .map(|(r, xr)| format!("refs {} xrefs {}", r, xr))
                .unwrap_or("refs ???".to_string()),
        );

        unsafe { xpc_release(*ptr) }
    }
}

impl Clone for XPCObject {
    fn clone(&self) -> Self {
        XPCObject::xpc_copy(self.as_ptr())
    }
}

#[cfg(test)]
mod tests {
    use libc::mach_port_t;
    use std::collections::HashMap;
    use std::ffi::{CStr, CString};
    use std::os::unix::prelude::RawFd;
    use std::sync::Arc;

    use crate::{get_bootstrap_port, xpc_dictionary_get_string};

    use super::MachPortType;
    use super::XPCObject;

    // Mostly for docs, int, uint, bool segfault here
    #[test]
    fn safely_get_refs() {
        let bootstrap_port: mach_port_t = unsafe { get_bootstrap_port() };

        for obj in &[
            XPCObject::from(5.24 as f64),
            XPCObject::from("foo"),
            XPCObject::from(1 as RawFd),
            XPCObject::from((MachPortType::Send, bootstrap_port)),
        ] {
            assert!(obj.get_refs().is_some())
        }
    }

    #[test]
    fn xpc_object_from_hashmap() {
        let mut hm: HashMap<&str, Arc<XPCObject>> = HashMap::new();
        let value = "foo";
        hm.insert("test", XPCObject::from(value).into());

        let xpc_object = XPCObject::from(hm);
        let key = CString::new("test").unwrap();

        let cstr =
            unsafe { CStr::from_ptr(xpc_dictionary_get_string(xpc_object.as_ptr(), key.as_ptr())) };

        assert_eq!(cstr.to_str().unwrap(), value);
    }
}
