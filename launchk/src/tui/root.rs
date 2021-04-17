use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, NamedView, Panel};
use cursive::{Cursive, View, Vec2};
use tokio::runtime::Handle;
use tokio::time::interval;

use crate::tui::omnibox::{Omnibox, OmniboxEvent};
use crate::tui::service_list::ServiceListView;
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
    omnibox_rx: Receiver<OmniboxEvent>,
    last_focus_index: RefCell<usize>,
    runtime_handle: Handle,
    cbsink_channel: Sender<CbSinkMessage>,
}

#[allow(dead_code)]
#[derive(Debug)]
enum RootLayoutChildren {
    SysInfo,
    Omnibox,
    ServiceList,
}

impl RootLayout {
    pub fn new(siv: &mut Cursive, runtime_handle: &Handle) -> Self {
        let (omnibox, omnibox_rx) = Omnibox::new(runtime_handle);

        let mut new = Self {
            omnibox_rx,
            layout: LinearLayout::vertical(),
            last_focus_index: RefCell::new(RootLayoutChildren::ServiceList as usize),
            cbsink_channel: RootLayout::cbsink_channel(siv, runtime_handle),
            runtime_handle: runtime_handle.clone()

        };

        new.setup(omnibox);
        new
    }

    fn setup(&mut self, omnibox: Omnibox) {
        let sysinfo = Panel::new(SysInfo::default()).full_width();

        let omnibox = Panel::new(NamedView::new("omnibox", omnibox))
            .full_width()
            .max_height(3);

        let service_list = ServiceListView::new(&self.runtime_handle, self.cbsink_channel.clone())
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
                    sink.send(cb_sink_msg).expect("Cannot forward CbSink message")
                }
            }
        });

        tx.clone()
    }

    fn focus_and_forward(&mut self, child: RootLayoutChildren, event: Event) -> EventResult {
        println!("Forwarding {:?} {:?}", child, event);
        self.layout.set_focus_index(child as usize);
        self.layout.on_event(event)
    }

    /// Check for Omnibox commands during Cursive's on_event
    fn poll_omnibox(&mut self) {
        let recv = self.omnibox_rx.try_recv();

        if recv.is_err() {
            return;
        }

        let recv = recv.unwrap();

        // Triggered by Omnibox when toggling to idle
        if recv == OmniboxEvent::FocusServiceList {
            self.layout.set_focus_index(RootLayoutChildren::ServiceList as usize);
            return;
        }

        let target = self
            .layout
            .get_child_mut(*self.last_focus_index.borrow())
            .and_then(|v| v.as_any_mut().downcast_mut::<OmniboxSubscribedView>());

        if target.is_none() {
            return;
        }

        target.unwrap().on_omnibox(recv).expect("Must deliver Omnibox message");
    }
}

impl ViewWrapper for RootLayout {
    wrap_impl!(self.layout: LinearLayout);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        let ev = match event {
            Event::Char('/') | Event::Char(':') | Event::CtrlChar('u') => {
                self.focus_and_forward(RootLayoutChildren::Omnibox, event)
            }
            Event::Char('s') |
                Event::Char('g') |
                Event::Char('u') |
                Event::Char('a') |
                Event::Char('d') => self.focus_and_forward(RootLayoutChildren::Omnibox, event),
            _ => self.layout.on_event(event),
        };

        self.poll_omnibox();

        ev
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.poll_omnibox();
        self.layout.layout(size)
    }
}
