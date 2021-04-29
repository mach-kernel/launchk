use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Load,
    Unload,
    Edit,
    // (message, on ok)
    Prompt(String, Vec<OmniboxCommand>),
    Quit,
}

impl fmt::Display for OmniboxCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

pub static OMNIBOX_COMMANDS: [(OmniboxCommand, &str); 4] = [
    (OmniboxCommand::Load, "▶️  Load highlighted job"),
    (OmniboxCommand::Unload, "⏏️  Unload highlighted job"),
    (
        OmniboxCommand::Edit,
        "✍️  Edit plist with $EDITOR, then reload job",
    ),
    (OmniboxCommand::Quit, "🚪 see ya!"),
];
