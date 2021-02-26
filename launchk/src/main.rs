use std::collections::HashMap;
use std::ptr::null_mut;
use xpc_sys;
use xpc_sys::*;

use cursive::direction::Orientation;
use cursive::views::Panel;
use cursive::{Printer, View};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::Arc;
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

    let response =
        unsafe { xpc_pipe_routine(bootstrap_pipe, XPCObject::from(message).0, &mut reply) };

    if response != 0 {
        panic!("XPC query failed!")
    }

    let lift = XPCObject(reply);
    println!("Response {}", lift);

    // let siv = cursive::default();
}
