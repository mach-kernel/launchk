use std::collections::HashMap;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::{get_bootstrap_port, mach_port_t};

lazy_static! {
    /// launchctl list [name]
    pub static ref LIST_SERVICES: HashMap<&'static str, XPCObject> = {
        // "list com.apple.Spotlight" (if specified)
        // msg.insert("name", XPCObject::from("com.apple.Spotlight"));

        let mut msg = HashMap::new();
        msg.insert("subsystem", XPCObject::from(3 as u64));
        msg.insert("handle", XPCObject::from(0 as u64));
        msg.insert("routine", XPCObject::from(815 as u64));
        msg.insert("type", XPCObject::from(1 as u64));

        msg.insert("legacy", XPCObject::from(true));
        msg
    };

    /// launchctl load [path]
    pub static ref LOAD_PATHS: HashMap<&'static str, XPCObject> = {
        let mut msg = HashMap::new();
        msg.insert("routine", XPCObject::from(800 as u64));
        msg.insert("subsystem", XPCObject::from(3 as u64));
        msg.insert("handle", XPCObject::from(0 as u64));
        msg.insert("legacy", XPCObject::from(true));
        msg.insert("legacy-load", XPCObject::from(true));
        msg.insert("enable", XPCObject::from(false));
        msg.insert("no-einprogress", XPCObject::from(true));

        msg.insert(
            "domain-port",
            XPCObject::from(get_bootstrap_port() as mach_port_t),
        );

        msg
    };

    /// launchctl unload [path]
    pub static ref UNLOAD_PATHS: HashMap<&'static str, XPCObject> = {
        let mut msg = HashMap::new();
        msg.insert("routine", XPCObject::from(801 as u64));
        msg.insert("subsystem", XPCObject::from(3 as u64));
        msg.insert("handle", XPCObject::from(0 as u64));
        msg.insert("legacy", XPCObject::from(true));
        msg.insert("legacy-load", XPCObject::from(true));
        msg.insert("enable", XPCObject::from(false));
        msg.insert("no-einprogress", XPCObject::from(true));

        msg.insert(
            "domain-port",
            XPCObject::from(get_bootstrap_port() as mach_port_t),
        );

        msg
    };

    pub static ref ENABLE_NAMES: HashMap<&'static str, XPCObject> = {
        let mut msg = HashMap::new();
        msg.insert("routine", XPCObject::from(808 as u64));
        msg.insert("subsystem", XPCObject::from(3 as u64));
        // UID or ASID
        msg.insert("handle", XPCObject::from(0 as u64));

        msg
    };

    pub static ref DISABLE_NAMES: HashMap<&'static str, XPCObject> = {
        let mut msg = HashMap::new();
        msg.insert("routine", XPCObject::from(809 as u64));
        msg.insert("subsystem", XPCObject::from(3 as u64));
        // UID or ASID
        msg.insert("handle", XPCObject::from(0 as u64));

        msg
    };
}

pub fn from_msg<'a>(proto: &HashMap<&'a str, XPCObject>) -> HashMap<&'a str, XPCObject> {
    let mut new_msg: HashMap<&str, XPCObject> = HashMap::new();
    new_msg.extend(proto.iter().map(|(k, v)| (k.clone(), v.clone())));
    new_msg
}
