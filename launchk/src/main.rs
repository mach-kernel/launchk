#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::ptr::null_mut;
use xpc_sys;
use xpc_sys::*;

use std::convert::TryInto;
use xpc_sys::objects::xpc_dictionary::{XPCDictionary, XPCDictionaryError};
use xpc_sys::objects::xpc_object::XPCObject;

use crate::tui::list_services;
use xpc_sys::traits::xpc_pipeable::{XPCPipeable, XPCPipeError};
use crate::launchd::messages::from_msg;

mod tui;
mod state;
mod launchd;

fn main() {
    let mut message: HashMap<&str, XPCObject> = from_msg(&launchd::messages::LIST_SERVICES);

    let xpc_object: XPCObject = message.into();
    let mut siv = cursive::default();

    let services = xpc_object.pipe_routine()
        .ok()
        .and_then(|reply| reply.try_into().ok())
        .and_then(|XPCDictionary(hm)| hm.get("services").map(|s| s.clone()))
        .and_then(|svcs| svcs.try_into().ok())
        .and_then(|XPCDictionary(hm)| Some(hm));

    list_services(&mut siv, &services.unwrap());
    siv.run();
}
