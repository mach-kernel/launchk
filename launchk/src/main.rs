#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cursive;

use std::collections::HashMap;
use std::ptr::null_mut;
use xpc_sys;
use xpc_sys::*;

use std::convert::TryInto;
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_object::XPCObject;

use crate::launchd::messages::from_msg;
use crate::tui::list_services;
use tokio::runtime::Handle;
use xpc_sys::traits::xpc_pipeable::XPCPipeable;

mod launchd;
mod tui;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut siv = cursive::default();
    list_services(&mut siv, runtime.handle().clone());
    runtime.block_on(async { siv.run() });
    // siv.run();
}
