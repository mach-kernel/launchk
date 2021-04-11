use cursive::view::ViewWrapper;

use cursive::{Cursive, View, XY};

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
use crate::tui::table_list_view::{TableListItem, TableListView};
use std::borrow::Borrow;
use std::cell::RefCell;

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
            .map(|ec| ec.entry_type.to_string())
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
    current_filter: RefCell<String>,
}

impl ServiceListView {
    pub fn new(runtime_handle: Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let arc_svc = Arc::new(RwLock::new(HashSet::new()));
        let ref_clone = arc_svc.clone();

        runtime_handle.spawn(async move { poll_services(ref_clone, cb_sink).await });

        Self {
            services: arc_svc.clone(),
            current_filter: RefCell::new("".into()),
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
        let filter = self.current_filter.borrow();

        let mut items: Vec<ServiceListItem> = services
            .iter()
            .filter_map(|s| {
                if !filter.is_empty()
                    && !s
                        .to_ascii_lowercase()
                        .contains(filter.to_ascii_lowercase().as_str())
                {
                    return None;
                }

                Some(ServiceListItem {
                    name: s.clone(),
                    entry_info: find_entry_info(s),
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
            OmniboxCommand::Filter(new_filter) => {
                self.current_filter.replace(new_filter);
            }
            OmniboxCommand::Clear => {
                self.current_filter.replace("".to_string());
            }
            _ => (),
        };

        Ok(())
    }
}
