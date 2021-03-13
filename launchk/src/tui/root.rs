use cursive::view::ViewWrapper;
use cursive::views::LinearLayout;
use cursive::{View, Cursive};
use tokio::runtime::Handle;
use tokio::time::interval;
use std::sync::Once;
use std::sync::mpsc::{Sender, Receiver, channel};
use std::time::Duration;
use crate::tui::service::ServiceView;

pub type CbSinkMessage = Box<dyn FnOnce(&mut Cursive) + Send>;

pub struct RootLayout {
    layout: LinearLayout
}

impl RootLayout {
    pub fn new() -> Self {
        Self { layout: LinearLayout::vertical() }
    }

    pub fn setup(&mut self, siv: &mut Cursive, handle: Handle) {
        self.with_view_mut(|v| {
            let tx = RootLayout::cbsink_channel(siv, &handle);
            let sl = ServiceView::new(handle, tx.clone());
            v.add_child(sl);
        });
    }

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
}

impl ViewWrapper for RootLayout {
    wrap_impl!(self.layout: LinearLayout);
}