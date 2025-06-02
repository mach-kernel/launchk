use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use crate::launchd::plist::LaunchdPlist;
use crate::launchd::command::find_in_all;

use xpc_sys::enums::{DomainType, SessionType};
use xpc_sys::object::try_xpc_into_rust::TryXPCIntoRust;
use xpc_sys::object::xpc_error::XPCError;
use xpc_sys::object::xpc_object::XPCHashMap;

const ENTRY_INFO_QUERY_TTL: Duration = Duration::from_secs(15);

lazy_static! {
    pub static ref ENTRY_STATUS_CACHE: Mutex<HashMap<String, LaunchdEntryStatus>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct LaunchdEntryStatus {
    pub plist: Option<LaunchdPlist>,
    pub limit_load_to_session_type: SessionType,
    pub domain: DomainType,
    // So, there is a pid_t, but it's i32, and the XPC response has an i64?
    pub pid: i64,
    tick: SystemTime,
}

impl Default for LaunchdEntryStatus {
    fn default() -> Self {
        LaunchdEntryStatus {
            limit_load_to_session_type: SessionType::Unknown,
            domain: DomainType::Unknown,
            plist: None,
            pid: 0,
            tick: SystemTime::now(),
        }
    }
}

/// Get entry info for label
pub fn get_entry_status<S: Into<String>>(label: S) -> LaunchdEntryStatus {
    let label_string = label.into();
    let mut cache = ENTRY_STATUS_CACHE.try_lock().unwrap();

    if cache.contains_key(label_string.as_str()) {
        let item = cache.get(label_string.as_str()).unwrap().clone();

        if item.tick.elapsed().unwrap() > ENTRY_INFO_QUERY_TTL {
            cache.remove(label_string.as_str());
            drop(cache);
            return get_entry_status(label_string);
        }

        return item;
    }

    let meta = build_entry_status(&label_string);
    cache.insert(label_string, meta.clone());
    meta
}

fn build_entry_status<S: Into<String>>(label: S) -> LaunchdEntryStatus {
    let label_string = label.into();
    let response = find_in_all(label_string.clone());

    let domain = response
        .clone()
        .map(|(domain, _)| domain)
        .unwrap_or(DomainType::Unknown);

    let service: XPCHashMap = response
        .and_then(|(_, d)| d.get("service").ok_or(XPCError::NotFound).cloned())
        .and_then(|o| o.to_rust())
        .unwrap_or(HashMap::new());

    let pid: i64 = service
        .get("PID")
        .and_then(|o| o.to_rust().ok())
        .unwrap_or(0);

    let limit_load_to_session_type: u64 = service
        .get("LimitLoadToSessionType")
        .and_then(|o| o.to_rust().ok())
        .unwrap_or(SessionType::Unknown as u64);

    let entry_config = crate::launchd::plist::for_label(label_string.clone());

    LaunchdEntryStatus {
        limit_load_to_session_type: SessionType::from(limit_load_to_session_type),
        domain,
        plist: entry_config,
        pid,
        tick: SystemTime::now(),
    }
}
