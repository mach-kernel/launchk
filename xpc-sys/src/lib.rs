#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::os::raw::{c_void, c_char, c_int};
use std::sync::Arc;
use std::ffi::CString;
use std::collections::HashMap;
use std::mem;
use std::ops::Deref;
use std::iter::FromIterator;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/**
 * Some extra private API definitions. Thanks:
 * https://chromium.googlesource.com/chromium/src.git/+/47.0.2507.2/sandbox/mac/xpc_private_stubs.sig
 */
extern "C" {
    pub fn xpc_pipe_create_from_port(port: mach_port_t, flags: u64) -> *mut c_void;
    pub fn xpc_pipe_routine_with_flags(pipe: *mut c_void, msg: xpc_object_t, response: *mut xpc_object_t, flags: u64) -> c_int;
    pub fn xpc_dictionary_set_mach_send(object: xpc_object_t, name: *const c_char, port: mach_port_t);
}

/// newtype for xpc_object_t
pub struct XPCObject {
    pub ptr: xpc_object_t,
}

impl From<i64> for XPCObject {
    /// Create XPCObject via xpc_int64_create
    fn from(value: i64) -> Self {
        unsafe {
            return XPCObject { ptr: xpc_int64_create(value) };
        }
    }
}

impl From<bool> for XPCObject {
    /// Create XPCObject via xpc_bool_create
    fn from(value: bool) -> Self {
        unsafe {
            return XPCObject { ptr: xpc_bool_create(value) };
        }
    }
}

impl From<&str> for XPCObject {
    /// Create XPCObject via xpc_string_create
    fn from(slice: &str) -> Self {
        let cstr = CString::new(slice).unwrap();
        unsafe {
            return XPCObject {
                ptr: xpc_string_create(cstr.into_boxed_c_str().as_ptr())
            }
        }
    }
}

impl From<HashMap<String, XPCObject>> for XPCObject {
    /// Create an XPC dictionary from HashMap<String, XPCObject>
    fn from(message: HashMap<String, XPCObject>) -> Self {
        // Must have valid references to the CStrings else they get dropped
        // before we can load them using xpc_dictionary_create
        let cstr_keys = Vec::from_iter(
            message.keys().flat_map(|k| CString::new(k.to_string()))
        );
        let cstr_ptrs = Vec::from_iter(cstr_keys.iter().map(|cs| cs.as_ptr()));
        let mut obj_values = Vec::from_iter(message.values().map(|xo| xo.ptr));

        unsafe {
            return XPCObject {
                ptr: xpc_dictionary_create(
                    cstr_ptrs.as_ptr(),
                    obj_values.as_mut_ptr(),
                    message.len() as u64
                )
            };
        }
    }
}

/// Look up bootstrap port for mach_task_self
pub fn get_bootstrap_port() -> mach_port_t {
    let mut num_ports: mach_msg_type_number_t = 0;
    let mut ret_ports: *mut mach_port_t = null_mut();

    unsafe {
        mach_ports_lookup(mach_task_self_, &mut ret_ports as *mut _, &mut num_ports);
    }

    println!("Found {} ports for mach_task_self_", num_ports);

    unsafe {
        println!("Returning mach_port_t {}", *ret_ports);
        return *ret_ports;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
