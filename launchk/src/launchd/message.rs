use std::collections::HashMap;
use xpc_sys::api::dict_builder::DictBuilder;
use xpc_sys::object::xpc_object::XPCHashMap;
// A bunch of XPCHashMap 'protos' that can be extended to make XPC queries

lazy_static! {
    /// launchctl list [name]
    pub static ref LIST_SERVICES: XPCHashMap = HashMap::new()
        // "list com.apple.Spotlight" (if specified)
        // .entry("name", "com.apple.Spotlight");
        .entry("subsystem", 3 as u64)
        .entry("handle", 0 as u64)
        .entry("routine", 815 as u64)
        .entry("legacy", true);

    /// launchctl load [path]
    pub static ref LOAD_PATHS: XPCHashMap = HashMap::new()
        .with_domain_port_as_bootstrap_port()
        .entry("routine", 800 as u64)
        .entry("subsystem", 3 as u64)
        .entry("handle", 0 as u64)
        .entry("legacy", true)
        .entry("legacy-load", true)
        .entry("enable", false)
        .entry("no-einprogress", true);

    /// launchctl unload [path]
    pub static ref UNLOAD_PATHS: XPCHashMap = HashMap::new()
        .with_domain_port_as_bootstrap_port()
        .entry("routine", 801 as u64)
        .entry("subsystem", 3 as u64)
        .entry("handle", 0 as u64)
        .entry("legacy", true)
        .entry("legacy-load", true)
        .entry("enable", false)
        .entry("no-einprogress", true);

    /// launchctl bootout domain/name
    pub static ref BOOTOUT: XPCHashMap = HashMap::new()
        .with_domain_port_as_bootstrap_port()
        .entry("routine", 801 as u64)
        .entry("subsystem", 3 as u64)
        .entry("no-einprogress", true);
    
    /// launchctl enable
    pub static ref ENABLE_NAMES: XPCHashMap = HashMap::new()
        .with_domain_port_as_bootstrap_port()
        // .entry("handle", UID or ASID)
        .entry("routine", 808 as u64)
        .entry("subsystem", 3 as u64);

    /// launchctl disable
    pub static ref DISABLE_NAMES: XPCHashMap = HashMap::new()
        .with_domain_port_as_bootstrap_port()
        // .entry("handle", UID or ASID)
        .entry("routine", 809 as u64)
        .entry("subsystem", 3 as u64);

    /// launchctl dumpstate
    /// Requires a shmem xpc_object_t member, see XPCShmem for more details
    pub static ref DUMPSTATE: XPCHashMap = HashMap::new()
        .entry("subsystem", 3 as u64)
        .entry("routine", 834 as u64)
        .entry("type", 1 as u64)
        .with_handle_or_default(None);

    /// launchctl dumpjpcategory
    /// Requires a FD".entry("fd", 1 as RawFd)"
    pub static ref DUMPJPCATEGORY: XPCHashMap = HashMap::new()
        .entry("subsystem", 3 as u64)
        .entry("routine", 837 as u64)
        .entry("type", 1 as u64)
        .with_handle_or_default(None);

    /// launchctl procinfo
    /// Requires a FD".entry("fd", 1 as RawFd)"
    pub static ref PROCINFO: XPCHashMap = HashMap::new()
        .entry("subsystem", 2 as u64)
        .entry("routine", 708 as u64);
}
