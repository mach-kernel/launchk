use std::collections::HashMap;
use std::ptr::null_mut;
use xpc_sys;
use xpc_sys::*;

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

    let msg_dict = XPCObject::from(message);
    println!("Sending {}", msg_dict);

    let pipe = get_xpc_bootstrap_pipe();
    let response = unsafe {
        let mut response: xpc_object_t = null_mut();
        let err = xpc_pipe_routine_with_flags(pipe, msg_dict.data, &mut response, 0);

        if err != 0 {
            print_errno(Some(err));
            panic!("Could not send!")
        }

        response
    };

    println!("Received {}", XPCObject::from(response));

    // let mut siv = cursive::default();
    // Creates a dialog with a single "Quit" button
    // siv.add_layer(Dialog::around(TextView::new(recv))
    //     .title("test")
    //     .button("Quit", |s| s.quit()));

    // Starts the event loop.
    // siv.run();
}
