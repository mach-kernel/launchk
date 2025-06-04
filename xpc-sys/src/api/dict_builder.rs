use crate::enums::{DomainType, SessionType};
use crate::object::xpc_object::XPCObject;
use crate::object::xpc_object::{MachPortType, XPCHashMap};
use crate::{get_bootstrap_port, rs_geteuid};
use mach2::port::mach_port_t;

/// Builder methods for XPCHashMap
pub trait DictBuilder {
    /// Add entry
    fn entry<S: Into<String>, O: Into<XPCObject>>(self, key: S, value: O) -> XPCHashMap;

    /// Add entry if option is Some()
    fn entry_if_present<S: Into<String>, O: Into<XPCObject>>(
        self,
        key: S,
        value: Option<O>,
    ) -> XPCHashMap;

    /// Add entry if option is Some()
    fn entry_if<S: Into<String>, O: Into<XPCObject>>(
        self,
        pred: bool,
        key: S,
        value: O,
    ) -> XPCHashMap;

    /// Extend an existing XPCHashMap
    fn extend(self, other: &XPCHashMap) -> XPCHashMap;

    /// Adds "domain_port" with get_bootstrap_port() -> _xpc_type_mach_send
    fn with_domain_port_as_bootstrap_port(self) -> XPCHashMap
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
    fn with_session_type_or_default(self, session: Option<SessionType>) -> XPCHashMap
    where
        Self: Sized,
    {
        self.entry("session", session.unwrap_or(SessionType::Aqua).to_string())
    }

    /// Adds provided handle or falls back on 0
    fn with_handle_or_default(self, handle: Option<u64>) -> XPCHashMap
    where
        Self: Sized,
    {
        self.entry("handle", handle.unwrap_or(0))
    }

    /// Adds provided DomainType, falls back on 7 (requestor's domain)
    fn with_domain_type_or_default(self, t: Option<DomainType>) -> XPCHashMap
    where
        Self: Sized,
    {
        self.entry("type", t.unwrap_or(DomainType::RequestorDomain) as u64)
    }

    /// Adds provided DomainType, falls back on 7 (requestor's domain)
    fn handle_and_type_from_domain(self, t: DomainType) -> XPCHashMap
    where
        Self: Sized,
    {
        self
            // no handle for system
            .entry_if(t == DomainType::System, "handle", 0u64)
            .entry_if(t == DomainType::System, "type", 1u64)
            // uid as handle for user
            .entry_if(t == DomainType::User, "handle", rs_geteuid() as u64)
            .entry_if(t == DomainType::User, "type", 8u64)
    }
}

impl DictBuilder for XPCHashMap {
    fn entry<S: Into<String>, O: Into<XPCObject>>(mut self, key: S, value: O) -> XPCHashMap {
        let xpc_object: XPCObject = value.into();
        self.insert(key.into(), xpc_object.into());
        self
    }

    fn entry_if_present<S: Into<String>, O: Into<XPCObject>>(
        self,
        key: S,
        value: Option<O>,
    ) -> XPCHashMap {
        if let Some(v) = value {
            self.entry(key, v)
        } else {
            self
        }
    }

    fn entry_if<S: Into<String>, O: Into<XPCObject>>(
        self,
        pred: bool,
        key: S,
        value: O,
    ) -> XPCHashMap {
        if pred {
            self.entry(key, value)
        } else {
            self
        }
    }

    fn extend(mut self, other: &XPCHashMap) -> XPCHashMap {
        let m = &mut self;
        m.extend(other.iter().map(|(s, o)| (s.clone(), o.clone())));
        self
    }
}
