use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, Panel};
use cursive::{Cursive, View};
use tokio::runtime::Handle;
use tokio::time::interval;

use crate::tui::omnibox::Omnibox;
use crate::tui::service::ServiceView;
use crate::tui::sysinfo::SysInfo;
use cursive::event::{Event, EventResult};
use cursive::traits::{Resizable, Scrollable};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

pub type CbSinkMessage = Box<dyn FnOnce(&mut Cursive) + Send>;

pub struct RootLayout {
    layout: LinearLayout,
}

enum RootLayoutChildren {
    SysInfo,
    Omnibox,
    ServiceList,
}

impl RootLayout {
    pub fn new() -> Self {
        Self {
            layout: LinearLayout::vertical(),
        }
    }

    pub fn setup(&mut self, siv: &mut Cursive, handle: Handle) {
        self.with_view_mut(|v| {
            let tx = RootLayout::cbsink_channel(siv, &handle);

            let sysinfo = Panel::new(SysInfo::default()).full_width();

            let omnibox = Panel::new(Omnibox::new()).full_width().max_height(3);

            let service_list = ServiceView::new(handle, tx.clone())
                .full_width()
                .full_height()
                .scrollable();

            v.add_child(sysinfo);
            v.add_child(omnibox);
            v.add_child(service_list);
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

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        match event {
            Event::Char('/') => {
                self.layout
                    .set_focus_index(RootLayoutChildren::Omnibox as usize);
                self.layout.on_event(event)
            }
            Event::Char(':') => {
                self.layout
                    .set_focus_index(RootLayoutChildren::Omnibox as usize);
                self.layout.on_event(event)
            }
            _ => self.layout.on_event(event),
        }
    }
}
