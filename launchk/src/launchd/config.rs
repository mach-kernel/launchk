// See https://www.launchd.info/ for more details

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

lazy_static! {
    static ref UNIT_META_CACHE: Mutex<HashMap<String, Option<LaunchdEntryConfig>>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntryType {
    /// Runs on behalf of currently logged in user
    Agent,
    /// Global system daemon
    Daemon,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntryLocation {
    /// macOS system provided agent or daemon
    System,
    /// "Administrator provided" agent or daemon
    Global,
    /// User provided agent
    User,
}

/// When querying XPC for a specific service, the LimitLoadToSessionType
/// key contains this information.
///
/// https://developer.apple.com/library/archive/technotes/tn2083/_index.html
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SessionType {
    Aqua,
    StandardIO,
    Background,
    LoginWindow,
    System,
    Unknown,
}

impl fmt::Display for SessionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

// TODO: This feels terrible
impl From<String> for SessionType {
    fn from(value: String) -> Self {
        let aqua: String = SessionType::Aqua.to_string();
        let standard_io: String = SessionType::StandardIO.to_string();
        let background: String = SessionType::Background.to_string();
        let login_window: String = SessionType::LoginWindow.to_string();
        let system: String = SessionType::System.to_string();

        match value {
            s if s == aqua => SessionType::Aqua,
            s if s == standard_io => SessionType::StandardIO,
            s if s == background => SessionType::Background,
            s if s == login_window => SessionType::LoginWindow,
            s if s == system => SessionType::System,
            _ => SessionType::Unknown,
        }
    }
}

impl TryFrom<XPCObject> for SessionType {
    type Error = ();

    fn try_from(value: XPCObject) -> Result<Self, Self::Error> {
        if value.xpc_type() != *xpc_type::String {
            return Err(());
        }

        let string: String = value.xpc_value().unwrap();
        Ok(string.into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchdEntryConfig {
    /// Path to launchd plist
    pub config_path: String,
    /// Is the plist read only?
    pub readonly: bool,
    /// Meta
    pub entry_type: EntryType,
    pub entry_location: EntryLocation,
    pub session_type: SessionType,
}

const USER_LAUNCH_AGENTS: &str = concat!(env!("HOME"), "/Library/LaunchAgents");
const ADMIN_LAUNCH_AGENTS: &str = "/Library/LaunchAgents";
const SYSTEM_LAUNCH_AGENTS: &str = "/System/Library/LaunchAgents";

const ADMIN_LAUNCH_DAEMONS: &str = "/Library/LaunchDaemons";
const SYSTEM_LAUNCH_DAEMONS: &str = "/System/Library/LaunchDaemons";

pub fn find_unit<T: Into<String>>(name: T) -> Option<LaunchdEntryConfig> {
    let name_string = name.into();
    let mut cache = UNIT_META_CACHE.try_lock().unwrap();
    if cache.contains_key(name_string.as_str()) {
        return cache.get(name_string.as_str()).unwrap().clone();
    }

    let meta = get_unit_meta(&name_string);
    cache.insert(name_string, meta.clone());
    meta
}

fn get_unit_meta<T: Into<String>>(name: T) -> Option<LaunchdEntryConfig> {
    let name_string: String = name.into();
    for parent_dir in [
        USER_LAUNCH_AGENTS,
        ADMIN_LAUNCH_AGENTS,
        SYSTEM_LAUNCH_AGENTS,
        ADMIN_LAUNCH_DAEMONS,
        SYSTEM_LAUNCH_DAEMONS,
    ]
    .iter()
    {
        let path = Path::new(*parent_dir).join(format!("{}.plist", name_string));
        let fs_meta = path.metadata();

        if fs_meta.is_err() {
            continue;
        }

        // "launchctl list [name]"
        let mut msg: HashMap<&str, XPCObject> = from_msg(&LIST_SERVICES);
        msg.insert("name", name_string.clone().into());
        let msg: XPCObject = msg.into();
        let limit_load_to_session_type: Result<SessionType, XPCError> = msg
            .pipe_routine()
            .and_then(|r| r.try_into())
            .and_then(|XPCDictionary(ref hm)| {
                hm.get("service")
                    .map(|s| s.clone())
                    .ok_or(XPCError::StandardError)
            })
            .and_then(|s| s.try_into())
            .and_then(|XPCDictionary(ref hm)| {
                hm.get("LimitLoadToSessionType")
                    .map(|s| s.clone())
                    .ok_or(XPCError::StandardError)
            })
            .and_then(|o| o.clone().try_into().map_err(|_| XPCError::StandardError));

        let fs_meta = fs_meta.unwrap();
        let entry_type = match *parent_dir {
            ADMIN_LAUNCH_DAEMONS | SYSTEM_LAUNCH_DAEMONS => EntryType::Daemon,
            _ => EntryType::Agent,
        };
        let entry_location = match *parent_dir {
            USER_LAUNCH_AGENTS => EntryLocation::User,
            ADMIN_LAUNCH_AGENTS | ADMIN_LAUNCH_DAEMONS => EntryLocation::Global,
            _ => EntryLocation::System,
        };

        return Some(LaunchdEntryConfig {
            config_path: path.to_string_lossy().to_string(),
            readonly: fs_meta.permissions().readonly(),
            session_type: limit_load_to_session_type.unwrap_or(SessionType::Unknown),
            entry_type,
            entry_location,
        });
    }

    None
}
