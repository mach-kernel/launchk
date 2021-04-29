#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cursive;

#[macro_use]
extern crate bitflags;

extern crate plist;

use crate::launchd::config::{init_label_map, LABEL_MAP_INIT};
use crate::tui::root::RootLayout;
use cursive::view::Resizable;
use cursive::views::Panel;
use cursive::Cursive;

mod launchd;
mod tui;

fn main() {
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Must build tokio runtime");

    // Cache launchd job plists, spawn fsnotify to keep up with changes
    LABEL_MAP_INIT.call_once(|| init_label_map(runtime.handle()));

    let mut siv: Cursive = cursive::default();
    siv.load_toml(include_str!("tui/style.toml"))
        .expect("Must load styles");

    let root_layout = RootLayout::new(&mut siv, runtime.handle());

    let panel = Panel::new(root_layout)
        .title("launchk")
        .full_width()
        .full_height();

    siv.add_layer(panel);
    siv.run();
}
