use std::cmp::Ordering;
use std::collections::HashSet;
use std::ptr::slice_from_raw_parts;
use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use cursive::direction::Direction;
use cursive::event::EventResult;
use cursive::view::CannotFocus;
use cursive::view::ViewWrapper;
use cursive::{Cursive, CursiveExt, View, XY};
use sudo::RunningAs;

use tokio::runtime::Handle;
use tokio::time::interval;
use xpc_sys::enums::{DomainType, SessionType};

use crate::launchd::job_type_filter::JobTypeFilter;
use crate::launchd::plist::{edit_and_replace, LaunchdEntryLocation, LABEL_TO_ENTRY_CONFIG};
use crate::launchd::command::{bootout, bootstrap, procinfo};
use crate::launchd::command::{disable, enable, list_all, load};
use crate::launchd::{
    entry_status::get_entry_status, entry_status::LaunchdEntryStatus, plist::LaunchdPlist,
};
use crate::tui::omnibox::command::OmniboxCommand;

use crate::tui::omnibox::state::OmniboxState;
use crate::tui::omnibox::subscribed_view::{OmniboxResult, OmniboxSubscriber};
use crate::tui::omnibox::view::{OmniboxError, OmniboxEvent, OmniboxMode};
use crate::tui::pager::show_pager;
use crate::tui::root::CbSinkMessage;
use crate::tui::service_list::list_item::ServiceListItem;
use crate::tui::table::table_list_view::TableListView;

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

        cb_sink.send(Box::new(Cursive::noop)).expect("Must send");
    }
}

pub struct ServiceListView {
    cb_sink: Sender<CbSinkMessage>,
    running_jobs: Arc<RwLock<HashSet<String>>>,
    table_list_view: TableListView<ServiceListItem>,
    label_filter: Arc<RwLock<String>>,
    job_type_filter: Arc<RwLock<JobTypeFilter>>,
}

enum ServiceListError {
    PresentationError
}

impl ServiceListView {
    pub fn new(runtime_handle: &Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let arc_svc = Arc::new(RwLock::new(HashSet::new()));
        runtime_handle.spawn(poll_running_jobs(arc_svc.clone(), cb_sink.clone()));

        Self {
            cb_sink,
            running_jobs: arc_svc.clone(),
            label_filter: Arc::new(RwLock::new("".into())),
            job_type_filter: Arc::new(RwLock::new(JobTypeFilter::launchk_default())),
            table_list_view: TableListView::new(vec![
                ("Name", None),
                ("Session", Some(12)),
                ("Job Type", Some(14)),
                ("PID", Some(6)),
                ("Loaded", Some(6)),
            ]),
        }
    }

    fn present_services(&self) -> Result<Vec<ServiceListItem>, ServiceListError> {
        let plists = LABEL_TO_ENTRY_CONFIG
            .read()
            .map_err(|_| ServiceListError::PresentationError)?;
        let running = self.running_jobs
            .read()
            .map_err(|_| ServiceListError::PresentationError)?;

        let name_filter = self.label_filter
            .read()
            .map_err(|_| ServiceListError::PresentationError)?;
        let job_type_filter = self.job_type_filter
            .read()
            .map_err(|_| ServiceListError::PresentationError)?;

        let running_no_plist = running.iter().filter(|r| !plists.contains_key(*r));

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

                let status = get_entry_status(label);
                let is_loaded = running.contains(label);

                let entry_job_type_filter = status
                    .plist
                    .as_ref()
                    .map(|ec| ec.job_type_filter(is_loaded))
                    .unwrap_or(if is_loaded {
                        JobTypeFilter::LOADED
                    } else {
                        JobTypeFilter::default()
                    });

                if !job_type_filter.is_empty() && !entry_job_type_filter.contains(*job_type_filter)
                {
                    return None;
                }

                Some(ServiceListItem {
                    status,
                    name: label.clone(),
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

        Ok(items)
    }

    fn handle_state_update(&mut self, state: OmniboxState) -> OmniboxResult {
        let OmniboxState {
            mode,
            label_filter,
            job_type_filter,
            ..
        } = state;

        match mode {
            OmniboxMode::LabelFilter => {
                let mut view_filter =
                    self.label_filter.try_write().map_err(|_| OmniboxError::StateError)?;
                *view_filter = label_filter;
            }
            OmniboxMode::JobTypeFilter => {
                let mut view_job_type_filter =
                    self.job_type_filter.try_write().map_err(|_| OmniboxError::StateError)?;
                *view_job_type_filter = job_type_filter;
            }
            OmniboxMode::Idle => {
                let mut view_filter =
                    self.label_filter.try_write().map_err(|_| OmniboxError::StateError)?;
                *view_filter = label_filter;

                let mut view_job_type_filter =
                    self.job_type_filter.try_write().map_err(|_| OmniboxError::StateError)?;
                *view_job_type_filter = job_type_filter;
            }
            _ => {}
        };

        Ok(None)
    }

    fn get_active_list_item(&self) -> Result<Arc<ServiceListItem>, OmniboxError> {
        self.table_list_view
            .get_highlighted_row()
            .ok_or_else(|| OmniboxError::CommandError("Cannot get highlighted row".to_string()))
    }

    fn with_active_item_plist(
        &self,
    ) -> Result<(ServiceListItem, Option<LaunchdPlist>), OmniboxError> {
        let item = &*self.get_active_list_item()?;
        let plist = item.status.plist.clone();

        Ok((item.clone(), plist))
    }

    fn handle_plist_command(&self, cmd: OmniboxCommand) -> OmniboxResult {
        let (ServiceListItem { name, status, .. }, plist) = self.with_active_item_plist()?;

        let plist =
            plist.ok_or_else(|| OmniboxError::CommandError("Cannot find plist".to_string()))?;

        match cmd {
            OmniboxCommand::Edit => {
                let edited =
                    edit_and_replace(&plist).map_err(OmniboxError::CommandError);

                // Reinit curses
                self.cb_sink
                    .send(Box::new(|siv: &mut Cursive| siv.run()))
                    .expect("Must clear");

                edited?;

                Ok(Some(OmniboxCommand::Confirm(
                    format!("Reload {}?", name),
                    vec![OmniboxCommand::Reload],
                )))
            }
            OmniboxCommand::Load(st, dt, _handle) => {
                load(name, plist.plist_path, Some(dt), Some(st), None)
                    .map(|_| None)
                    .map_err(|e| OmniboxError::CommandError(e.to_string()))
            }
            OmniboxCommand::Bootstrap(dt) => {
                bootstrap(name, dt, plist.plist_path)
                    .map(|_| None)
                    .map_err(|e| OmniboxError::CommandError(e.to_string()))
            }
            OmniboxCommand::Bootout(dt) => {
                bootout(name, dt)
                    .map(|_| None)
                    .map_err(|e| OmniboxError::CommandError(e.to_string()))
            }
            _ => Ok(None),
        }
    }

    fn handle_command(&self, cmd: OmniboxCommand) -> OmniboxResult {
        let (ServiceListItem { name, status, .. }, plist) = self.with_active_item_plist()?;

        let need_escalate = plist
            .map(|p| p.entry_location.into())
            .map(|d: DomainType| d == DomainType::System)
            .unwrap_or(false);

        match cmd {
            | OmniboxCommand::DisableRequest
            | OmniboxCommand::EnableRequest
            | OmniboxCommand::ProcInfo
            | OmniboxCommand::BootstrapRequest
            | OmniboxCommand::BootoutRequest
            | OmniboxCommand::Edit => {
                if (sudo::check() != RunningAs::Root) && need_escalate {
                    return Ok(Some(OmniboxCommand::Confirm(
                        "This requires root privileges. Sudo and restart?".to_string(),
                        vec![OmniboxCommand::Quit, OmniboxCommand::Sudo],
                    )));
                }
            }
            _ => (),
        };

        match cmd {
            OmniboxCommand::Reload => {
                let LaunchdEntryStatus {
                    limit_load_to_session_type,
                    domain,
                    ..
                } = status;

                match (limit_load_to_session_type, domain) {
                    (_, DomainType::Unknown) | (SessionType::Unknown, _) => Ok(Some(
                        OmniboxCommand::DomainSessionPrompt(name.clone(), false, |dt, st| {
                            vec![
                                OmniboxCommand::Unload(dt.clone(), None),
                                OmniboxCommand::Load(st.expect("Must provide"), dt, None),
                            ]
                        }),
                    )),
                    (st, dt) => Ok(Some(OmniboxCommand::Chain(vec![
                        OmniboxCommand::Unload(dt.clone(), None),
                        OmniboxCommand::Load(st.clone(), dt.clone(), None),
                    ]))),
                }
            }
            OmniboxCommand::BootstrapRequest => {
                let LaunchdEntryStatus { domain, .. } = status;
                Ok(Some(OmniboxCommand::Bootstrap(domain)))
            }
            OmniboxCommand::BootoutRequest => {
                let LaunchdEntryStatus { domain, .. } = status;
                Ok(Some(OmniboxCommand::Bootout(domain)))
            }
            OmniboxCommand::EnableRequest => Ok(Some(OmniboxCommand::DomainSessionPrompt(
                name.clone(),
                true,
                |dt, _| vec![OmniboxCommand::Enable(dt)],
            ))),
            OmniboxCommand::DisableRequest => {
                let LaunchdEntryStatus { domain, .. } = status;

                match domain {
                    DomainType::Unknown => Ok(Some(OmniboxCommand::DomainSessionPrompt(
                        name.clone(),
                        true,
                        |dt, _| vec![OmniboxCommand::Disable(dt)],
                    ))),
                    _ => Ok(Some(OmniboxCommand::Chain(vec![OmniboxCommand::Disable(
                        domain,
                    )]))),
                }
            }
            OmniboxCommand::Enable(dt) => enable(name, dt)
                .map(|_| None)
                .map_err(|e| OmniboxError::CommandError(e.to_string())),
            OmniboxCommand::Disable(dt) => disable(name, dt)
                .map(|_| None)
                .map_err(|e| OmniboxError::CommandError(e.to_string())),
            OmniboxCommand::ProcInfo => {
                if status.pid == 0 {
                    return Err(OmniboxError::CommandError(format!("No PID for {}", name)));
                }
                let (size, shmem) =
                    procinfo(status.pid).map_err(|e| OmniboxError::CommandError(e.to_string()))?;

                show_pager(&self.cb_sink, unsafe {
                    &*slice_from_raw_parts(shmem.region as *mut u8, size)
                })
                .map_err(|e| OmniboxError::CommandError(e))?;

                Ok(None)
            }
            OmniboxCommand::Edit | OmniboxCommand::Load(_, _, _) | OmniboxCommand::Unload(_, _) | OmniboxCommand::Bootout(_) | OmniboxCommand::Bootstrap(_) => {
                self.handle_plist_command(cmd)
            }
            _ => Ok(None),
        }
    }
}

impl ViewWrapper for ServiceListView {
    wrap_impl!(self.table_list_view: TableListView<ServiceListItem>);

    fn wrap_layout(&mut self, size: XY<usize>) {
        self.table_list_view.layout(size);

        if let Ok(sorted) = self.present_services() {
            self.with_view_mut(|v| v.replace_and_preserve_selection(sorted));
        }
    }

    fn wrap_take_focus(&mut self, _: Direction) -> Result<EventResult, CannotFocus> {
        Ok(EventResult::Consumed(None))
    }
}

impl OmniboxSubscriber for ServiceListView {
    fn on_omnibox(&mut self, event: OmniboxEvent) -> OmniboxResult {
        match event {
            OmniboxEvent::StateUpdate(state) => self.handle_state_update(state),
            OmniboxEvent::Command(cmd) => self.handle_command(cmd),
        }
    }
}
