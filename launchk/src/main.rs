#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cursive;

#[macro_use]
extern crate bitflags;

extern crate plist;

use crate::tui::root::RootLayout;
use cursive::view::Resizable;
use cursive::views::Panel;
use cursive::Cursive;

mod launchd;
mod tui;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut siv: Cursive = cursive::default();
    siv.load_toml(include_str!("tui/style.toml")).unwrap();

    let mut root_layout = RootLayout::new();
    root_layout.setup(&mut siv, runtime.handle().clone());

    let panel = Panel::new(root_layout)
        .title("launchk")
        .full_width()
        .full_height();

    siv.add_layer(panel);
    siv.run();
}
