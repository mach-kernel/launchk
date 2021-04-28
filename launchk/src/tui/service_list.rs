use cursive::view::ViewWrapper;

use cursive::{Cursive, View, XY};

use std::collections::HashSet;
use std::thread;

use std::sync::mpsc::{Sender, channel, Receiver};
use std::sync::{Arc, RwLock, Mutex};

use std::time::Duration;
use tokio::runtime::Handle;

use tokio::time::interval;

use crate::tui::omnibox::view::{OmniboxEvent, OmniboxMode, OmniboxCommand, OmniboxError};
use crate::tui::omnibox::state::{OmniboxState};
use crate::tui::omnibox::subscribed_view::{OmniboxSubscriber, OmniboxResult};
use crate::tui::root::CbSinkMessage;
use crate::launchd::config::{LABEL_TO_ENTRY_CONFIG};

use crate::launchd::query::{find_entry_info, list_all, LaunchdEntryInfo, load, unload};
use crate::tui::table::table_list_view::{TableListItem, TableListView};
use std::borrow::Borrow;
use std::cell::RefCell;

use crate::tui::job_type_filter::JobTypeFilter;
use cursive::direction::Direction;
use std::process::Command;
use std::cmp::Ordering;
use std::rc::Rc;
use cursive::views::{Dialog, TextView};

lazy_static! {
    static ref EDITOR: &'static str = option_env!("EDITOR").unwrap_or("vim");
}

/// Polls XPC for job list
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

        let pid = if self.entry_info.pid > 0 && self.job_type_filter.intersects(JobTypeFilter::LOADED) {
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

pub struct ServiceListView {
    running_jobs: Arc<RwLock<HashSet<String>>>,
    table_list_view: TableListView<ServiceListItem>,
    label_filter: RefCell<String>,
    job_type_filter: RefCell<JobTypeFilter>,
    cb_sink: Sender<CbSinkMessage>,
}

impl ServiceListView {
    pub fn new(runtime_handle: &Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let arc_svc = Arc::new(RwLock::new(HashSet::new()));
        let ref_clone = arc_svc.clone();
        let cb_sink_clone = cb_sink.clone();

        runtime_handle.spawn(async move { poll_running_jobs(ref_clone, cb_sink_clone).await });

        Self {
            cb_sink,
            running_jobs: arc_svc.clone(),
            label_filter: RefCell::new("".into()),
            job_type_filter: RefCell::new(JobTypeFilter::launchk_default()),
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

        let name_filter = self.label_filter.borrow();
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
                    .unwrap_or(if is_loaded { JobTypeFilter::LOADED } else { JobTypeFilter::default() });

                if !job_type_filter.is_empty() && !entry_job_type_filter.contains(*job_type_filter)
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

        items.sort_by(|a, b| {
            let loaded_a = a.job_type_filter.intersects(JobTypeFilter::LOADED);
            let loaded_b = b.job_type_filter.intersects(JobTypeFilter::LOADED);
            let name_cmp = a.name.cmp(&b.name);

            if !loaded_a && loaded_b {
                Ordering::Less
            } else if loaded_a && !loaded_b {
                Ordering::Greater
            } else {
                name_cmp
            }
        });

        Some(items)
    }

    fn handle_state_update(&mut self, state: OmniboxState) -> OmniboxResult {
        let OmniboxState {
            mode,
            label_filter,
            job_type_filter,
            ..
        } = state;

        match mode {
            OmniboxMode::LabelFilter => { self.label_filter.replace(label_filter); }
            OmniboxMode::JobTypeFilter => { self.job_type_filter.replace(job_type_filter); },
            OmniboxMode::Idle => {
                self.label_filter.replace(label_filter);
                self.job_type_filter.replace(job_type_filter);
            }
            _ => {},
        };

        Ok(None)
    }

    fn get_active_list_item(&self) -> Result<Rc<ServiceListItem>, OmniboxError> {
        self.table_list_view
            .get_highlighted_row()
            .ok_or_else(|| OmniboxError::CommandError("Cannot get highlighted row".to_string()))
    }

    fn handle_command(&mut self, cmd: OmniboxCommand) -> OmniboxResult {
        match cmd {
            OmniboxCommand::Edit => {
                let ServiceListItem { name, entry_info, .. } =
                    &*self.get_active_list_item()?;

                let entry_config = entry_info
                    .entry_config
                    .as_ref()
                    .ok_or_else(|| OmniboxError::CommandError("Cannot find plist".to_string()))?;

                if entry_config.readonly {
                    return Err(OmniboxError::CommandError(format!("{} is read-only", entry_config.plist_path)))
                }

                let exit = Command::new(*EDITOR)
                    .arg(&entry_config.plist_path)
                    .status()
                    .map_err(|e| OmniboxError::CommandError(format!("{} failed: {}", *EDITOR, e.to_string())))?;

                self.cb_sink.send(Box::new(Cursive::clear)).expect("Must clear");

                if exit.success() {
                    Ok(Some(OmniboxCommand::Prompt(format!("Unload {}?", name), vec![OmniboxCommand::Unload, OmniboxCommand::Load])))
                } else {
                    Err(OmniboxError::CommandError(format!("{} didn't exit 0", *EDITOR)))
                }
            },
            OmniboxCommand::Load | OmniboxCommand::Unload => {
                let ServiceListItem { name, entry_info, .. } =
                    &*self.get_active_list_item()?;

                let entry_config = entry_info
                    .entry_config
                    .as_ref()
                    .ok_or_else(|| OmniboxError::CommandError("Cannot find plist".to_string()))?;

                let xpc_query = if cmd == OmniboxCommand::Load {
                    load
                } else {
                    unload
                };

                xpc_query(name, &entry_config.plist_path)
                    .map(|_| None)
                    .map_err(|e| OmniboxError::CommandError(e.to_string()))
            },
            _ => Ok(None)
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
    fn on_omnibox(&mut self, event: OmniboxEvent) -> OmniboxResult {
        match event {
            OmniboxEvent::StateUpdate(state) => self.handle_state_update(state),
            OmniboxEvent::Command(cmd) => self.handle_command(cmd),
            _ => Ok(None)
        }
    }
}
