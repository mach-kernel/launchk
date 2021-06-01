use crate::launchd::enums::{DomainType, SessionType};
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_object::MachPortType;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::{get_bootstrap_port, mach_port_t};

/// Builder methods for XPCDictionary to make querying easier
pub trait QueryBuilder {
    /// Add entry to query
    fn entry<S: Into<String>, O: Into<XPCObject>>(self, key: S, value: O) -> XPCDictionary;

    /// Add entry if option is Some()
    fn entry_if_present<S: Into<String>, O: Into<XPCObject>>(
        self,
        key: S,
        value: Option<O>,
    ) -> XPCDictionary;

    /// Extend an existing XPCDictionary
    fn extend(self, other: &XPCDictionary) -> XPCDictionary;

    /// Adds "domain_port" with get_bootstrap_port() -> _xpc_type_mach_send
    fn with_domain_port_as_bootstrap_port(self) -> XPCDictionary
    where
        Self: Sized,
    {
        self.entry(
            "domain-port",
            (MachPortType::Send, unsafe {
                get_bootstrap_port() as mach_port_t
            }),
        )
    }

    /// Adds provided session type or falls back on Aqua
    fn with_session_type_or_default(self, session: Option<SessionType>) -> XPCDictionary
    where
        Self: Sized,
    {
        self.entry("session", session.unwrap_or(SessionType::Aqua).to_string())
    }

    /// Adds provided handle or falls back on 0
    fn with_handle_or_default(self, handle: Option<u64>) -> XPCDictionary
    where
        Self: Sized,
    {
        self.entry("handle", handle.unwrap_or(0))
    }

    /// Adds provided DomainType, falls back on 7 (requestor's domain)
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
        let xpc_object: XPCObject = value.into();
        hm.insert(key.into(), xpc_object.into());
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
