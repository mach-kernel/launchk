pub mod service;

use crate::tui::service::ServiceView;
use cursive::view::{Resizable, Scrollable};
use cursive::views::{Dialog, DummyView, LinearLayout, Panel, SelectView, TextView};
use cursive::Cursive;

use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;
use tokio::runtime::{Handle};
use tokio::time::interval;
use cursive::utils::markup::StyledString;
use cursive::theme::{BaseColor, Effect, Color, Style};

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

    let mut playout = LinearLayout::vertical();
    let mut header = StyledString::new();
    header.append_styled("Name", Style::from(Color::Dark(BaseColor::Black)).combine(Effect::Bold));
    header.append("                                                                   ");
    header.append_styled("Launchd PID", Style::from(Color::Dark(BaseColor::Black)).combine(Effect::Bold));

    playout.add_child(TextView::new(header).max_height(1).full_width());

    playout.add_child(sl
        .full_height()
        .full_width()
        .scrollable()
    );

    layout.add_child(
        Panel::new(playout)
            .title("launchk")
            .full_height()
            .full_width()
            .scrollable(),
    );

    siv.add_layer(layout);
}