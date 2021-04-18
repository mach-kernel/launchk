use std::borrow::Borrow;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::{Mutex, Once};

use crate::tui::job_type_filter::JobTypeFilter;
use std::fs::{DirEntry, ReadDir};
use tokio::runtime::Handle;
use std::sync::mpsc::{channel, Receiver, Sender};
use notify::{watcher, DebouncedEvent, Watcher, RecursiveMode};
use std::time::Duration;
use std::iter::FilterMap;

pub static LABEL_MAP_INIT: Once = Once::new();

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

async fn fsnotify_subscriber() {
    let (tx, rx): (Sender<DebouncedEvent>, Receiver<DebouncedEvent>) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(5))
        .expect("Must make fsnotify watcher");

    // Register plist paths
    let watchers = [
        watcher.watch(Path::new(USER_LAUNCH_AGENTS), RecursiveMode::Recursive),
        watcher.watch(Path::new(GLOBAL_LAUNCH_AGENTS), RecursiveMode::Recursive),
        watcher.watch(Path::new(SYSTEM_LAUNCH_AGENTS), RecursiveMode::Recursive),
        watcher.watch(Path::new(ADMIN_LAUNCH_DAEMONS), RecursiveMode::Recursive),
        watcher.watch(Path::new(GLOBAL_LAUNCH_DAEMONS), RecursiveMode::Recursive),
    ];

    for sub in watchers.iter() {
        sub.expect("Must subscribe to fs events");
    }

    loop {
        let event = rx.recv();
        if event.is_err() {
            continue;
        }

        let event = event.unwrap();

        let reload_plists = match event {
            DebouncedEvent::Create(pb) => fs::read_dir(pb),
            DebouncedEvent::Write(pb) => fs::read_dir(pb),
            DebouncedEvent::Remove(pb) => fs::read_dir(pb),
            DebouncedEvent::Rename(_, new) => fs::read_dir(new),
            _ => continue,
        };

        if reload_plists.is_err() {
            continue;
        }

        insert_plists(readdir_filter_plists(reload_plists.unwrap()));
    }
}

fn build_label_map_entry(plist_path: DirEntry) -> Option<(String, LaunchdEntryConfig)> {
    let path = plist_path.path();
    let path_string = path.to_string_lossy().to_string();

    let label = plist::Value::from_file(path.clone()).ok()?;
    let label = label
        .as_dictionary()
        .and_then(|d| d.get("Label"))
        .and_then(|v| v.as_string());

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

    Some((label?.to_string(), LaunchdEntryConfig {
        entry_location,
        entry_type,
        plist_path: path_string,
        readonly: path
            .metadata()
            .map(|m| m.permissions().readonly())
            .unwrap_or(true),
    }))
}

fn readdir_filter_plists(rd: ReadDir) -> FilterMap<ReadDir, fn(futures::io::Result<DirEntry>) -> Option<DirEntry>> {
    rd.filter_map(|e|  {
        if e.is_err() {
            return None;
        }

        let path = e.borrow().as_ref().unwrap().path();

        if path.is_dir()
            || path
            .extension()
            .map(|ex| ex.to_string_lossy().ne("plist"))
            .unwrap_or(true) {
            None
        } else {
            Some(e.unwrap())
        }
    })
}

fn insert_plists(plists: impl Iterator<Item=DirEntry>) {
    let mut label_map = LABEL_TO_PLIST.lock().unwrap();

    for plist_path in plists {
        let entry = build_label_map_entry(plist_path);
        if entry.is_none() { continue; }
        let (label, entry) = entry.unwrap();
        label_map.insert(label, entry);
    }
}

/// Unsure if this is overkill, since the filenames
/// usually match the label property. Still looking for
/// a way to do dumpstate, dumpjpcategory without parsing the string
pub fn init_label_map(runtime_handle: &Handle) {
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
        .flat_map(readdir_filter_plists);

    insert_plists(plists);

    // Spawn fsnotify subscriber
    runtime_handle.spawn(async { fsnotify_subscriber().await });
}

/// Get plist + fs meta for an entry by its label
pub fn for_entry<S: Into<String>>(label: S) -> Option<LaunchdEntryConfig> {
    let label_map = LABEL_TO_PLIST.try_lock().ok()?;
    label_map.get(label.into().as_str()).map(|c| c.clone())
}
