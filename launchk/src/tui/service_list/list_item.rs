use crate::launchd::entry_status::LaunchdEntryStatus;
use crate::launchd::job_type_filter::JobTypeFilter;
use crate::tui::table::table_list_view::TableListItem;
use xpc_sys::enums::SessionType;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServiceListItem {
    pub name: String,
    pub status: LaunchdEntryStatus,
    pub job_type_filter: JobTypeFilter,
}

impl TableListItem for ServiceListItem {
    fn as_row(&self) -> Vec<String> {
        let session_type = match &self.status.limit_load_to_session_type {
            SessionType::Unknown => "-".to_string(),
            other => other.to_string(),
        };

        let entry_type = self
            .status
            .plist
            .as_ref()
            .map(|p| p.entry_type.to_string())
            .unwrap_or("-".to_string());

        let pid = if self.status.pid > 0 && self.job_type_filter.intersects(JobTypeFilter::LOADED) {
            format!("{}", self.status.pid)
        } else {
            "-".to_string()
        };

        let mut loaded = if self.job_type_filter.intersects(JobTypeFilter::LOADED) {
            "✔".to_string()
        } else {
            "✘".to_string()
        };

        if self.job_type_filter.intersects(JobTypeFilter::DISABLED) {
            loaded = format!("{} disabled", loaded)
        }

        vec![
            self.name.clone(),
            session_type,
            entry_type,
            pid,
            loaded.to_string(),
        ]
    }
}
