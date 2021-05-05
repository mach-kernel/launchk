use std::fmt;
use crate::launchd::enums::{DomainType, SessionType};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Chain(Vec<OmniboxCommand>),
    Load(SessionType, DomainType, Option<u64>),
    Unload(DomainType, Option<u64>),
    // Reuses domain, handle, limit load to session type from existing
    Reload,
    Enable,
    Disable,
    Edit,
    // (message, on ok)
    Confirm(String, Vec<OmniboxCommand>),
    DomainSessionPrompt(fn(DomainType, SessionType) -> Vec<OmniboxCommand>),
    FocusServiceList,
    Quit,
}

impl fmt::Display for OmniboxCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

pub static OMNIBOX_COMMANDS: [(&str, &str, OmniboxCommand); 7] = [
    ("load", "▶️  Load highlighted job", OmniboxCommand::DomainSessionPrompt(|dt, st| vec![
        OmniboxCommand::Load(st, dt, None)
    ])),
    ("unload", "⏏️  Unload highlighted job", OmniboxCommand::DomainSessionPrompt(|dt, _| vec![
        OmniboxCommand::Unload(dt, None)
    ])),
    ("enable", "▶️  Enable highlighted job (enables load)", OmniboxCommand::Enable),
    ("disable", "⏏️  Disable highlighted job (prevents load)", OmniboxCommand::Disable),
    (
        "edit",
        "✍️  Edit plist with $EDITOR, then reload job",
        OmniboxCommand::Edit,
    ),
    ("reload" ,"🔄  Reload highlighted job", OmniboxCommand::Reload),
    ("exit", "🚪 see ya!", OmniboxCommand::Quit),
];
