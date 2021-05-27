use crate::launchd::enums::{DomainType, SessionType};
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_object::MachPortType;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::{get_bootstrap_port, mach_port_t};

pub trait QueryBuilder {
    fn entry<S: Into<String>, O: Into<XPCObject>>(self, key: S, value: O) -> XPCDictionary;
    fn entry_if_present<S: Into<String>, O: Into<XPCObject>>(
        self,
        key: S,
        value: Option<O>,
    ) -> XPCDictionary;

    fn extend(self, other: &XPCDictionary) -> XPCDictionary;

    fn with_domain_port(self) -> XPCDictionary
    where
        Self: Sized,
    {
        self.entry(
            "domain-port",
            (MachPortType::Send, get_bootstrap_port() as mach_port_t),
        )
    }

    fn with_session_type_or_default(self, session: Option<SessionType>) -> XPCDictionary
    where
        Self: Sized,
    {
        self.entry("session", session.unwrap_or(SessionType::Aqua).to_string())
    }

    fn with_handle_or_default(self, handle: Option<u64>) -> XPCDictionary
    where
        Self: Sized,
    {
        self.entry("handle", handle.unwrap_or(0))
    }

    fn with_domain_type_or_default(self, t: Option<DomainType>) -> XPCDictionary
    where
        Self: Sized,
    {
        self.entry("type", t.unwrap_or(DomainType::RequestorDomain) as u64)
    }
}

impl QueryBuilder for XPCDictionary {
    fn entry<S: Into<String>, O: Into<XPCObject>>(mut self, key: S, value: O) -> XPCDictionary {
        let Self(hm) = &mut self;
        hm.insert(key.into(), value.into());
        self
    }

    fn entry_if_present<S: Into<String>, O: Into<XPCObject>>(
        self,
        key: S,
        value: Option<O>,
    ) -> XPCDictionary {
        if value.is_none() {
            self
        } else {
            self.entry(key, value.unwrap())
        }
    }

    fn extend(mut self, other: &XPCDictionary) -> XPCDictionary {
        let Self(self_hm) = &mut self;
        let Self(other_hm) = other;
        self_hm.extend(other_hm.iter().map(|(s, o)| (s.clone(), o.clone())));
        self
    }
}
