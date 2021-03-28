use std::fmt;
use xpc_sys::uid_t;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
/// Used in conjunction with the service name
pub enum DomainTarget {
    User(uid_t),
    GUI(uid_t),
    System,
}

pub struct ServiceTarget(pub DomainTarget, pub String);

impl fmt::Display for DomainTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl DomainTarget {
    fn for_service<S>(&self, name: S) -> String
    where
        S: Into<String>,
    {
        vec![self.to_string(), name.into()].join("/")
    }
}

