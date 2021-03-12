use crate::launchd;
use crate::launchd::messages::from_msg;
use crate::tui::CbSinkMessage;

use cursive::view::ViewWrapper;
use cursive::views::SelectView;
use cursive::{Cursive, Printer, View, XY};

use std::collections::HashMap;
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

use xpc_sys::traits::xpc_value::TryXPCValue;

use cursive::theme::{BaseColor, Color, Effect, Style};
use std::cell::Cell;

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

pub struct ServiceView {
    services: Arc<RwLock<HashMap<String, XPCObject>>>,
    select_view: SelectView<XPCObject>,
    current_size: Cell<XY<usize>>,
}

impl ServiceView {
    pub fn new(runtime_handle: Handle, cb_sink: Sender<CbSinkMessage>) -> Self {
        let ref_svc = Arc::new(RwLock::new(HashMap::new()));
        let ref_clone = ref_svc.clone();

        runtime_handle.spawn(async move {
            poll_services(ref_clone, cb_sink).await;
        });

        let select_view: SelectView<XPCObject> = SelectView::new().into();

        Self {
            services: ref_svc.clone(),
            current_size: Cell::new(XY::new(0, 0)),
            select_view,
        }
    }

    fn sorted_services(&self) -> Vec<(String, XPCObject)> {
        let read = self.services.try_read();
        if read.is_err() {
            return vec![];
        }

        let read = read.unwrap();
        let mut vec: Vec<(String, XPCObject)> = vec![];

        for (key, xpc_object) in read.iter() {
            vec.push((key.clone(), xpc_object.clone()));
        }

        vec.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));
        vec
    }
}

impl ViewWrapper for ServiceView {
    wrap_impl!(self.select_view: SelectView<XPCObject>);

    fn wrap_draw(&self, printer: &Printer<'_, '_>) {
        let middle = self.current_size.get().x / 2;
        let bold = Style::from(Color::Dark(BaseColor::Black)).combine(Effect::Bold);

        printer.with_style(bold, |p| p.print(XY::new(0, 0), "Name"));

        printer.with_style(bold, |p| p.print(XY::new(middle, 0), "PID"));

        let tsnow = format!("{:?}", SystemTime::now());
        printer.print(XY::new(0, 1), &tsnow);

        let sub = printer.offset(XY::new(0, 2));
        self.select_view.draw(&sub);
    }

    fn wrap_layout(&mut self, size: XY<usize>) {
        self.current_size.replace(size);
        let sorted = self.sorted_services();

        self.with_view_mut(move |v| {
            let current_selection = v.selected_id().unwrap_or(0);
            v.clear();

            for (name, xpc_object) in sorted.iter() {
                let XPCDictionary(hm) = xpc_object.try_into().unwrap();
                let pid: i64 = hm.get("pid").unwrap().xpc_value().unwrap();
                let pid_str = format!("{}", pid);

                let mut trunc_name = name.clone();
                let indent = size.x / 2;
                if trunc_name.chars().count() > indent {
                    trunc_name.truncate(indent - 1);
                }

                let row = format!("{:indent$}{}", trunc_name, pid_str, indent = indent);
                v.add_item(row, xpc_object.clone());
            }

            v.set_selection(current_selection);
        })
        .unwrap();
    }
}
