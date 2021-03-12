use std::collections::HashMap;
use xpc_sys::objects::xpc_object::XPCObject;

lazy_static! {
    pub static ref LIST_SERVICES: HashMap<&'static str, XPCObject> = {
        // "list com.apple.Spotlight" (if specified)
        // msg.insert("name", XPCObject::from("com.apple.Spotlight"));

        // Not necessary?
        //
        // msg.insert(
        //     "domain-port",
        //     XPCObject::from(get_bootstrap_port() as mach_port_t),
        // );

        let mut msg = HashMap::new();
        msg.insert("type", XPCObject::from(1 as u64));
        msg.insert("handle", XPCObject::from(0 as u64));
        msg.insert("subsystem", XPCObject::from(3 as u64));
        msg.insert("routine", XPCObject::from(815 as u64));
        msg.insert("legacy", XPCObject::from(true));
        msg
    };
}

pub fn from_msg<'a>(proto: &HashMap<&'a str, XPCObject>) -> HashMap<&'a str, XPCObject> {
    let mut new_msg: HashMap<&str, XPCObject> = HashMap::new();
    new_msg.extend(proto.iter().map(|(k, v)| (k.clone(), v.clone())));
    new_msg
}
