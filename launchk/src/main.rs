use cursive::views::{Dialog, TextView};

use xpc_bindgen;
use xpc_bindgen::{xpc_connection_create_mach_service, xpc_connection_t, XPC_CONNECTION_MACH_SERVICE_PRIVILEGED, dispatch_queue_create, xpc_connection_set_event_handler, xpc_object_t, xpc_copy_description, xpc_int64_create, xpc_dictionary_create, xpc_connection_send_message, xpc_string_create, xpc_connection_resume, XPC_CONNECTION_MACH_SERVICE_LISTENER, xpc_bool_create, xpc_connection_create, bootstrap_port, mach_port_t};
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void, c_int};
use std::ptr::null_mut;
use block::ConcreteBlock;
use std::ops::Deref;
use cursive::view::AnyView;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::mem;

use std::sync::mpsc;
use futures::channel::mpsc::unbounded;
use futures::SinkExt;
use std::thread::sleep;

static APP_ID: &str = "com.dstancu.launchk";

// xpc_pipe_routine_with_flags
// https://chromium.googlesource.com/chromium/src.git/+/47.0.2507.2/sandbox/mac/xpc_private_stubs.sig
extern "C" {
    pub fn xpc_pipe_create_from_port(port: mach_port_t, flags: u64) -> *mut c_void;
    pub fn xpc_pipe_routine_with_flags(pipe: *mut c_void, msg: xpc_object_t, response: *mut xpc_object_t, flags: u64) -> c_int;
    pub fn xpc_dictionary_set_mach_send(object: xpc_object_t, name: *const c_char, port: mach_port_t);

    // TODO: bindgen
    static errno: c_int;
    pub fn strerror(errnum: c_int) -> *const c_char;
}

fn xpc_i64(value: i64) -> Rc<xpc_object_t> {
    unsafe {
        let new_obj = xpc_int64_create(value);
        let rc = Rc::new(new_obj);
        rc
        // let cloned = rc.clone();
        // cloned.to_owned()
    }
}

fn xpc_str(value: &str) -> Rc<xpc_object_t> {
    unsafe {
        let new_obj = xpc_string_create(CString::new(value).unwrap().into_boxed_c_str().as_ptr());
        let rc = Rc::new(new_obj);
        rc
        // let cloned = rc.clone();
        // cloned.to_owned()
    }
}

fn xpc_bool(value: bool) -> Rc<xpc_object_t> {
    unsafe {
        let new_obj = xpc_bool_create(value);
        let rc = Rc::new(new_obj);
        rc
        // let cloned = rc.clone();
        // cloned.to_owned()
    }
}

fn xpc_dict(message: HashMap<String, xpc_object_t>) -> xpc_object_t {
    let mut xpc_dict_keys = Vec::new();
    let mut xpc_dict_values = Vec::new();

    for (k, v) in &message {
        unsafe {
            let desc = CString::from_raw(xpc_copy_description(v.to_owned()));
            println!("Adding {} {}", k, desc.to_string_lossy());
        }

        let key = CString::new(k.deref()).unwrap();
        unsafe { xpc_dict_keys.push(key.as_ptr()) };
        xpc_dict_values.push(v.to_owned());
        mem::forget(key);
    }

    unsafe {
        xpc_dictionary_create(
            xpc_dict_keys.into_boxed_slice().as_mut_ptr(),
            xpc_dict_values.into_boxed_slice().as_mut_ptr(),
            message.len() as u64
        )
    }
}

fn print_errno() {
    unsafe {
        let error = CStr::from_ptr(strerror(errno));
        println!("Error: {}", error.to_str().unwrap());
    }
}

fn main() {
    let bootstrap_pipe = unsafe {
        xpc_pipe_create_from_port(bootstrap_port, 0)
    };

    print_errno();

    let mut message: HashMap<String, xpc_object_t> = HashMap::new();

    message.insert("type".to_string(), *xpc_i64(7));
    message.insert("handle".to_string(), *xpc_i64(0));
    message.insert("subsystem".to_string(), *xpc_i64(3));
    message.insert("routine".to_string(), *xpc_i64(815));
    message.insert("legacy".to_string(), *xpc_bool(true));
    message.insert("name".to_string(), *xpc_str("com.apple.Spotlight"));

    let msg_dict = xpc_dict(message);

    unsafe {
        let name = CString::new("domain-port").unwrap();
        xpc_dictionary_set_mach_send(msg_dict, name.as_ptr(), bootstrap_port);

        let desc = CString::from_raw(xpc_copy_description(msg_dict));
        println!("Sending {}", desc.to_string_lossy());
    }

    unsafe {
        let mut response: xpc_object_t = null_mut();
        let xpcwr_err = xpc_pipe_routine_with_flags(bootstrap_pipe, msg_dict, &mut response, 0);
        let error = CStr::from_ptr(strerror(xpcwr_err));
        println!("sent pipe with {}", error.to_str().unwrap());
    };

    // let mut siv = cursive::default();
    //
    // // Creates a dialog with a single "Quit" button
    // siv.add_layer(Dialog::around(TextView::new("Hello Dialog!"))
    //                      .title("Cursive")
    //                      .button("Quit", |s| s.quit()));
    //
    // // Starts the event loop.
    // siv.run();
}
