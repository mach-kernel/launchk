mod service;

use crate::tui::service::ServiceView;
use cursive::view::{Resizable, Scrollable};
use cursive::views::{Dialog, DummyView, LinearLayout, Panel, SelectView};
use cursive::Cursive;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use tokio::runtime::{Handle};
use tokio::time::interval;

pub type CbSinkMessage = Box<dyn FnOnce(&mut Cursive) + Send>;

/// Cursive uses a different crate for its channel, so this is some glue
pub fn cbsink_channel(siv: &mut Cursive, handle: &Handle) -> Sender<CbSinkMessage> {
    let (tx, rx): (Sender<CbSinkMessage>, Receiver<CbSinkMessage>) = channel();
    let sink = siv.cb_sink().clone();

    handle.spawn(async move {
        let mut interval = interval(Duration::from_millis(500));

        loop {
            interval.tick().await;

            if let Ok(cb_sink_msg) = rx.recv() {
                sink.send(cb_sink_msg).unwrap();
            }
        }
    });

    tx.clone()
}

pub fn list_services(siv: &mut Cursive, handle: Handle) {
    let mut layout = LinearLayout::vertical();
    let tx = cbsink_channel(siv, &handle);
    let sl = ServiceView::new(handle, tx.clone());

    layout.add_child(
        Panel::new(sl)
            .title("launchk")
            .full_height()
            .full_width()
            .scrollable(),
    );

    siv.add_layer(layout);
}
