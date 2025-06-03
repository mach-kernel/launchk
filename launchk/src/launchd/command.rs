use crate::launchd::message::{DUMPJPCATEGORY, DUMPSTATE, LIST_SERVICES, PROCINFO};
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::ffi::CString;
use xpc_sys::{object::xpc_shmem::XPCShmem, rs_geteuid, MAP_SHARED};

use crate::launchd::entry_status::ENTRY_STATUS_CACHE;
use regex::Regex;
use std::iter::FromIterator;
use std::slice::from_raw_parts;
use xpc_sys::api::dict_builder::DictBuilder;
use xpc_sys::api::pipe_routine::{handle_reply_dict_errors, pipe_interface_routine, pipe_routine};
use xpc_sys::enums::DomainType;
use xpc_sys::object::try_xpc_into_rust::TryXPCIntoRust;
use xpc_sys::object::xpc_error::XPCError;
use xpc_sys::object::xpc_object::XPCHashMap;

pub fn find_in_all<S: Into<String>>(label: S) -> Result<(DomainType, XPCHashMap), XPCError> {
    let label_string = label.into();

    for domain_type in DomainType::System as u64..=DomainType::RequestorDomain as u64 {
        let dict: XPCHashMap = HashMap::new()
            .extend(&LIST_SERVICES)
            .entry("type", domain_type)
            .entry("name", label_string.clone());

        let response = pipe_routine(None, dict).and_then(handle_reply_dict_errors);

        if response.is_ok() {
            return Ok((domain_type.into(), response.unwrap().to_rust()?));
        }
    }

    Err(XPCError::NotFound)
}

/// Query for jobs in a domain
pub fn list(domain_type: DomainType, name: Option<String>) -> Result<XPCHashMap, XPCError> {
    let dict = HashMap::new()
        .extend(&LIST_SERVICES)
        .with_domain_type_or_default(Some(domain_type))
        .entry_if_present("name", name);

    pipe_routine(None, dict)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())
}

/// Query for jobs across all domain types
pub fn list_all() -> HashSet<String> {
    let mut everything = vec![
        DomainType::System,
        DomainType::RequestorUserDomain,
        DomainType::RequestorDomain,
    ];

    if rs_geteuid() == 0 {
        everything.push(DomainType::User);
    }

    let list = everything
        .iter()
        .filter_map(|t| {
            let svc_for_type = list(t.clone(), None)
                .and_then(|d| {
                    d.get("services")
                        .ok_or(XPCError::NotFound)
                        .map(|o| o.clone())
                })
                .and_then(|o| o.to_rust())
                .map(|d: XPCHashMap| d.keys().map(|k| k.clone()).collect());

            if svc_for_type.is_err() {
                log::error!(
                    "[query/list_all]: poll error {}, domain, {}",
                    svc_for_type.err().unwrap(),
                    t
                );
                None
            } else {
                svc_for_type.ok()
            }
        })
        .flat_map(|k: Vec<String>| k.into_iter());

    HashSet::from_iter(list)
}

pub fn blame<S: Into<String>>(label: S, domain_type: DomainType) -> Result<String, XPCError> {
    let label_string = label.into();
    log::debug!("blame: {} {}", &label_string, domain_type);

    ENTRY_STATUS_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label_string);

    let dict = HashMap::new()
        .entry("name", label_string)
        .handle_and_type_from_domain(domain_type);

    let response: XPCHashMap = pipe_interface_routine(None, 707, dict, None)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())?;

    let reason: String = response
        .get("reason")
        .ok_or(XPCError::NotFound)
        .and_then(|o| o.to_rust())?;

    Ok(reason)
}

pub fn bootout<S: Into<String>>(label: S, domain_type: DomainType) -> Result<XPCHashMap, XPCError> {
    let label_string = label.into();
    log::debug!("bootout: {} {}", &label_string, domain_type);

    ENTRY_STATUS_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label_string);

    let dict = HashMap::new()
        .entry("name", label_string)
        .entry("no-einprogress", true)
        .handle_and_type_from_domain(domain_type);

    pipe_interface_routine(None, 801, dict, None)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())
}

pub fn bootstrap<S: Into<String>>(
    label: S,
    domain_type: DomainType,
    plist_path: S,
) -> Result<XPCHashMap, XPCError> {
    let label_string = label.into();
    log::debug!("bootstrap: {} {}", &label_string, domain_type);

    ENTRY_STATUS_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label_string);

    let dict = HashMap::new()
        .entry("by-cli", true)
        .entry("paths", vec![plist_path.into()])
        .handle_and_type_from_domain(domain_type);

    pipe_interface_routine(None, 800, dict, None)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())
}

pub fn enable<S: Into<String>>(label: S, domain_type: DomainType) -> Result<XPCHashMap, XPCError> {
    let label_string = label.into();

    let dict = HashMap::new()
        .entry("name", label_string.clone())
        .entry("names", vec![label_string])
        .handle_and_type_from_domain(domain_type);

    pipe_interface_routine(None, 808, dict, None)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())
}

pub fn disable<S: Into<String>>(label: S, domain_type: DomainType) -> Result<XPCHashMap, XPCError> {
    let label_string = label.into();

    let dict = HashMap::new()
        .entry("name", label_string.clone())
        .entry("names", vec![label_string])
        .handle_and_type_from_domain(domain_type);

    pipe_interface_routine(None, 809, dict, None)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())
}

/// Create a shared shmem region for the XPC routine to write
/// dumpstate contents into, and return the bytes written and
/// shmem region
pub fn dumpstate() -> Result<(usize, XPCShmem), XPCError> {
    let shmem = XPCShmem::allocate_task_self(
        0x1400000,
        i32::try_from(MAP_SHARED).expect("Must conv flags"),
    )?;

    let dict = HashMap::new()
        .extend(&DUMPSTATE)
        .entry("shmem", &shmem.xpc_object);

    let response: XPCHashMap = pipe_routine(None, dict)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())?;

    let bytes_written: u64 = response
        .get("bytes-written")
        .ok_or(XPCError::NotFound)?
        .to_rust()?;

    Ok((usize::try_from(bytes_written).unwrap(), shmem))
}

pub fn dumpjpcategory() -> Result<(usize, XPCShmem), XPCError> {
    let shmem = XPCShmem::allocate_task_self(
        0x1400000,
        i32::try_from(MAP_SHARED).expect("Must conv flags"),
    )?;

    let dict = HashMap::new()
        .extend(&DUMPJPCATEGORY)
        .entry("shmem", &shmem.xpc_object);

    let response: XPCHashMap = pipe_routine(None, dict)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())?;

    let bytes_written: u64 = response
        .get("bytes-written")
        .ok_or(XPCError::NotFound)?
        .to_rust()?;

    Ok((usize::try_from(bytes_written).unwrap(), shmem))
}

pub fn procinfo(pid: i64) -> Result<(usize, XPCShmem), XPCError> {
    let shmem = XPCShmem::allocate_task_self(
        0x1400000,
        i32::try_from(MAP_SHARED).expect("Must conv flags"),
    )?;

    let dict = HashMap::new()
        .extend(&PROCINFO)
        .entry("shmem", &shmem.xpc_object)
        .entry("pid", pid);

    let response: XPCHashMap = pipe_routine(None, dict)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())?;

    let bytes_written: u64 = response
        .get("bytes-written")
        .ok_or(XPCError::NotFound)?
        .to_rust()?;

    Ok((usize::try_from(bytes_written).unwrap(), shmem))
}

pub fn read_disabled(domain_type: DomainType) -> Result<(usize, XPCShmem), XPCError> {
    let shmem = XPCShmem::allocate_task_self(
        1_000_000,
        i32::try_from(MAP_SHARED).expect("Must conv flags"),
    )?;

    let dict = HashMap::new()
        .entry("shmem", &shmem.xpc_object)
        .handle_and_type_from_domain(domain_type);

    let response: XPCHashMap = pipe_interface_routine(None, 828, dict, None)
        .and_then(handle_reply_dict_errors)
        .and_then(|o| o.to_rust())?;

    let bytes_written: u64 = response
        .get("bytes-written")
        .ok_or(XPCError::NotFound)?
        .to_rust()?;

    Ok((usize::try_from(bytes_written).unwrap(), shmem))
}

pub fn read_disabled_hashset(domain_type: DomainType) -> Result<HashSet<String>, XPCError> {
    let (sz, shmem) = read_disabled(domain_type)?;

    // Copy out of shmem and make a CString
    let slice: &[u8] = unsafe { from_raw_parts(shmem.region as *const _, sz) };
    let data: Vec<u8> = slice.iter().map(|c| c.clone()).collect();
    let cs = unsafe { CString::from_vec_unchecked(data) };

    // Find all the quoted service names
    let re =
        Regex::new(r#""([\w.]+)" => disabled"#).map_err(|e| XPCError::ValueError(e.to_string()))?;

    let services: Vec<String> = re
        .captures_iter(cs.to_str().unwrap())
        .flat_map(|c| c.iter().flatten().map(|m| m.as_str().to_string()).last())
        .collect();

    let mut hs = HashSet::new();
    hs.extend(services.iter().map(|s| s.clone()));
    Ok(hs)
}
