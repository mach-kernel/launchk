#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cursive;

#[macro_use]
extern crate bitflags;

extern crate plist;

use cursive::view::Resizable;
use cursive::views::{NamedView, Panel};
use git_version::git_version;
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

    let mut siv = cursive::default();
    siv.load_toml(include_str!("tui/style.toml"))
        .expect("Must load styles");

    let root_layout = RootLayout::new(&mut siv, runtime.handle());
    let root_layout = NamedView::new("root_layout", root_layout);

    let panel = Panel::new(root_layout)
        .title(format!("launchk ({})", git_version!()))
        .full_width()
        .full_height();

    siv.add_layer(panel);
    siv.run();
    siv.quit();

    // Fix reset on exit
    // https://github.com/gyscos/cursive/issues/415
    drop(siv);
    clearscreen::clear().expect("Clear");
    exit(0);
}
