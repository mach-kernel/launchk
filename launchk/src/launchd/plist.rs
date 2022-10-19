use std::collections::HashMap;
use std::env;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Once, RwLock};

use crate::launchd::job_type_filter::JobTypeFilter;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::runtime::Handle;

pub static PLIST_MAP_INIT: Once = Once::new();

lazy_static! {
    pub static ref LABEL_TO_ENTRY_CONFIG: RwLock<HashMap<String, LaunchdPlist>> =
        RwLock::new(HashMap::new());
    static ref EDITOR: String = env::var("EDITOR").unwrap_or("vim".to_string());
    static ref TMP_DIR: String = env::var("TMPDIR").unwrap_or("/tmp".to_string());
    static ref USER_LAUNCH_AGENTS: String =
        env::var("HOME").expect("Must read $HOME") + "/Library/LaunchAgents";
}

/*
od -xc binary.plist
0000000      7062    696c    7473    3030
            b   p   l   i   s   t   0   0
*/
static PLIST_MAGIC: &str = "bplist00";

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
    /// Admin provided agent or daemon in /Library,
    /// would name it admin...but the [sguadl] filter
    /// needs uniques
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
pub struct LaunchdPlist {
    pub entry_type: LaunchdEntryType,
    pub entry_location: LaunchdEntryLocation,
    pub plist_path: String,
    pub readonly: bool,
}

// TODO: This should be somewhere else
impl LaunchdPlist {
    pub fn job_type_filter(&self, is_loaded: bool) -> JobTypeFilter {
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

        if is_loaded {
            jtf.toggle(JobTypeFilter::LOADED);
        }

        jtf
    }
}

pub const ADMIN_LAUNCH_AGENTS: &str = "/Library/LaunchAgents";
pub const SYSTEM_LAUNCH_AGENTS: &str = "/System/Library/LaunchAgents";

pub const ADMIN_LAUNCH_DAEMONS: &str = "/Library/LaunchDaemons";
pub const SYSTEM_LAUNCH_DAEMONS: &str = "/System/Library/LaunchDaemons";

async fn fsnotify_subscriber() {
    let (tx, rx): (Sender<DebounceEventResult>, Receiver<DebounceEventResult>) = channel();
    let mut debouncer = new_debouncer(Duration::from_secs(5), None, tx).unwrap();
    let watcher = debouncer.watcher();

    // Register plist paths
    let watchers = [
        watcher.watch(Path::new(&*USER_LAUNCH_AGENTS), RecursiveMode::Recursive),
        watcher.watch(Path::new(ADMIN_LAUNCH_AGENTS), RecursiveMode::Recursive),
        watcher.watch(Path::new(SYSTEM_LAUNCH_AGENTS), RecursiveMode::Recursive),
        watcher.watch(Path::new(ADMIN_LAUNCH_DAEMONS), RecursiveMode::Recursive),
        watcher.watch(Path::new(SYSTEM_LAUNCH_DAEMONS), RecursiveMode::Recursive),
    ];

    for sub in watchers.iter() {
        sub.as_ref().expect("Must subscribe to fs events");
    }

    loop {
        let events = rx
            .recv()
            .map(|dbr| dbr.ok())
            .ok()
            .flatten()
            .unwrap_or(vec![]);

        let paths: Vec<PathBuf> = events
            .iter()
            .filter_map(|e| path_if_plist(&e.path))
            .collect();

        insert_plists(paths.into_iter());
    }
}

fn build_label_map_entry(plist_path: PathBuf) -> Option<(String, LaunchdPlist)> {
    let path_string = plist_path.to_string_lossy().to_string();
    let label = plist::Value::from_file(&path_string).ok()?;
    let label = label
        .as_dictionary()
        .and_then(|d| d.get("Label"))
        .and_then(|v| v.as_string());

    let entry_type = if path_string.starts_with(ADMIN_LAUNCH_DAEMONS)
        || path_string.starts_with(SYSTEM_LAUNCH_DAEMONS)
    {
        LaunchdEntryType::Daemon
    } else {
        LaunchdEntryType::Agent
    };

    let entry_location = if path_string.starts_with(&*USER_LAUNCH_AGENTS) {
        LaunchdEntryLocation::User
    } else if path_string.starts_with(ADMIN_LAUNCH_AGENTS)
        || path_string.starts_with(ADMIN_LAUNCH_DAEMONS)
    {
        LaunchdEntryLocation::Global
    } else {
        LaunchdEntryLocation::System
    };

    Some((
        label?.to_string(),
        LaunchdPlist {
            entry_location,
            entry_type,
            plist_path: path_string,
            readonly: plist_path
                .metadata()
                .map(|m| m.permissions().readonly())
                .unwrap_or(true),
        },
    ))
}

fn path_if_plist(path: &PathBuf) -> Option<PathBuf> {
    if path.is_dir()
        || path
            .extension()
            .map(|ex| ex.to_string_lossy().ne("plist"))
            .unwrap_or(true)
    {
        None
    } else {
        Some(path.clone())
    }
}

fn insert_plists(plists: impl Iterator<Item = PathBuf>) {
    let mut label_map = LABEL_TO_ENTRY_CONFIG.write().expect("Must update");

    for plist_path in plists {
        log::info!("Loading plist {:?}", plist_path);

        let entry = build_label_map_entry(plist_path);
        if entry.is_none() {
            continue;
        }
        let (label, entry) = entry.unwrap();
        label_map.insert(label, entry);
    }
}

/// Unsure if this is overkill, since the filenames
/// usually match the label property. Still looking for
/// a way to do dumpstate, dumpjpcategory without parsing the string
pub fn init_plist_map(runtime_handle: &Handle) {
    let dirs = [
        &USER_LAUNCH_AGENTS,
        ADMIN_LAUNCH_AGENTS,
        SYSTEM_LAUNCH_AGENTS,
        ADMIN_LAUNCH_DAEMONS,
        SYSTEM_LAUNCH_DAEMONS,
    ];

    // Get all the plists from everywhere into one stream
    let plists = dirs
        .iter()
        .filter_map(|&dirname| fs::read_dir(Path::new(dirname)).ok())
        .flat_map(|rd| rd.map(|e| e.ok()))
        .flatten()
        .filter_map(|d| path_if_plist(&d.path()));

    insert_plists(plists);

    // Spawn fsnotify subscriber
    runtime_handle.spawn(fsnotify_subscriber());
}

/// Get plist for a label
pub fn for_label<S: Into<String>>(label: S) -> Option<LaunchdPlist> {
    let label_map = LABEL_TO_ENTRY_CONFIG.read().ok()?;
    label_map.get(label.into().as_str()).map(|c| c.clone())
}

/// Given a LaunchdPlist, start editor pointing to temporary file
/// and replace on exit. Uses plist crate to validate changes and
/// help show contents for binary encoded files
pub fn edit_and_replace(plist_meta: &LaunchdPlist) -> Result<(), String> {
    if plist_meta.readonly {
        return Err("plist is read-only!".to_string());
    }

    let mut file =
        File::open(&plist_meta.plist_path).map_err(|_| "Couldn't read file".to_string())?;

    // We want to write back in the correct format,
    // can't assume we can safely write XML everywhere?
    let mut magic_buf: [u8; 8] = [0; 8];
    file.read_exact(&mut magic_buf)
        .map_err(|_| "Couldn't read magic".to_string())?;
    let is_binary = std::str::from_utf8(&magic_buf)
        .map_err(|_| "Couldn't read magic".to_string())?
        == PLIST_MAGIC;

    // plist -> validate with crate -> temp file
    let og_plist = plist::Value::from_file(&plist_meta.plist_path).map_err(|e| e.to_string())?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Must get ts");
    let temp_path = Path::new(&*TMP_DIR).join(format!("{}", now.as_secs()));
    og_plist
        .to_file_xml(&temp_path)
        .map_err(|e| e.to_string())?;

    // Start $EDITOR
    let exit = Command::new(&*EDITOR)
        .arg(&temp_path)
        .status()
        .map_err(|e| format!("{} failed: {}", &*EDITOR, e.to_string()))?;

    if !exit.success() {
        return Err(format!("{} did not exit successfully", &*EDITOR));
    }

    // temp file -> validate with crate -> original
    let plist =
        plist::Value::from_file(&temp_path).map_err(|e| format!("Changes not saved: {}", e))?;

    if og_plist == plist {
        return Err("No changes made".to_string());
    }

    let writer = if is_binary {
        plist::Value::to_file_binary
    } else {
        plist::Value::to_file_xml
    };

    writer(&plist, &plist_meta.plist_path).map_err(|e| e.to_string())?;

    Ok(())
}
