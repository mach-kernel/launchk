#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

#[macro_use]
extern crate lazy_static;

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_long, c_void};
use std::ptr::null_mut;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

//
pub mod object;
//

pub type xpc_pipe_t = *mut c_void;

/// Some extra private API definitions. Thanks:
///
/// https://developer.apple.com/documentation/kernel/mach
/// https://chromium.googlesource.com/chromium/src.git/+/47.0.2507.2/sandbox/mac/xpc_private_stubs.sig
extern "C" {
    // Seems to give same output as strerror ¯\_(ツ)_/¯
    pub fn xpc_strerror(err: c_int) -> *const c_char;

    pub static errno: c_int;

    pub fn xpc_pipe_create_from_port(port: mach_port_t, flags: u64) -> xpc_pipe_t;
    pub fn xpc_pipe_routine_with_flags(
        pipe: xpc_pipe_t,
        msg: xpc_object_t,
        reply: *mut xpc_object_t,
        flags: u64,
    ) -> c_int;
    pub fn xpc_pipe_routine(pipe: xpc_pipe_t, msg: xpc_object_t, reply: *mut xpc_object_t)
        -> c_int;

    pub fn xpc_mach_send_create(port: mach_port_t) -> xpc_object_t;
    pub fn xpc_dictionary_set_mach_send(
        object: xpc_object_t,
        name: *const c_char,
        port: mach_port_t,
    );

    // https://opensource.apple.com/source/Libsystem/Libsystem-1213/alloc_once_private.h.auto.html
    pub static _os_alloc_once_table: [_os_alloc_once_s; 10];
}

#[repr(C)]
pub struct _os_alloc_once_s {
    pub once: c_long,
    pub ptr: *mut c_void,
}

#[repr(C)]
pub struct xpc_global_data {
    pub a: u_int64_t,
    pub xpc_flags: u_int64_t,
    pub task_bootstrap_port: mach_port_t,
    pub xpc_bootstrap_pipe: xpc_pipe_t,
}

/// Look up bootstrap port for mach_task_self
pub fn lookup_bootstrap_port() -> mach_port_t {
    let mut num_ports: mach_msg_type_number_t = 0;
    let mut found_ports: *mut mach_port_t = null_mut();

    let kr: kern_return_t =
        unsafe { mach_ports_lookup(mach_task_self_, &mut found_ports, &mut num_ports) };

    if kr != KERN_SUCCESS as i32 {
        panic!("Unable to obtain Mach bootstrap port!");
    }

    let ret_port: mach_port_t = unsafe { *found_ports.offset(0) };
    println!(
        "{} ports for mach_task_self_, taking first: mach_port_t {}",
        num_ports, ret_port
    );

    // Deallocate others
    unsafe {
        for i in 1..num_ports {
            let port = *found_ports.offset(i as isize);
            println!("Deallocating mach_port_t {}", port);
            mach_port_deallocate(mach_task_self_, port);
        }
    }

    ret_port
}

/// Attempt to yield existing bootstrap_port if not
/// MACH_PORT_NULL
pub fn get_bootstrap_port() -> mach_port_t {
    unsafe {
        if bootstrap_port == MACH_PORT_NULL {
            println!("Bootstrap port is null! Querying for port");
            lookup_bootstrap_port()
        } else {
            println!("Found bootstrap port {}", bootstrap_port);
            bootstrap_port
        }
    }
}

/// Get xpc global data bootstrap pipe or find bootstrap port + create new pipe
pub fn get_xpc_bootstrap_pipe() -> xpc_pipe_t {
    match read_xpc_global_data() {
        Some(xpcgd) => {
            unsafe {
                println!(
                    "Found _os_alloc_once_table: {:?}",
                    &_os_alloc_once_table as *const _
                );
                println!("Found xpc_bootstrap_pipe: {:?}", xpcgd.xpc_bootstrap_pipe);
            }
            xpcgd.xpc_bootstrap_pipe
        }
        None => unsafe {
            println!("Can't find _os_alloc_once_table, creating new bootstrap pipe");
            xpc_pipe_create_from_port(get_bootstrap_port(), 0)
        },
    }
}

pub fn read_xpc_global_data() -> Option<&'static xpc_global_data> {
    let gd: *mut xpc_global_data = unsafe { _os_alloc_once_table[1].ptr as *mut _ };
    unsafe { gd.as_ref() }
}

pub fn print_errno(err: Option<i32>) {
    unsafe {
        let unwrapped = err.unwrap_or(errno);
        let error = CStr::from_ptr(strerror(unwrapped));
        println!("Error {}: {}", unwrapped, error.to_str().unwrap());
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
