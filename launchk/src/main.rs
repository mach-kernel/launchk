use std::collections::HashMap;
use actix::prelude::*;
use std::ptr::null_mut;
use xpc_sys;
use xpc_sys::*;

mod actor;
use actor::xpc::{XPCActor, XPCRequest};


fn main() {
    // "launchctl list com.apple.Spotlight"
    let mut message: HashMap<&str, XPCObject> = HashMap::new();
    message.insert("type", XPCObject::from(1 as u64));
    message.insert("handle", XPCObject::from(0 as u64));
    message.insert("subsystem", XPCObject::from(3 as u64));
    message.insert("routine", XPCObject::from(815 as u64));
    message.insert("legacy", XPCObject::from(true));
    message.insert("name", XPCObject::from("com.apple.Spotlight"));
    message.insert(
        "domain-port",
        XPCObject::from(get_bootstrap_port() as mach_port_t),
    );

    let mut system = actix::System::new("launchk");
    system.block_on(async {
        let addr = XPCActor::start_default();
        let res = addr.send(XPCRequest::PipeRoutine(XPCObject::from(message).data)).await;

        match res {
            Ok(Ok(response)) => println!("Recv {}", response),
            _ => println!("Error"),
        }
    });
    system.run();

    // let mut siv = cursive::default();
    // Creates a dialog with a single "Quit" button
    // siv.add_layer(Dialog::around(TextView::new(recv))
    //     .title("test")
    //     .button("Quit", |s| s.quit()));

    // Starts the event loop.
    // siv.run();
}
