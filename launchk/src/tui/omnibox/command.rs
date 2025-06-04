use std::fmt;
use xpc_sys::enums::DomainType;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Chain(Vec<OmniboxCommand>),
    // (message, on ok)
    Confirm(String, Vec<OmniboxCommand>),
    BootstrapRequest,
    BootoutRequest,
    EnableRequest,
    DisableRequest,
    Blame,
    Bootstrap(DomainType),
    Bootout(DomainType),
    Enable(DomainType),
    Disable(DomainType),
    Edit,
    FocusServiceList,
    CSRInfo,
    DumpState,
    DumpJetsamPropertiesCategory,
    ProcInfo,
    Sudo,
    Help,
    Quit,
}

impl fmt::Display for OmniboxCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

pub static OMNIBOX_COMMANDS: [(&str, &str, OmniboxCommand); 12] = [
    (
        "blame",
        "Why launchd launched the service",
        OmniboxCommand::Blame,
    ),
    (
        "bootstrap",
        "Bootstrap highlighted service",
        OmniboxCommand::BootstrapRequest,
    ),
    (
        "bootout",
        "Stop highlighted service",
        OmniboxCommand::BootoutRequest,
    ),
    (
        "enable",
        "Enable highlighted job (enables load)",
        OmniboxCommand::EnableRequest,
    ),
    (
        "disable",
        "Disable highlighted job (prevents load)",
        OmniboxCommand::DisableRequest,
    ),
    ("edit", "Edit plist with $EDITOR", OmniboxCommand::Edit),
    ("csrinfo", "See all CSR flags", OmniboxCommand::CSRInfo),
    (
        "dumpstate",
        "launchctl dumpstate",
        OmniboxCommand::DumpState,
    ),
    (
        "dumpjpcategory",
        "launchctl dumpjpcategory",
        OmniboxCommand::DumpJetsamPropertiesCategory,
    ),
    (
        "procinfo",
        "launchctl procinfo for highlighted process",
        OmniboxCommand::ProcInfo,
    ),
    ("help", "Show all commands", OmniboxCommand::Help),
    ("exit", "Exit", OmniboxCommand::Quit),
];
