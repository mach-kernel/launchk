mod service_list;

use cursive::view::{Resizable, Scrollable};
use cursive::views::{DummyView, LinearLayout, Panel, SelectView, Dialog};
use cursive::Cursive;
use std::collections::HashMap;
use xpc_sys::objects::xpc_object::XPCObject;
use crate::tui::service_list::{ServiceListItem, ServiceListView};

pub fn list_services(siv: &mut Cursive, services: &HashMap<String, XPCObject>) {
    let mut layout = LinearLayout::vertical();
    let mut sv: SelectView<XPCObject> = SelectView::<XPCObject>::new().on_submit(|s, item| {
        let mut d = Dialog::text(item.to_string());
        d.add_button("OK", |s| { s.pop_layer(); });
        s.add_layer(d);
    });

    for (name, obj) in services {
        sv.add_item(name, obj.clone())
    }

    layout.add_child(
        Panel::new(DummyView)
            .title("launchk")
            .full_width()
            .min_height(5),
    );

    layout.add_child(
        Panel::new(sv.scrollable())
            .title("services")
            .full_width()
            .full_height(),
    );


    // let items = services.iter().map(|(name, obj)| ServiceListItem {
    //     name: name.clone(),
    //     object: obj.clone()
    // });
    // layout.add_child(ServiceListView::new(items).full_width().full_height().scrollable());

    siv.add_layer(layout);
}
