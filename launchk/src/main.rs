use std::collections::HashMap;
use std::ptr::null_mut;
use xpc_sys;
use xpc_sys::*;

use std::convert::TryInto;
use xpc_sys::objects::dictionary::{XPCDictionary, XPCDictionaryError};
use xpc_sys::objects::types::XPCObject;

// struct ServiceListView {
//     services: Vec<String>,
// }

// impl View for ServiceListView {
//     fn draw(&self, printer: &Printer) {
//     }
// }

fn main() {
    // "launchctl list com.apple.Spotlight"
    let mut message: HashMap<&str, XPCObject> = HashMap::new();
    message.insert("type", XPCObject::from(1 as u64));
    message.insert("handle", XPCObject::from(0 as u64));
    message.insert("subsystem", XPCObject::from(3 as u64));
    message.insert("routine", XPCObject::from(815 as u64));
    message.insert("legacy", XPCObject::from(true));
    // message.insert("name", XPCObject::from("com.apple.Spotlight"));
    message.insert(
        "domain-port",
        XPCObject::from(get_bootstrap_port() as mach_port_t),
    );

    let bootstrap_pipe = get_xpc_bootstrap_pipe();
    let mut reply: xpc_object_t = null_mut();

    let send = unsafe {
        xpc_pipe_routine(
            bootstrap_pipe,
            XPCObject::from(message).as_ptr(),
            &mut reply,
        )
    };

    if send != 0 {
        panic!("XPC query failed!")
    }

    let _as_obj: Result<XPCDictionary, XPCDictionaryError> = reply.try_into();

    // let as_dict = XPCDictionary::new(XPCObject::new(reply));
    // println!("Response {}", XPCObject(reply)z);

    // let siv = cursive::default();
}
