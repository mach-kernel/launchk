#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate cursive;

use crate::tui::list_services;

mod launchd;
mod tui;

fn main() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let mut siv = cursive::default();
    list_services(&mut siv, runtime.handle().clone());
    runtime.block_on(async { siv.run() });
}
