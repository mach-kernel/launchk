use crate::launchd::message::{from_msg, LIST_SERVICES};
use std::collections::HashMap;
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

    // "launchctl list [name]"
    let mut msg: HashMap<&str, XPCObject> = from_msg(&LIST_SERVICES);
    msg.insert("name", label_string.clone().into());

    let msg: XPCObject = msg.into();
    let limit_load_to_session_type: LimitLoadToSessionType = msg
        .pipe_routine()
        .and_then(|o| o.try_into())
        .and_then(|d: XPCDictionary| d.get(&["service", "LimitLoadToSessionType"]))
        .and_then(|r| r.try_into())
        .unwrap_or(LimitLoadToSessionType::Unknown);

    let entry_config = crate::launchd::config::for_entry(label_string.clone());

    LaunchdEntryInfo {
        limit_load_to_session_type,
        entry_config
    }
}
