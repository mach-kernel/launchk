// See https://www.launchd.info/ for more details

use std::path::Path;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntryScope {
    /// Runs on behalf of currently logged in user
    Agent,
    /// Global system daemon
    Daemon,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EntryProvider {
    /// macOS system provided agent or daemon
    System,
    /// Admin provided agent or daemon
    Admin,
    /// User provided agent
    User,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct LaunchdEntryConfig {
    /// Path to launchd plist
    pub config_path: String,
    /// Is the plist read only?
    pub readonly: bool,
    /// Meta
    pub scope: EntryScope,
    pub provider: EntryProvider,
}

const USER_LAUNCH_AGENTS: &str = concat!(env!("HOME"), "/Library/LaunchAgents");
const ADMIN_LAUNCH_AGENTS: &str = "/Library/LaunchAgents";
const SYSTEM_LAUNCH_AGENTS: &str = "/System/Library/LaunchAgents";

const ADMIN_LAUNCH_DAEMONS: &str = "/Library/LaunchDaemons";
const SYSTEM_LAUNCH_DAEMONS: &str = "/System/Library/LaunchDaemons";

pub fn find_unit<T: Into<String>>(name: T) -> Option<LaunchdEntryConfig> {
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
        let meta = path.metadata();

        if meta.is_err() {
            continue;
        }

        let meta = meta.unwrap();
        let scope = match *parent_dir {
            ADMIN_LAUNCH_DAEMONS | SYSTEM_LAUNCH_DAEMONS => EntryScope::Daemon,
            _ => EntryScope::Agent,
        };
        let provider = match *parent_dir {
            USER_LAUNCH_AGENTS => EntryProvider::User,
            ADMIN_LAUNCH_AGENTS | ADMIN_LAUNCH_DAEMONS => EntryProvider::Admin,
            _ => EntryProvider::System,
        };

        return Some(LaunchdEntryConfig {
            config_path: path.to_string_lossy().to_string(),
            readonly: meta.permissions().readonly(),
            scope,
            provider,
        });
    }

    None
}
