use std::borrow::Borrow;

use crate::launchd::entry_status::LaunchdEntryStatus;
use crate::launchd::job_type_filter::JobTypeFilter;
use crate::tui::table::table_list_view::TableListItem;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct ServiceListItem {
    pub name: String,
    pub status: LaunchdEntryStatus,
    pub job_type_filter: JobTypeFilter,
}

impl TableListItem for ServiceListItem {
    fn as_row(&self) -> Vec<String> {
        let session_type = self.status.limit_load_to_session_type.to_string();

        let entry_type = self
            .status
            .plist
            .borrow()
            .as_ref()
            .map(|ec| format!("{}/{}", ec.entry_location, ec.entry_type))
            .unwrap_or("-".to_string());

        let pid = if self.status.pid > 0 && self.job_type_filter.intersects(JobTypeFilter::LOADED) {
            format!("{}", self.status.pid)
        } else {
            "-".to_string()
        };

        let loaded = if self.job_type_filter.intersects(JobTypeFilter::LOADED) {
            "✔"
        } else {
            "✘"
        };

        vec![
            self.name.clone(),
            session_type,
            entry_type,
            pid,
            loaded.to_string(),
        ]
    }
}
