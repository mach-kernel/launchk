use cursive::view::{ViewWrapper, Selector};

use cursive::{Cursive, View, XY, Printer};

use std::collections::HashSet;

use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use std::time::Duration;
use tokio::runtime::Handle;

use tokio::time::interval;

use crate::tui::omnibox::OmniboxCommand;
use crate::tui::omnibox_subscribed_view::OmniboxSubscriber;
use crate::tui::root::CbSinkMessage;

use crate::launchd::service::{find_entry_info, list_all, LaunchdEntryInfo};
use crate::tui::table::table_list_view::{TableListItem, TableListView};
use std::borrow::Borrow;
use std::cell::RefCell;
use cursive::event::{EventResult, Event};

use std::fmt;
use std::fmt::Formatter;
use crate::tui::job_type_filter::JobTypeFilter;
use std::prelude::v1::Result::Err;

async fn poll_services(svcs: Arc<RwLock<HashSet<String>>>, cb_sink: Sender<CbSinkMessage>) {
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

pub struct ServiceListItem {
    name: String,
    entry_info: LaunchdEntryInfo,
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

        vec![
            self.name.clone(),
            session_type,
            entry_type,
            format!("{}", self.entry_info.pid),
        ]
    }
}

pub struct ServiceListView {
    services: Arc<RwLock<HashSet<String>>>,
    table_list_view: TableListView<ServiceListItem>,
    name_filter: RefCell<String>,
    job_type_filter: RefCell<JobTypeFilter>,
}

impl ServiceListView {
    pub fn new(runtime_handle: &Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let arc_svc = Arc::new(RwLock::new(HashSet::new()));
        let ref_clone = arc_svc.clone();

        runtime_handle.spawn(async move { poll_services(ref_clone, cb_sink).await });

        Self {
            services: arc_svc.clone(),
            name_filter: RefCell::new("".into()),
            job_type_filter: RefCell::new(JobTypeFilter::default()),
            table_list_view: TableListView::new(vec![
                "Name".to_string(),
                "Session Type".to_string(),
                "Job Type".to_string(),
                "PID".to_string(),
            ]),
        }
    }

    fn present_services(&self) -> Vec<ServiceListItem> {
        let services = self.services.try_read();

        if services.is_err() {
            return vec![];
        }

        let services = services.unwrap();
        let name_filter = self.name_filter.borrow();
        let job_type_filter = self.job_type_filter.borrow();

        let mut items: Vec<ServiceListItem> = services
            .iter()
            .filter_map(|s| {
                if !name_filter.is_empty()
                    && !s
                        .to_ascii_lowercase()
                        .contains(name_filter.to_ascii_lowercase().as_str())
                {
                    return None;
                }

                let entry_info = find_entry_info(s);

                if !job_type_filter.is_empty() &&
                    entry_info.entry_config
                        .as_ref()
                        .map(|ec| !job_type_filter.contains(ec.job_type_filter()))
                        .unwrap_or(true) {
                    return None;
                }

                Some(ServiceListItem {
                    name: s.clone(),
                    entry_info,
                })
            })
            .collect();

        items.sort_by(|a, b| a.name.cmp(&b.name));
        items
    }
}

impl ViewWrapper for ServiceListView {
    wrap_impl!(self.table_list_view: TableListView<ServiceListItem>);

    fn wrap_layout(&mut self, size: XY<usize>) {
        let sorted = self.present_services();
        self.with_view_mut(|v| v.replace_and_preserve_selection(sorted));
        self.table_list_view.layout(size);
    }
}

impl OmniboxSubscriber for ServiceListView {
    fn on_omnibox(&mut self, cmd: OmniboxCommand) -> Result<(), ()> {
        match cmd {
            OmniboxCommand::NameFilter(new_filter) => {
                self.name_filter.replace(new_filter);
            }
            OmniboxCommand::JobTypeFilter(new_filter) => {
                self.job_type_filter.replace(new_filter);
            }
            OmniboxCommand::Clear => {
                self.name_filter.replace("".to_string());
                self.job_type_filter.replace(JobTypeFilter::default());
            }
            _ => (),
        };

        Ok(())
    }
}
