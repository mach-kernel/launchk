#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cursive;

use cursive::views::{Panel, LinearLayout};
use cursive::view::{Resizable, Scrollable};
use crate::tui::root::RootLayout;

mod launchd;
mod tui;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut siv = cursive::default();

    let mut root_layout = RootLayout::new();
    root_layout.setup(&mut siv, runtime.handle().clone());

    let panel = Panel::new(root_layout)
        .title("launchk")
        .full_width()
        .full_height()
        .scrollable();

    siv.add_layer(panel);
    runtime.block_on(async { siv.run() });
}
