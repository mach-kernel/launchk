use std::collections::HashMap;
use std::sync::{Mutex, Once};

use std::fs;
use std::path::Path;

use futures::StreamExt;

use std::borrow::Borrow;

use crate::tui::job_type_filter::JobTypeFilter;
use std::fmt;

static LABEL_MAP_INIT: Once = Once::new();

lazy_static! {
    static ref LABEL_TO_PLIST: Mutex<HashMap<String, LaunchdEntryConfig>> =
        Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LaunchdEntryType {
    /// Runs on behalf of currently logged in user
    Agent,
    /// Global system daemon
    Daemon,
}

impl fmt::Display for LaunchdEntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LaunchdEntryLocation {
    /// macOS system provided agent or daemon
    System,
    /// "Administrator provided" agent or daemon
    /// TODO: Global? Local? What's right?
    Global,
    /// User provided agent
    User,
}

impl fmt::Display for LaunchdEntryLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchdEntryConfig {
    pub entry_type: LaunchdEntryType,
    pub entry_location: LaunchdEntryLocation,
    pub plist_path: String,
    pub readonly: bool,
}

impl LaunchdEntryConfig {
    pub fn job_type_filter(&self) -> JobTypeFilter {
        let mut jtf = JobTypeFilter::default();

        match self.entry_location {
            LaunchdEntryLocation::System => jtf.toggle(JobTypeFilter::SYSTEM),
            LaunchdEntryLocation::Global => jtf.toggle(JobTypeFilter::GLOBAL),
            LaunchdEntryLocation::User => jtf.toggle(JobTypeFilter::USER),
        };

        match self.entry_type {
            LaunchdEntryType::Agent => jtf.toggle(JobTypeFilter::AGENT),
            LaunchdEntryType::Daemon => jtf.toggle(JobTypeFilter::DAEMON),
        };

        jtf
    }
}

pub const USER_LAUNCH_AGENTS: &str = concat!(env!("HOME"), "/Library/LaunchAgents");
pub const GLOBAL_LAUNCH_AGENTS: &str = "/Library/LaunchAgents";
pub const SYSTEM_LAUNCH_AGENTS: &str = "/System/Library/LaunchAgents";

pub const ADMIN_LAUNCH_DAEMONS: &str = "/Library/LaunchDaemons";
pub const GLOBAL_LAUNCH_DAEMONS: &str = "/System/Library/LaunchDaemons";

/// Unsure if this is overkill, since the filenames
/// usually match the label property. Still looking for
/// a way to do dumpstate, dumpjpcategory without parsing the string
fn init_label_map() {
    let dirs = [
        USER_LAUNCH_AGENTS,
        GLOBAL_LAUNCH_AGENTS,
        SYSTEM_LAUNCH_AGENTS,
        ADMIN_LAUNCH_DAEMONS,
        GLOBAL_LAUNCH_DAEMONS,
    ];

    // Get all the plists from everywhere into one stream
    let plists = dirs
        .iter()
        .filter_map(|&dirname| fs::read_dir(Path::new(dirname)).ok())
        .flat_map(|rd| {
            rd.flat_map(|e| {
                if e.is_err() {
                    return e.into_iter();
                }
                let path = e.borrow().as_ref().unwrap().path();

                if path.is_dir()
                    || path
                        .extension()
                        .map(|ex| ex.to_string_lossy().ne("plist"))
                        .unwrap_or(true)
                {
                    Err(()).into_iter()
                } else {
                    e.into_iter()
                }
            })
        });

    let mut label_map = LABEL_TO_PLIST.lock().unwrap();

    for plist_path in plists {
        let path = plist_path.path();
        let path_string = path.to_string_lossy().to_string();

        let label = plist::Value::from_file(path.clone());

        if label.is_err() {
            continue;
        }

        let label = label.unwrap();
        let label = label
            .as_dictionary()
            .and_then(|d| d.get("Label"))
            .and_then(|v| v.as_string());

        if label.is_none() {
            continue;
        }

        let entry_type = if path_string.contains(ADMIN_LAUNCH_DAEMONS)
            || path_string.contains(GLOBAL_LAUNCH_DAEMONS)
        {
            LaunchdEntryType::Daemon
        } else {
            LaunchdEntryType::Agent
        };

        let entry_location = if path_string.contains(USER_LAUNCH_AGENTS) {
            LaunchdEntryLocation::User
        } else if path_string.contains(GLOBAL_LAUNCH_AGENTS)
            || path_string.contains(ADMIN_LAUNCH_DAEMONS)
        {
            LaunchdEntryLocation::Global
        } else {
            LaunchdEntryLocation::System
        };

        label_map.insert(
            label.unwrap().to_string(),
            LaunchdEntryConfig {
                entry_location,
                entry_type,
                plist_path: path_string,
                readonly: path
                    .metadata()
                    .map(|m| m.permissions().readonly())
                    .unwrap_or(true),
            },
        );
    }
}

/// Get plist + fs meta for an entry by its label
pub fn for_entry<S: Into<String>>(label: S) -> Option<LaunchdEntryConfig> {
    LABEL_MAP_INIT.call_once(init_label_map);
    let label_map = LABEL_TO_PLIST.try_lock().ok()?;
    label_map.get(label.into().as_str()).map(|c| c.clone())
}
