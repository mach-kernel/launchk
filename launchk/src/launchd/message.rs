use crate::launchd::query_builder::QueryBuilder;
use xpc_sys::objects::xpc_dictionary::XPCDictionary;

lazy_static! {
    /// launchctl list [name]
    pub static ref LIST_SERVICES: XPCDictionary = XPCDictionary::new()
        // "list com.apple.Spotlight" (if specified)
        // .entry("name", "com.apple.Spotlight");
        .entry("subsystem", 3 as u64)
        .entry("handle", 0 as u64)
        .entry("routine", 815 as u64)
        .entry("legacy", true);

    /// launchctl load [path]
    pub static ref LOAD_PATHS: XPCDictionary = XPCDictionary::new()
        .with_domain_port()
        .entry("routine", 800 as u64)
        .entry("subsystem", 3 as u64)
        .entry("handle", 0 as u64)
        .entry("legacy", true)
        .entry("legacy-load", true)
        .entry("enable", false)
        .entry("no-einprogress", true);

    /// launchctl unload [path]
    pub static ref UNLOAD_PATHS: XPCDictionary = XPCDictionary::new()
        .with_domain_port()
        .entry("routine", 801 as u64)
        .entry("subsystem", 3 as u64)
        .entry("handle", 0 as u64)
        .entry("legacy", true)
        .entry("legacy-load", true)
        .entry("enable", false)
        .entry("no-einprogress", true);


    pub static ref ENABLE_NAMES: XPCDictionary = XPCDictionary::new()
        .with_domain_port()
        // .entry("handle", UID or ASID)
        .entry("routine", 808 as u64)
        .entry("subsystem", 3 as u64);

    pub static ref DISABLE_NAMES: XPCDictionary = XPCDictionary::new()
        .with_domain_port()
        // .entry("handle", UID or ASID)
        .entry("routine", 809 as u64)
        .entry("subsystem", 3 as u64);

    pub static ref DUMPSTATE: XPCDictionary = XPCDictionary::new()
        .entry("subsystem", 3 as u64)
        .entry("routine", 834 as u64)
        .entry("type", 1 as u64)
        .with_handle_or_default(None);
}
