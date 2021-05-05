use crate::launchd::message::{LIST_SERVICES, LOAD_PATHS, UNLOAD_PATHS};
use std::collections::HashSet;

use xpc_sys::objects::xpc_object::XPCObject;

use xpc_sys::traits::xpc_pipeable::XPCPipeable;

use crate::launchd::entry_status::ENTRY_STATUS_CACHE;
use std::iter::FromIterator;
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_error::XPCError;

use crate::launchd::enums::{DomainType, SessionType};

// TODO: reuse list_all()
pub fn find_in_all<S: Into<String>>(label: S) -> Result<XPCDictionary, XPCError> {
    let label_string = label.into();

    for domain_type in DomainType::System as u64..DomainType::RequestorDomain as u64 {
        let response = XPCDictionary::new()
            .extend(&LIST_SERVICES)
            .entry("type", domain_type)
            .entry("name", label_string.clone())
            .pipe_routine_with_error_handling();

        if response.is_ok() {
            return response;
        }
    }

    Err(XPCError::NotFound)
}

/// Query for jobs in a domain
pub fn list(domain_type: DomainType, name: Option<String>) -> Result<XPCDictionary, XPCError> {
    XPCDictionary::new()
        .extend(&LIST_SERVICES)
        .entry("type", domain_type as u64)
        .entry_if_present("name", name)
        .pipe_routine_with_error_handling()
}

/// Query for jobs across all domain types
pub fn list_all() -> HashSet<String> {
    let everything = (DomainType::System as u64..DomainType::RequestorDomain as u64)
        .filter_map(|t| {
            let svc_for_type = list(t.into(), None)
                .and_then(|d| d.get_as_dictionary(&["services"]))
                .map(|XPCDictionary(ref hm)| hm.keys().map(|k| k.clone()).collect());

            svc_for_type.ok()
        })
        .flat_map(|k: Vec<String>| k.into_iter());

    HashSet::from_iter(everything)
}

pub fn load<S: Into<String>>(
    label: S,
    plist_path: S,
    domain_type: Option<DomainType>,
    session: Option<SessionType>,
    handle: Option<u64>,
) -> Result<XPCDictionary, XPCError> {
    ENTRY_STATUS_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label.into());

    XPCDictionary::new()
        .extend(&LOAD_PATHS)
        .entry(
            "type",
            domain_type.unwrap_or(DomainType::RequestorDomain) as u64,
        )
        .entry("handle", handle.unwrap_or(0))
        .entry(
            "session",
            session.map(|s| s.to_string()).unwrap_or("Aqua".to_string()),
        )
        .entry("paths", vec![XPCObject::from(plist_path.into())])
        .pipe_routine_with_error_handling()
}

pub fn unload<S: Into<String>>(
    label: S,
    plist_path: S,
    domain_type: Option<DomainType>,
    session: Option<SessionType>,
    handle: Option<u64>,
) -> Result<XPCDictionary, XPCError> {
    ENTRY_STATUS_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label.into());

    XPCDictionary::new()
        .extend(&UNLOAD_PATHS)
        .entry(
            "type",
            domain_type.unwrap_or(DomainType::RequestorDomain) as u64,
        )
        .entry("handle", handle.unwrap_or(0))
        .entry(
            "session",
            session.map(|s| s.to_string()).unwrap_or("Aqua".to_string()),
        )
        .entry("paths", vec![XPCObject::from(plist_path.into())])
        .pipe_routine_with_error_handling()
}
