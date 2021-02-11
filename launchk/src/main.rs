use cursive::views::{Dialog, TextView};

use xpc_sys;
use xpc_sys::*;
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

use std::sync::{mpsc, Arc};
use futures::channel::mpsc::unbounded;
use futures::SinkExt;
use std::thread::sleep;

extern "C" {
    pub fn strerror(err: c_int) -> *const c_char;
    static errno: c_int;
}

fn print_errno(err: Option<i32>) {
    let reserr = unsafe { match err {
        Some(e) => e,
        None => errno,
    } };

    unsafe {
        let error = CStr::from_ptr(strerror(reserr));
        println!("Error {}: {}", reserr, error.to_str().unwrap());
    }
}

fn main() {
    unsafe {
        println!("Does bootstrap port exist? {}", bootstrap_port != MACH_PORT_NULL);
    }

    let found_strap_port = get_bootstrap_port();
    let bootstrap_pipe = unsafe {
        xpc_pipe_create_from_port(found_strap_port, 0)
    };

    let mut message: HashMap<String, XPCObject> = HashMap::new();

    message.insert("type".to_string(), XPCObject::from(7));
    message.insert("handle".to_string(), XPCObject::from(0));
    message.insert("subsystem".to_string(), XPCObject::from(3));
    message.insert("routine".to_string(), XPCObject::from(815));
    message.insert("legacy".to_string(), XPCObject::from(true));
    message.insert("name".to_string(), XPCObject::from("com.apple.Spotlight"));

    let msg_dict = XPCObject::from(message);

    unsafe {
        let name = CString::new("domain-port").unwrap();
        xpc_dictionary_set_mach_send(msg_dict.ptr, name.as_ptr(), bootstrap_port);

        let desc = CString::from_raw(xpc_copy_description(msg_dict.ptr));
        println!("Sending {}", desc.to_string_lossy());
    }

    unsafe {
        let mut response: xpc_object_t = null_mut();
        let pipe_err = xpc_pipe_routine_with_flags(bootstrap_pipe, msg_dict.ptr, &mut response, 0);
        print_errno(Some(pipe_err));
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
