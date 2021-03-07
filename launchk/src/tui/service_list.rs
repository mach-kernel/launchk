use std::sync::{Mutex, RwLock};
use std::collections::BTreeMap;
use xpc_sys::objects::xpc_object::XPCObject;
use cursive::{View, Printer};
use cursive::direction::Orientation;
use std::iter::FromIterator;

pub struct ServiceListItem {
    pub name: String,
    pub object: XPCObject
}

pub struct ServiceListView {
    services: RwLock<Vec<ServiceListItem>>,
}

impl ServiceListView {
    pub fn new(services: impl Iterator<Item=ServiceListItem>) -> Self {
        ServiceListView {
            services: RwLock::new(Vec::from_iter(services))
        }
    }
}

impl Default for ServiceListView {
    fn default() -> Self {
        ServiceListView { services: RwLock::new(vec![]) }
    }
}

impl View for ServiceListView {
    fn draw(&self, printer: &Printer) {
        let read = self.services.try_read();
        if read.is_err() { return; }

        for (index, ServiceListItem { name, object }) in read.unwrap().iter().enumerate() {
            printer.print_line(Orientation::Horizontal, (0, index), name.len(), name.as_str());
        }
    }
}

