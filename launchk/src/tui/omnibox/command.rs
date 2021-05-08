use crate::launchd::enums::{DomainType, SessionType};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Chain(Vec<OmniboxCommand>),
    Load(SessionType, DomainType, Option<u64>),
    Unload(DomainType, Option<u64>),
    // Reuses domain, handle, limit load to session type from existing
    Reload,
    Enable(DomainType),
    Disable(DomainType),
    Edit,
    // (message, on ok)
    Confirm(String, Vec<OmniboxCommand>),
    // (prompt for domain only?, action gen fn)
    DomainSessionPrompt(
        bool,
        fn(DomainType, Option<SessionType>) -> Vec<OmniboxCommand>,
    ),
    FocusServiceList,
    Quit,
}

impl fmt::Display for OmniboxCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

pub static OMNIBOX_COMMANDS: [(&str, &str, OmniboxCommand); 7] = [
    (
        "load",
        "▶️  Load highlighted job",
        OmniboxCommand::DomainSessionPrompt(false, |dt, st| {
            vec![OmniboxCommand::Load(
                st.expect("Must be provided"),
                dt,
                None,
            )]
        }),
    ),
    (
        "unload",
        "⏏️  Unload highlighted job",
        OmniboxCommand::DomainSessionPrompt(false, |dt, _| vec![OmniboxCommand::Unload(dt, None)]),
    ),
    (
        "enable",
        "▶️  Enable highlighted job (enables load)",
        OmniboxCommand::DomainSessionPrompt(true, |dt, _| vec![OmniboxCommand::Enable(dt)]),
    ),
    (
        "disable",
        "⏏️  Disable highlighted job (prevents load)",
        OmniboxCommand::DomainSessionPrompt(true, |dt, _| vec![OmniboxCommand::Disable(dt)]),
    ),
    (
        "edit",
        "✍️  Edit plist with $EDITOR, then reload job",
        OmniboxCommand::Edit,
    ),
    (
        "reload",
        "🔄  Reload highlighted job",
        OmniboxCommand::Reload,
    ),
    ("exit", "🚪 see ya!", OmniboxCommand::Quit),
];
