use crate::launchd::message::{from_msg, LIST_SERVICES};
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::path::Path;
use std::sync::Mutex;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::objects::xpc_type;
use xpc_sys::traits::xpc_pipeable::XPCPipeable;
use xpc_sys::traits::xpc_value::TryXPCValue;

use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_error::XPCError;
use crate::launchd::config::{LaunchdEntryType, LaunchdEntryLocation, USER_LAUNCH_AGENTS, ADMIN_LAUNCH_DAEMONS, ADMIN_LAUNCH_AGENTS, SYSTEM_LAUNCH_DAEMONS, SYSTEM_LAUNCH_AGENTS, LaunchdEntryConfig};
use std::iter::FromIterator;

lazy_static! {
    static ref ENTRY_INFO_CACHE: Mutex<HashMap<String, LaunchdEntryInfo>> =
        Mutex::new(HashMap::new());
}

/// LimitLoadToSessionType key in XPC response
/// https://developer.apple.com/library/archive/technotes/tn2083/_index.html
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LimitLoadToSessionType {
    Aqua,
    StandardIO,
    Background,
    LoginWindow,
    System,
    Unknown,
}

impl fmt::Display for LimitLoadToSessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TODO: This feels terrible
impl From<String> for LimitLoadToSessionType {
    fn from(value: String) -> Self {
        let aqua: String = LimitLoadToSessionType::Aqua.to_string();
        let standard_io: String = LimitLoadToSessionType::StandardIO.to_string();
        let background: String = LimitLoadToSessionType::Background.to_string();
        let login_window: String = LimitLoadToSessionType::LoginWindow.to_string();
        let system: String = LimitLoadToSessionType::System.to_string();

        match value {
            s if s == aqua => LimitLoadToSessionType::Aqua,
            s if s == standard_io => LimitLoadToSessionType::StandardIO,
            s if s == background => LimitLoadToSessionType::Background,
            s if s == login_window => LimitLoadToSessionType::LoginWindow,
            s if s == system => LimitLoadToSessionType::System,
            _ => LimitLoadToSessionType::Unknown,
        }
    }
}

impl TryFrom<XPCObject> for LimitLoadToSessionType {
    type Error = XPCError;

    fn try_from(value: XPCObject) -> Result<Self, Self::Error> {
        if value.xpc_type() != *xpc_type::String {
            return Err(XPCError::ValueError("xpc_type must be string".to_string()));
        }

        let string: String = value.xpc_value().unwrap();
        Ok(string.into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchdEntryInfo {
    pub entry_config: Option<LaunchdEntryConfig>,
    pub limit_load_to_session_type: LimitLoadToSessionType,
}

pub fn list(domain_type: u64, name: Option<String>) -> Result<XPCDictionary, XPCError> {
    let mut msg = from_msg(&LIST_SERVICES);
    msg.insert("type", domain_type.into());

    if name.is_some() {
        msg.insert("name", name.unwrap().into());
    }

    let msg: XPCObject = msg.into();
    msg
        .pipe_routine()
        .and_then(|o| o.try_into())
        .and_then(|dict: XPCDictionary| dict.0.get("error").map(|e| {
            Err(XPCError::PipeError(e.to_string()))
        }).unwrap_or(Ok(dict)))
}

pub fn list_all() -> HashSet<String> {
    let everything = (0..8)
        .filter_map(|t| {
            let svc_for_type = list(t as u64, None)
                .and_then(|d| d.get_as_dictionary(&["services"]))
                .map(|XPCDictionary(ref hm)| hm.keys().map(|k| k.clone()).collect());

            svc_for_type.ok()
        })
        .flat_map(|k: Vec<String>| k.into_iter());

    HashSet::from_iter(everything)
}

/// Get more information about a unit from its label
pub fn find_entry_info<T: Into<String>>(label: T) -> LaunchdEntryInfo {
    let label_string = label.into();
    let mut cache = ENTRY_INFO_CACHE.try_lock().unwrap();
    if cache.contains_key(label_string.as_str()) {
        return cache.get(label_string.as_str()).unwrap().clone();
    }

    let meta = build_entry_info(&label_string);
    cache.insert(label_string, meta.clone());
    meta
}

fn build_entry_info<T: Into<String>>(label: T) -> LaunchdEntryInfo {
    let label_string = label.into();

    // "launchctl list [name]", type is 7 when name provided?
    let limit_load_to_session_type = list(7 as u64, Some(label_string.clone()))
        .and_then(|d| d.get(&["service", "LimitLoadToSessionType"]))
        .and_then(|r| r.try_into())
        .unwrap_or(LimitLoadToSessionType::Unknown);

    let entry_config = crate::launchd::config::for_entry(label_string.clone());

    LaunchdEntryInfo {
        limit_load_to_session_type,
        entry_config
    }
}
