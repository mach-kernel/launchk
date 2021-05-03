use std::fmt;

use crate::launchd::query::{DomainType, LimitLoadToSessionType};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Chain(Vec<OmniboxCommand>),
    // Load(DomainType, Option<u64>, LimitLoadToSessionType),
    // Unload(DomainType, Option<u64>),
    Load,
    Unload,
    // Reuses domain, handle, limit load to session type from existing
    Reload,
    Enable,
    Disable,
    Edit,
    // (message, on ok)
    Prompt(String, Vec<OmniboxCommand>),
    FocusServiceList,
    Quit,
}

impl fmt::Display for OmniboxCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

pub static OMNIBOX_COMMANDS: [(OmniboxCommand, &str); 7] = [
    (OmniboxCommand::Load, "▶️  Load highlighted job"),
    (OmniboxCommand::Unload, "⏏️  Unload highlighted job"),
    (OmniboxCommand::Enable, "▶️  Enable highlighted job (enables load)"),
    (OmniboxCommand::Disable, "⏏️  Disable highlighted job (prevents load)"),
    (
        OmniboxCommand::Edit,
        "✍️  Edit plist with $EDITOR, then reload job",
    ),
    (OmniboxCommand::Reload ,"🔄  Reload highlighted job"),
    (OmniboxCommand::Quit, "🚪 see ya!"),
];
