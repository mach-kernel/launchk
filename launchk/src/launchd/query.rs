use crate::launchd::message::{
    DISABLE_NAMES, ENABLE_NAMES, LIST_SERVICES, LOAD_PATHS, UNLOAD_PATHS, DUMPSTATE
};
use std::collections::HashSet;

use xpc_sys::{objects::xpc_shmem::XPCShmem, traits::xpc_pipeable::XPCPipeable};

use crate::launchd::entry_status::ENTRY_STATUS_CACHE;
use std::iter::FromIterator;
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_error::XPCError;

use crate::launchd::enums::{DomainType, SessionType};
use crate::launchd::query_builder::QueryBuilder;

pub fn find_in_all<S: Into<String>>(label: S) -> Result<(DomainType, XPCDictionary), XPCError> {
    let label_string = label.into();

    for domain_type in DomainType::System as u64..DomainType::RequestorDomain as u64 {
        let response = XPCDictionary::new()
            .extend(&LIST_SERVICES)
            .entry("type", domain_type)
            .entry("name", label_string.clone())
            .pipe_routine_with_error_handling();

        if response.is_ok() {
            return response.map(|r| (domain_type.into(), r));
        }
    }

    Err(XPCError::NotFound)
}

/// Query for jobs in a domain
pub fn list(domain_type: DomainType, name: Option<String>) -> Result<XPCDictionary, XPCError> {
    XPCDictionary::new()
        .extend(&LIST_SERVICES)
        .with_domain_type_or_default(Some(domain_type))
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
        .with_domain_type_or_default(domain_type)
        .with_session_type_or_default(session)
        .with_handle_or_default(handle)
        .entry("paths", vec![plist_path.into()])
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
        .with_domain_type_or_default(domain_type)
        .with_session_type_or_default(session)
        .with_handle_or_default(handle)
        .entry("paths", vec![plist_path.into()])
        .pipe_routine_with_error_handling()
}

pub fn enable<S: Into<String>>(
    label: S,
    domain_type: DomainType,
) -> Result<XPCDictionary, XPCError> {
    let label_string = label.into();

    XPCDictionary::new()
        .extend(&ENABLE_NAMES)
        .with_domain_type_or_default(Some(domain_type))
        .entry("name", label_string.clone())
        .entry("names", vec![label_string])
        .with_handle_or_default(None)
        .pipe_routine_with_error_handling()
}

pub fn disable<S: Into<String>>(
    label: S,
    domain_type: DomainType,
) -> Result<XPCDictionary, XPCError> {
    let label_string = label.into();

    XPCDictionary::new()
        .extend(&DISABLE_NAMES)
        .with_domain_type_or_default(Some(domain_type))
        .entry("name", label_string.clone())
        .entry("names", vec![label_string])
        .with_handle_or_default(None)
        .pipe_routine_with_error_handling()
}

pub fn dumpstate() -> Result<XPCDictionary, XPCError> {
    let shmem = XPCShmem::new_task_self(0x1400000, 0)?;

    XPCDictionary::new()
        .extend(&DUMPSTATE)
        // this is going to vm_deallocate when this function exits. need better solve.
        .entry("shmem", shmem.xpc_object.clone())
        .pipe_routine_with_error_handling()
}
