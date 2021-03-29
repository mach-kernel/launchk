use std::fmt;
use xpc_sys;
use xpc_sys::{uid_t, au_asid_t, pid_t, xpc_fd_create};
use std::fmt::Formatter;
use crate::launchd::message::from_msg;
use xpc_sys::objects::xpc_object::XPCObject;
use std::collections::HashMap;
use xpc_sys::traits::xpc_pipeable::{XPCPipeResult, XPCPipeable};
use std::borrow::Borrow;
use std::intrinsics::discriminant_value;

#[repr(C, u64)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Used in conjunction with the service name,
/// ordinals correspond to the `type` key in XPC
pub enum DomainTarget {
    System = 1,
    User(uid_t) = 2,
    Login(au_asid_t) = 3,
    GUI(uid_t) = 8,
    PID(pid_t) = 5,
}

impl fmt::Display for DomainTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = format!("{:?}", self).to_ascii_lowercase();

        match self {
            DomainTarget::User(uid) | DomainTarget::GUI(uid) => write!(f, "{}/{}", name, uid),
            DomainTarget::Login(asid) => write!(f, "{}/{}", name, asid),
            DomainTarget::PID(pid) => write!(f, "{}/{}", name, pid),
            _ => write!(f, "{}", name)
        }
    }
}

pub struct QueryTarget(pub DomainTarget, pub Option<String>);

impl fmt::Display for QueryTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let QueryTarget(domain, maybe_name) = self;
        let name = maybe_name.borrow().as_ref().map(|n| format!("/{}", n)).unwrap_or("".to_string());

        write!(f, "{}{}", domain, name)
    }
}

/// XPC query for launchctl print domain-target | service-target
pub fn print(qt: QueryTarget) -> XPCPipeResult {
    let QueryTarget(domain, maybe_name) = qt;

    // Absence of a name -> domain target only,
    // different routine + subsystem
    let routine: u64 = maybe_name.clone().map(|_| 708).unwrap_or(828);
    let subsystem: u64 = maybe_name.clone().map(|_| 2).unwrap_or(3);

    let mut msg: HashMap<&str, XPCObject> = HashMap::new();
    msg.insert("routine", routine.into());
    msg.insert("subsystem", subsystem.into());
    msg.insert("type", unsafe { discriminant_value(&domain) }.into());
    msg.insert("handle", (0 as u64).into());

    // Without this, EINVAL -- how can we get this as a dictionary?!
    // let fd: XPCObject = unsafe { xpc_fd_create(1) }.into();
    // msg.insert("fd", fd);

    if maybe_name.is_some() {
        msg.insert("name", maybe_name.unwrap().into());
    }

    let obj: XPCObject = msg.into();
    println!("Sending {}", obj);
    obj.pipe_routine()
}

