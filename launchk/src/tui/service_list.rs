use cursive::view::ViewWrapper;

use cursive::{Cursive, View, XY};

use std::collections::HashSet;

use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use std::time::Duration;
use tokio::runtime::Handle;

use tokio::time::interval;

use crate::tui::omnibox::{OmniboxEvent, OmniboxState, OmniboxMode, OmniboxCommand, OmniboxError};
use crate::tui::omnibox_subscribed_view::OmniboxSubscriber;
use crate::tui::root::CbSinkMessage;
use crate::launchd::config::{LABEL_TO_ENTRY_CONFIG};

use crate::launchd::service::{find_entry_info, list_all, LaunchdEntryInfo};
use crate::tui::table::table_list_view::{TableListItem, TableListView};
use std::borrow::Borrow;
use std::cell::RefCell;

use crate::tui::job_type_filter::JobTypeFilter;
use cursive::direction::Direction;
use std::process::Command;

async fn poll_running_jobs(svcs: Arc<RwLock<HashSet<String>>>, cb_sink: Sender<CbSinkMessage>) {
    let mut interval = interval(Duration::from_secs(1));

    loop {
        interval.tick().await;
        let write = svcs.try_write();

        if write.is_err() {
            continue;
        }

        let mut write = write.unwrap();
        *write = list_all();

        cb_sink.send(Box::new(Cursive::noop)).unwrap();
    }
}

#[derive(Debug)]
pub struct ServiceListItem {
    name: String,
    entry_info: LaunchdEntryInfo,
    job_type_filter: JobTypeFilter,
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

        let pid = if self.entry_info.pid > 0 {
            format!("{}", self.entry_info.pid)
        } else {
            "-".to_string()
        };

        // TODO: This would be cute and stuff, but truncate() panics
        let loaded = if self.job_type_filter.intersects(JobTypeFilter::LOADED) {
            "y"
            // "✓"
        } else {
            "n"
            // "✗"
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

pub struct ServiceListView {
    running_jobs: Arc<RwLock<HashSet<String>>>,
    table_list_view: TableListView<ServiceListItem>,
    name_filter: RefCell<String>,
    job_type_filter: RefCell<JobTypeFilter>,
}

impl ServiceListView {
    pub fn new(runtime_handle: &Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let arc_svc = Arc::new(RwLock::new(HashSet::new()));
        let ref_clone = arc_svc.clone();

        runtime_handle.spawn(async move { poll_running_jobs(ref_clone, cb_sink).await });

        Self {
            running_jobs: arc_svc.clone(),
            name_filter: RefCell::new("".into()),
            job_type_filter: RefCell::new(JobTypeFilter::default()),
            table_list_view: TableListView::new(vec![
                ("Name", None),
                ("Session", Some(12)),
                ("Job Type", Some(14)),
                ("PID", Some(6)),
                ("Loaded", Some(6)),
            ]),
        }
    }

    fn present_services(&self) -> Option<Vec<ServiceListItem>> {
        let plists = LABEL_TO_ENTRY_CONFIG.read().ok()?;
        let running = self.running_jobs.read().ok()?;

        let name_filter = self.name_filter.borrow();
        let job_type_filter = self.job_type_filter.borrow();

        let running_no_plist = running
            .iter()
            .filter(|r| !plists.contains_key(*r));

        let mut items: Vec<ServiceListItem> = plists
            .keys()
            .into_iter()
            .chain(running_no_plist)
            .filter_map(|label| {
                if !name_filter.is_empty()
                    && !label
                        .to_ascii_lowercase()
                        .contains(name_filter.to_ascii_lowercase().as_str())
                {
                    return None;
                }

                let entry_info = find_entry_info(label);
                let is_loaded = running.contains(label);

                let entry_job_type_filter = entry_info
                    .entry_config
                    .as_ref()
                    .map(|ec| ec.job_type_filter(is_loaded))
                    .unwrap_or(JobTypeFilter::empty());

                if !job_type_filter.is_empty() && !entry_job_type_filter.intersects(*job_type_filter)
                {
                    return None;
                }

                Some(ServiceListItem {
                    name: label.clone(),
                    entry_info,
                    job_type_filter: entry_job_type_filter,
                })
            })
            .collect();

        // TODO: Place unsorted jobs on top, to avoid having an extra modal toggle for loading
        items.sort_by(|a, b| a.name.cmp(&b.name));
        Some(items)
    }

    fn handle_state_update(&mut self, state: OmniboxState) -> Result<(), OmniboxError> {
        let OmniboxState {
            mode,
            name_filter,
            job_type_filter,
            ..
        } = state;

        match mode {
            OmniboxMode::NameFilter => { self.name_filter.replace(name_filter); },
            OmniboxMode::JobTypeFilter => { self.job_type_filter.replace(job_type_filter); },
            OmniboxMode::Idle => {
                self.name_filter.replace(name_filter);
                self.job_type_filter.replace(job_type_filter);
            }
            _ => {},
        };

        Ok(())
    }

    fn handle_command(&mut self, cmd: OmniboxCommand) -> Result<(), OmniboxError> {
        match cmd {
            OmniboxCommand::Edit => {
                let entry_config = self
                    .table_list_view
                    .get_highlighted_row()
                    .and_then(|rc| rc.entry_info.entry_config.clone())
                    .ok_or(OmniboxError::CommandError("Cannot find plist for entry".to_string()))?;

                if entry_config.readonly {
                    Err(OmniboxError::CommandError(format!("{} is read-only", entry_config.plist_path)))
                } else {
                    let vim = Command::new("vim")
                        .arg(entry_config.plist_path)
                        .status()
                        .expect("Must get status");

                    println!("vim exited {}", vim);
                    Ok(())
                }
            }
            _ => Ok(())
        }
    }
}

impl ViewWrapper for ServiceListView {
    wrap_impl!(self.table_list_view: TableListView<ServiceListItem>);

    fn wrap_layout(&mut self, size: XY<usize>) {
        self.table_list_view.layout(size);

        if let Some(sorted) = self.present_services() {
            self.with_view_mut(|v| v.replace_and_preserve_selection(sorted));
        }
    }

    fn wrap_take_focus(&mut self, _: Direction) -> bool {
        true
    }
}

impl OmniboxSubscriber for ServiceListView {
    fn on_omnibox(&mut self, event: OmniboxEvent) -> Result<(), OmniboxError> {
        match event {
            OmniboxEvent::StateUpdate(state) => self.handle_state_update(state),
            OmniboxEvent::Command(cmd) => self.handle_command(cmd),
            _ => Ok(())
        }
    }
}
