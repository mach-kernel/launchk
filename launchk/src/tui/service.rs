use crate::launchd;
use crate::launchd::messages::from_msg;
use crate::tui::CbSinkMessage;

use cursive::view::ViewWrapper;
use cursive::views::SelectView;
use cursive::{Cursive, Printer, Vec2, View};

use std::collections::{HashMap};
use std::convert::TryInto;

use std::sync::mpsc::Sender;
use std::sync::{Arc, RwLock};

use std::time::{Duration, SystemTime};
use tokio::runtime::Handle;

use tokio::time::interval;
use xpc_sys::objects::xpc_dictionary::XPCDictionary;
use xpc_sys::objects::xpc_error::XPCError::StandardError;
use xpc_sys::objects::xpc_object::XPCObject;
use xpc_sys::traits::xpc_pipeable::XPCPipeable;

async fn poll_services(
    svcs: Arc<RwLock<HashMap<String, XPCObject>>>,
    cb_sink: Sender<CbSinkMessage>,
) -> () {
    // launchctl list
    let message: HashMap<&str, XPCObject> = from_msg(&launchd::messages::LIST_SERVICES);
    let mut interval = interval(Duration::from_secs(5));

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

pub struct ServiceView {
    services: Arc<RwLock<HashMap<String, XPCObject>>>,
    select_view: SelectView<XPCObject>,
}

impl ServiceView {
    pub fn new(runtime_handle: Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let ref_svc = Arc::new(RwLock::new(HashMap::new()));
        let ref_clone = ref_svc.clone();

        runtime_handle.spawn(async move {
            poll_services(ref_clone, cb_sink).await;
        });

        Self {
            services: ref_svc.clone(),
            select_view: SelectView::new(),
        }
    }
}

impl ViewWrapper for ServiceView {
    wrap_impl!(self.select_view: SelectView<XPCObject>);

    fn wrap_layout(&mut self, _size: Vec2) {
        let clone = self.services.clone();

        self.with_view_mut(move |v| {
            v.clear();

            // TODO
            let tsnow = format!("{:?}", SystemTime::now());
            v.add_item(tsnow, XPCObject::default());

            let read = clone.try_read();
            if read.is_err() {
                v.add_item("Loading...", XPCObject::default());
            }

            let read = read.unwrap();
            let mut vec: Vec<(&String, &XPCObject)> = read.iter().collect();
            vec.sort_by(|(k, _), (k1, _)| k.cmp(k1));

            for (name, xpc_object) in vec {
                v.add_item(name.clone(), xpc_object.clone());
            }
        })
        .unwrap();
    }
}
