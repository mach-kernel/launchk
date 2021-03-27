use crate::launchd;
use crate::launchd::messages::from_msg;

use cursive::view::ViewWrapper;

use cursive::{Cursive, View, XY};

use std::collections::HashMap;
use std::convert::TryInto;

use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use std::time::Duration;
use tokio::runtime::Handle;

use tokio::time::interval;
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_error::XPCError::StandardError;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::traits::xpc_pipeable::XPCPipeable;

use xpc_sys::traits::xpc_value::TryXPCValue;

use crate::tui::omnibox::OmniboxCommand;
use crate::tui::omnibox_subscribed_view::OmniboxSubscriber;
use crate::tui::root::CbSinkMessage;

use crate::launchd::config::{find_unit, LaunchdEntryConfig};
use crate::tui::table_list_view::{TableListItem, TableListView};
use std::borrow::Borrow;
use std::cell::RefCell;

async fn poll_services(
    svcs: Arc<RwLock<HashMap<String, XPCObject>>>,
    cb_sink: Sender<CbSinkMessage>,
) -> () {
    // launchctl list
    let message: HashMap<&str, XPCObject> = from_msg(&launchd::messages::LIST_SERVICES);
    let mut interval = interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        let msg_list: XPCObject = message.clone().into();
        let services = msg_list
            .pipe_routine()
            .and_then(|r| r.try_into())
            .and_then(|XPCDictionary(ref hm)| {
                hm.get("services").map(|s| s.clone()).ok_or(StandardError)
            })
            .and_then(|s| s.try_into())
            .and_then(|XPCDictionary(ref hm)| Ok(hm.clone()));

        if services.is_err() {
            continue;
        }

        let svc_write = svcs.try_write();
        if svc_write.is_err() {
            continue;
        }
        *svc_write.unwrap() = services.unwrap();

        cb_sink.send(Box::new(Cursive::noop)).unwrap();
    }
}

pub struct ServiceListItem {
    name: String,
    pid: i64,
    entry_config: Option<LaunchdEntryConfig>,
}

impl TableListItem for ServiceListItem {
    fn as_row(&self) -> Vec<String> {
        let session_type = self
            .entry_config
            .borrow()
            .as_ref()
            .map(|ec| format!("{:?}", ec.session_type))
            .unwrap_or("-".to_string());

        let entry_type = self
            .entry_config
            .borrow()
            .as_ref()
            .map(|ec| format!("{:?}", ec.entry_type))
            .unwrap_or("-".to_string());

        vec![
            self.name.clone(),
            session_type,
            entry_type,
            format!("{}", self.pid),
        ]
    }
}

pub struct ServiceListView {
    services: Arc<RwLock<HashMap<String, XPCObject>>>,
    table_list_view: TableListView<ServiceListItem>,
    current_filter: RefCell<String>,
}

impl ServiceListView {
    pub fn new(runtime_handle: Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let arc_svc = Arc::new(RwLock::new(HashMap::new()));
        let ref_clone = arc_svc.clone();

        runtime_handle.spawn(async move { poll_services(ref_clone, cb_sink).await });

        Self {
            services: arc_svc.clone(),
            current_filter: RefCell::new("".into()),
            table_list_view: TableListView::new(vec![
                "Name".to_string(),
                "Session Type".to_string(),
                "Kind".to_string(),
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

        let mut vec: Vec<ServiceListItem> = services
            .iter()
            .filter_map(|(s, o)| {
                if !filter.is_empty()
                    && !s
                        .to_ascii_lowercase()
                        .contains(filter.to_ascii_lowercase().as_str())
                {
                    return None;
                }

                let XPCDictionary(hm) = o.try_into().unwrap();
                let pid: i64 = hm.get("pid").and_then(|p| p.xpc_value().ok()).unwrap();

                Some(ServiceListItem {
                    name: s.clone(),
                    pid,
                    entry_config: find_unit(s),
                })
            })
            .collect();

        vec.sort_by(|a, b| a.name.cmp(&b.name));
        vec
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
