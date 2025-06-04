use std::time::SystemTime;

use crate::launchd::job_type_filter::JobTypeFilter;
use crate::tui::omnibox::command::{OmniboxCommand, OMNIBOX_COMMANDS};
use crate::tui::omnibox::view::OmniboxMode;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OmniboxState {
    pub mode: OmniboxMode,
    pub tick: SystemTime,
    pub label_filter: String,
    pub command_filter: String,
    pub job_type_filter: JobTypeFilter,
}

impl OmniboxState {
    /// Produce new state
    pub fn with_new(
        &self,
        mode: Option<OmniboxMode>,
        label_filter: Option<String>,
        command_filter: Option<String>,
        job_type_filter: Option<JobTypeFilter>,
    ) -> OmniboxState {
        OmniboxState {
            tick: SystemTime::now(),
            mode: mode.unwrap_or(self.mode.clone()),
            label_filter: label_filter.unwrap_or(self.label_filter.clone()),
            command_filter: command_filter.unwrap_or(self.command_filter.clone()),
            job_type_filter: job_type_filter.unwrap_or(self.job_type_filter),
        }
    }

    /// Suggest a command based on name filter
    pub fn suggest_command(&self) -> Option<(&str, &str, OmniboxCommand)> {
        let OmniboxState {
            mode,
            command_filter,
            ..
        } = self;

        if *mode != OmniboxMode::CommandFilter || command_filter.is_empty() {
            return None;
        }

        OMNIBOX_COMMANDS
            .iter().find(|(c, _, _)| c.to_string().starts_with(command_filter)).cloned()
    }
}

impl Default for OmniboxState {
    fn default() -> Self {
        Self {
            mode: OmniboxMode::Idle,
            tick: SystemTime::now(),
            label_filter: "".to_string(),
            command_filter: "".to_string(),
            job_type_filter: JobTypeFilter::launchk_default(),
        }
    }
}
