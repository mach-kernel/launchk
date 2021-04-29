use std::borrow::Borrow;

use crate::launchd::job_type_filter::JobTypeFilter;
use crate::tui::table::table_list_view::TableListItem;
use crate::launchd::query::LaunchdEntryInfo;

#[derive(Debug)]
pub struct ServiceListItem {
    pub name: String,
    pub entry_info: LaunchdEntryInfo,
    pub job_type_filter: JobTypeFilter,
}

impl TableListItem for ServiceListItem {
    fn as_row(&self) -> Vec<String> {
        let session_type = self.entry_info.limit_load_to_session_type.to_string();

        let entry_type = self
            .entry_info
            .entry_config
            .borrow()
            .as_ref()
            .map(|ec| format!("{}/{}", ec.entry_location, ec.entry_type))
            .unwrap_or("-".to_string());

        let pid =
            if self.entry_info.pid > 0 && self.job_type_filter.intersects(JobTypeFilter::LOADED) {
                format!("{}", self.entry_info.pid)
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