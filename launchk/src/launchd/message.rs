use std::collections::HashMap;
use xpc_sys::api::dict_builder::DictBuilder;
use xpc_sys::object::xpc_object::XPCHashMap;
// A bunch of XPCHashMap 'protos' that can be extended to make XPC queries

lazy_static! {
    /// launchctl list [name]
    pub static ref LIST_SERVICES: XPCHashMap = HashMap::new()
        // "list com.apple.Spotlight" (if specified)
        // .entry("name", "com.apple.Spotlight");
        .entry("subsystem", 3_u64)
        .entry("handle", 0_u64)
        .entry("routine", 815_u64)
        .entry("legacy", true);

    /// launchctl dumpstate
    /// Requires a shmem xpc_object_t member, see XPCShmem for more details
    pub static ref DUMPSTATE: XPCHashMap = HashMap::new()
        .entry("subsystem", 3_u64)
        .entry("routine", 834_u64)
        .entry("type", 1_u64)
        .with_handle_or_default(None);

    /// launchctl dumpjpcategory
    /// Requires a FD".entry("fd", 1 as RawFd)"
    pub static ref DUMPJPCATEGORY: XPCHashMap = HashMap::new()
        .entry("subsystem", 3_u64)
        .entry("routine", 837_u64)
        .entry("type", 1_u64)
        .with_handle_or_default(None);

    /// launchctl procinfo
    /// Requires a FD".entry("fd", 1 as RawFd)"
    pub static ref PROCINFO: XPCHashMap = HashMap::new()
        .entry("subsystem", 2_u64)
        .entry("routine", 708_u64);
}
