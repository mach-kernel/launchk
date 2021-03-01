use cursive::view::{Resizable, Scrollable};
use cursive::views::{DummyView, LinearLayout, Panel, SelectView};
use cursive::Cursive;
use std::collections::HashMap;
use xpc_sys::object::xpc_object::XPCObject;

pub fn list_services(siv: &mut Cursive, services: &HashMap<String, XPCObject>) {
    let mut layout = LinearLayout::vertical();
    let mut sv: SelectView<XPCObject> = SelectView::new();

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

    siv.add_layer(layout);
}
