#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cursive;

#[macro_use]
extern crate bitflags;

extern crate plist;

use cursive::view::{Resizable, AnyView};
use cursive::views::{Panel, NamedView};
use cursive::Cursive;
use std::process::exit;

use crate::launchd::plist::{init_plist_map, PLIST_MAP_INIT};
use crate::tui::root::RootLayout;

mod launchd;
mod tui;

fn main() {
    env_logger::init();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Must build tokio runtime");

    // Cache launchd job plist paths, spawn fsnotify to keep up with changes
    PLIST_MAP_INIT.call_once(|| init_plist_map(runtime.handle()));

    let mut siv: Cursive = cursive::default();
    siv.load_toml(include_str!("tui/style.toml"))
        .expect("Must load styles");
    
    let root_layout = RootLayout::new(&mut siv, runtime.handle());
    let root_layout = NamedView::new("root_layout", root_layout);

    let panel = Panel::new(root_layout)
        .title("launchk")
        .full_width()
        .full_height();

    siv.add_layer(panel);
    siv.run();

    exit(0);
}
