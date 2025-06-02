use std::fmt;
use xpc_sys::enums::{DomainType, SessionType};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Chain(Vec<OmniboxCommand>),
    // (message, on ok)
    Confirm(String, Vec<OmniboxCommand>),
    // Try to see if we have session type & domain in entry_status,
    // to avoid having to prompt the user
    LoadRequest,
    UnloadRequest,
    BootstrapRequest,
    BootoutRequest,
    EnableRequest,
    DisableRequest,
    Bootstrap(DomainType),
    Bootout(DomainType),
    Load(SessionType, DomainType, Option<u64>),
    Unload(DomainType, Option<u64>),
    // Reuses domain, handle, limit load to session type from existing
    Reload,
    Enable(DomainType),
    Disable(DomainType),
    Edit,
    // (unit label, prompt for domain only?, action gen fn)
    DomainSessionPrompt(
        String,
        bool,
        fn(DomainType, Option<SessionType>) -> Vec<OmniboxCommand>,
    ),
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
        "bootstrap",
        "▶️  Bootstrap highlighted service",
        OmniboxCommand::BootstrapRequest,
    ),
    (
        "bootout",
        "⏏️  Remove highlighted job",
        OmniboxCommand::BootoutRequest,
    ),
    (
        "enable",
        "▶️  Enable highlighted job (enables load)",
        OmniboxCommand::EnableRequest,
    ),
    (
        "disable",
        "⏏️  Disable highlighted job (prevents load)",
        OmniboxCommand::DisableRequest,
    ),
    (
        "edit",
        "✍️  Edit plist with $EDITOR then reload job",
        OmniboxCommand::Edit,
    ),
    (
        "reload",
        "🔄  Reload highlighted job",
        OmniboxCommand::Reload,
    ),
    ("csrinfo", "ℹ️  See all CSR flags", OmniboxCommand::CSRInfo),
    (
        "dumpstate",
        "ℹ️  launchctl dumpstate",
        OmniboxCommand::DumpState,
    ),
    (
        "dumpjpcategory",
        "ℹ️  launchctl dumpjpcategory",
        OmniboxCommand::DumpJetsamPropertiesCategory,
    ),
    (
        "procinfo",
        "ℹ️  launchctl procinfo for highlighted process",
        OmniboxCommand::ProcInfo,
    ),
    ("help", "🤔  Show all commands", OmniboxCommand::Help),
    ("exit", "🚪  see ya!", OmniboxCommand::Quit),
];
