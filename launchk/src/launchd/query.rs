use crate::launchd::message::{from_msg, LIST_SERVICES, LOAD_PATHS, UNLOAD_PATHS};
use std::collections::{HashMap, HashSet};
use std::convert::{TryFrom, TryInto};
use std::fmt;

use std::sync::Mutex;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::objects::xpc_type;
use xpc_sys::traits::xpc_pipeable::{XPCPipeResult, XPCPipeable};
use xpc_sys::traits::xpc_value::TryXPCValue;

use crate::launchd::config::LaunchdEntryConfig;
use std::iter::FromIterator;
use std::time::{Duration, SystemTime};
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_error::XPCError;

const ENTRY_INFO_QUERY_TTL: u64 = 15; // seconds

#[link(name = "c")]
extern "C" {
    fn geteuid() -> u32;
}

lazy_static! {
    static ref ENTRY_INFO_CACHE: Mutex<HashMap<String, LaunchdEntryInfo>> =
        Mutex::new(HashMap::new());
    static ref IS_ROOT: bool = unsafe { geteuid() } == 0;
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
    // So, there is a pid_t, but it's i32, and the XPC response has an i64?
    pub pid: i64,
    tick: SystemTime,
}

impl Default for LaunchdEntryInfo {
    fn default() -> Self {
        LaunchdEntryInfo {
            limit_load_to_session_type: LimitLoadToSessionType::Unknown,
            entry_config: None,
            pid: 0,
            tick: SystemTime::now(),
        }
    }
}

pub fn find_in_all<S: Into<String>>(label: S) -> Result<XPCDictionary, XPCError> {
    let label_string = label.into();

    for domain_type in 0..8 {
        let response = list(domain_type, Some(label_string.clone()));
        if response.is_ok() {
            return response;
        }
    }

    Err(XPCError::NotFound)
}

pub fn list(domain_type: u64, name: Option<String>) -> Result<XPCDictionary, XPCError> {
    let mut msg = from_msg(&LIST_SERVICES);
    msg.insert("type", domain_type.into());

    if name.is_some() {
        msg.insert("name", name.unwrap().into());
    }

    let msg: XPCObject = msg.into();
    msg.pipe_routine()
        .and_then(|o| o.try_into())
        .and_then(|dict: XPCDictionary| {
            dict.0
                .get("error")
                .map(|e| Err(XPCError::PipeError(e.to_string())))
                .unwrap_or(Ok(dict))
        })
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
pub fn find_entry_info<S: Into<String>>(label: S) -> LaunchdEntryInfo {
    let label_string = label.into();
    let mut cache = ENTRY_INFO_CACHE.try_lock().unwrap();

    if cache.contains_key(label_string.as_str()) {
        let item = cache.get(label_string.as_str()).unwrap().clone();

        if item.tick.elapsed().unwrap() > Duration::from_secs(ENTRY_INFO_QUERY_TTL) {
            cache.remove(label_string.as_str());
            drop(cache);
            return find_entry_info(label_string);
        }

        return item;
    }

    let meta = build_entry_info(&label_string);
    cache.insert(label_string, meta.clone());
    meta
}

fn build_entry_info<S: Into<String>>(label: S) -> LaunchdEntryInfo {
    let label_string = label.into();
    let response = find_in_all(label_string.clone());

    let pid: i64 = response
        .as_ref()
        .map_err(|e| e.clone())
        .and_then(|r| r.get(&["service", "PID"]))
        .and_then(|o| o.xpc_value())
        .unwrap_or(0);

    let limit_load_to_session_type = response
        .as_ref()
        .map_err(|e| e.clone())
        .and_then(|r| r.get(&["service", "LimitLoadToSessionType"]))
        .and_then(|o| o.try_into())
        .unwrap_or(LimitLoadToSessionType::Unknown);

    let entry_config = crate::launchd::config::for_entry(label_string.clone());

    LaunchdEntryInfo {
        limit_load_to_session_type,
        entry_config,
        pid,
        tick: SystemTime::now(),
    }
}

pub fn load<S: Into<String>>(label: S, plist_path: S) -> XPCPipeResult {
    let mut message: HashMap<&str, XPCObject> = from_msg(&LOAD_PATHS);
    let label_string = label.into();

    message.insert(
        "type",
        if *IS_ROOT {
            XPCObject::from(1 as u64)
        } else {
            XPCObject::from(7 as u64)
        },
    );

    let paths = vec![XPCObject::from(plist_path.into())];
    message.insert("paths", XPCObject::from(paths));
    message.insert("session", XPCObject::from("Aqua"));

    let message: XPCObject = message.into();

    ENTRY_INFO_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label_string);

    handle_load_unload_errors(label_string, message.pipe_routine()?)
}

pub fn unload<S: Into<String>>(label: S, plist_path: S) -> XPCPipeResult {
    let mut message: HashMap<&str, XPCObject> = from_msg(&UNLOAD_PATHS);
    let label_string = label.into();

    message.insert(
        "type",
        if *IS_ROOT {
            XPCObject::from(1 as u64)
        } else {
            XPCObject::from(7 as u64)
        },
    );

    let paths = vec![XPCObject::from(plist_path.into())];
    message.insert("paths", XPCObject::from(paths));
    message.insert("session", XPCObject::from("Aqua"));

    let message: XPCObject = message.into();

    ENTRY_INFO_CACHE
        .lock()
        .expect("Must invalidate")
        .remove(&label_string);

    handle_load_unload_errors(label_string, message.pipe_routine()?)
}

fn handle_load_unload_errors(label: String, result: XPCObject) -> XPCPipeResult {
    let dict: XPCDictionary = result.clone().try_into()?;
    let error_dict = dict.get_as_dictionary(&["errors"]);

    if error_dict.is_err() {
        Ok(result)
    } else {
        let mut error_string = "".to_string();
        let XPCDictionary(hm) = error_dict.unwrap();

        if hm.is_empty() {
            return Ok(result);
        }

        for (_, errcode) in hm {
            let errcode: i64 = errcode.xpc_value().unwrap();
            error_string.push_str(
                format!("{}: {}\n", label, xpc_sys::rs_xpc_strerror(errcode as i32)).as_str(),
            );
        }

        Err(XPCError::QueryError(error_string))
    }
}
