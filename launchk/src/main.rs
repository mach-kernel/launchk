use std::collections::HashMap;
use std::ptr::null_mut;
use xpc_sys;
use xpc_sys::*;

use std::convert::TryInto;
use xpc_sys::object::xpc_dictionary::XPCDictionary;
use xpc_sys::object::xpc_object::XPCObject;
use cursive::views::{LinearLayout, SelectView};
use cursive::view::Scrollable;

mod tui;

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

    if let Ok(XPCDictionary(reply_dict)) = reply.try_into() {
        let mut siv = cursive::default();
        let mut sv: SelectView<XPCObject> = SelectView::new();

        let services = reply_dict.get("services").unwrap();
        if let Ok(XPCDictionary(svc_dict)) = XPCDictionary::new(services) {
            for (k, v) in svc_dict {
                sv.add_item(k, v);
            }
        }

        siv.add_layer(LinearLayout::horizontal().child(
            sv.scrollable()
        ));

        siv.run()
    }
}
