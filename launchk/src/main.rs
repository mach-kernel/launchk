use cursive::views::{Dialog, TextView};

use xpc_sys;
use xpc_sys::*;
use std::ffi::{CString, CStr};
use std::os::raw::{c_char, c_void, c_int};
use std::ptr::null_mut;
use std::ops::Deref;
use cursive::view::AnyView;
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::mem;

use std::sync::{mpsc, Arc};
use std::thread::sleep;

fn main() {
    let res_bootstrap_port = get_bootstrap_port();

    // "launchctl list com.apple.Spotlight"
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
        xpc_dictionary_set_mach_send(msg_dict.ptr, name.as_ptr(), res_bootstrap_port);

        let desc = CString::from_raw(xpc_copy_description(msg_dict.ptr));
        println!("Assembled message {}", desc.to_string_lossy());
    }

    let pipe = get_xpc_bootstrap_pipe();
    unsafe {
        let mut response: xpc_object_t = null_mut();
        let pipe_err = xpc_pipe_routine_with_flags(pipe, msg_dict.ptr, &mut response, 0);
        print_errno(Some(pipe_err));
        let desc = CString::from_raw(xpc_copy_description(response));
        println!("Recv {}", desc.to_string_lossy());
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
