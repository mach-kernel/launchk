use cursive::view::{AnyView, ViewWrapper};
use cursive::views::{LinearLayout, NamedView, Panel};
use cursive::{Cursive, View};
use tokio::runtime::Handle;
use tokio::time::interval;

use crate::tui::omnibox::{Omnibox, OmniboxCommand};
use crate::tui::service::ServiceView;
use crate::tui::sysinfo::SysInfo;
use cursive::event::{Event, EventResult};
use cursive::traits::{Resizable, Scrollable};
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use crate::tui::omnibox_subscribed_view::{OmniboxSubscribedView, OmniboxSubscriber, Subscribable};

pub type CbSinkMessage = Box<dyn FnOnce(&mut Cursive) + Send>;

pub struct RootLayout {
    layout: LinearLayout,
    omnibox_rx: RefCell<Option<Receiver<OmniboxCommand>>>,
    last_focus_index: RefCell<usize>,
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
            omnibox_rx: RefCell::new(None),
            last_focus_index: RefCell::new(RootLayoutChildren::ServiceList as usize),
        }
    }

    pub fn setup(&mut self, siv: &mut Cursive, handle: Handle) {
        let tx = RootLayout::cbsink_channel(siv, &handle);

        let sysinfo = Panel::new(SysInfo::default()).full_width();

        let (omnibox, rx_omnibox) = Omnibox::new();
        self.omnibox_rx.replace(Some(rx_omnibox));

        let omnibox = Panel::new(NamedView::new("omnibox", omnibox))
            .full_width()
            .max_height(3);

        let service_list = ServiceView::new(handle, tx.clone())
            .full_width()
            .full_height()
            .scrollable()
            .subscribable();

        self.with_view_mut(|v| {
            v.add_child(sysinfo);
            v.add_child(omnibox);
            v.add_child(service_list);
        });

        self.layout
            .set_focus_index(RootLayoutChildren::ServiceList as usize)
            .unwrap_or(());
    }

    /// Cursive uses a different crate for its channel, so this is some glue
    fn cbsink_channel(siv: &mut Cursive, handle: &Handle) -> Sender<CbSinkMessage> {
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

    fn focus_and_forward(&mut self, child: RootLayoutChildren, event: Event) -> EventResult {
        self.last_focus_index.replace(self.layout.get_focus_index());
        self.layout.set_focus_index(child as usize).unwrap_or(());
        self.layout.on_event(event)
    }

    /// Check for Omnibox commands during Cursive's on_event
    fn poll_omnibox(&mut self) {
        let rxcell = self.omnibox_rx.borrow();
        let rxcell = rxcell.as_ref();

        if rxcell.is_none() {
            return;
        }

        let recv = rxcell.unwrap().try_recv();

        if recv.is_err() {
            return;
        }

        let target = self
            .layout
            .get_child_mut(*self.last_focus_index.borrow())
            .and_then(|v| v.as_any_mut().downcast_mut::<OmniboxSubscribedView>());

        if target.is_none() {
            return;
        }

        target.unwrap().on_omnibox(recv.unwrap());
    }
}

impl ViewWrapper for RootLayout {
    wrap_impl!(self.layout: LinearLayout);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        self.poll_omnibox();

        match event {
            Event::Char('/') | Event::Char(':') | Event::CtrlChar('u') => {
                self.focus_and_forward(RootLayoutChildren::Omnibox, event)
            }
            _ => self.layout.on_event(event),
        }
    }
}
