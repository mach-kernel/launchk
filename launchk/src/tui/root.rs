use std::cell::RefCell;
use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Duration;

use cursive::event::{Event, EventResult, Key};
use cursive::traits::{Resizable, Scrollable};
use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, NamedView, Panel};
use cursive::{Cursive, Vec2, View, Printer};

use tokio::runtime::Handle;
use tokio::time::interval;

use crate::tui::dialog;
use crate::tui::omnibox::command::OmniboxCommand;
use crate::tui::omnibox::subscribed_view::{
    OmniboxResult, OmniboxSubscribedView, OmniboxSubscriber, Subscribable,
};
use crate::tui::omnibox::view::{OmniboxError, OmniboxEvent, OmniboxView};
use crate::tui::service_list::view::ServiceListView;
use crate::tui::sysinfo::SysInfo;

pub type CbSinkMessage = Box<dyn FnOnce(&mut Cursive) + Send>;

pub struct RootLayout {
    layout: LinearLayout,
    omnibox_rx: Receiver<OmniboxEvent>,
    omnibox_tx: Sender<OmniboxEvent>,
    last_focus_index: RefCell<usize>,
    runtime_handle: Handle,
    cbsink_channel: Sender<CbSinkMessage>,
    key_ring: VecDeque<Event>,
}

#[derive(Debug)]
enum RootLayoutChildren {
    #[allow(dead_code)]
    SysInfo,
    Omnibox,
    ServiceList,
}

impl RootLayout {
    pub fn new(siv: &mut Cursive, runtime_handle: &Handle) -> Self {
        let (omnibox, omnibox_tx, omnibox_rx) = OmniboxView::new(runtime_handle);

        let mut new = Self {
            omnibox_tx,
            omnibox_rx,
            layout: LinearLayout::vertical(),
            last_focus_index: RefCell::new(RootLayoutChildren::ServiceList as usize),
            cbsink_channel: RootLayout::cbsink_channel(siv, runtime_handle),
            runtime_handle: runtime_handle.clone(),
            key_ring: VecDeque::with_capacity(3),
        };

        new.setup(omnibox);
        new
    }

    fn setup(&mut self, omnibox: OmniboxView) {
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
                    sink.send(cb_sink_msg)
                        .expect("Cannot forward CbSink message")
                }
            }
        });

        tx.clone()
    }

    fn focus_and_forward(&mut self, child: RootLayoutChildren, event: Event) -> EventResult {
        self.layout
            .set_focus_index(child as usize)
            .expect("Must focus child");
        self.layout.on_event(event)
    }

    /// Poll for Omnibox commands without blocking
    fn poll_omnibox(&mut self) {
        let recv = self.omnibox_rx.try_recv();

        if recv.is_err() {
            return;
        }

        let recv = recv.unwrap();
        log::info!("[root/poll_omnibox]: {:?}", recv);

        self.on_omnibox(recv.clone())
            .expect("Root for effects only");

        // The Omnibox command is sent to the actively focused view
        let target = self
            .layout
            .get_child_mut(*self.last_focus_index.borrow())
            .and_then(|v| v.as_any_mut().downcast_mut::<OmniboxSubscribedView>());

        if target.is_none() {
            return;
        }

        match target.unwrap().on_omnibox(recv) {
            // Forward Omnibox command responses from view
            Ok(Some(c)) => self
                .omnibox_tx
                .send(OmniboxEvent::Command(c))
                .expect("Must send response commands"),
            Err(OmniboxError::CommandError(s)) => self
                .cbsink_channel
                .send(dialog::show_error(s))
                .expect("Must show error"),
            _ => {}
        };
    }

    fn ring_to_arrows(&mut self) -> Option<Event> {
        if self.key_ring.len() < 3 {
            None
        } else {
            let res = match self
                .key_ring
                .iter()
                .take(3)
                .collect::<Vec<&Event>>()
                .as_slice()
            {
                [Event::Key(Key::Esc), Event::Char('['), Event::Char('A')] => {
                    Some(Event::Key(Key::Up))
                }
                [Event::Key(Key::Esc), Event::Char('['), Event::Char('B')] => {
                    Some(Event::Key(Key::Down))
                }
                [Event::Key(Key::Esc), Event::Char('['), Event::Char('C')] => {
                    Some(Event::Key(Key::Right))
                }
                [Event::Key(Key::Esc), Event::Char('['), Event::Char('D')] => {
                    Some(Event::Key(Key::Left))
                }
                _ => None,
            };

            self.key_ring.truncate(0);
            res
        }
    }
}

impl ViewWrapper for RootLayout {
    wrap_impl!(self.layout: LinearLayout);
    
    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        log::debug!("[root/event]: {:?}", event);

        self.poll_omnibox();
        let ev = match event {
            Event::Char('/')
            | Event::Char(':')
            | Event::CtrlChar('u')
            | Event::Char('s')
            | Event::Char('g')
            | Event::Char('u')
            | Event::Char('a')
            | Event::Char('d')
            | Event::Char('l')
            | Event::Key(Key::Backspace) => {
                self.focus_and_forward(RootLayoutChildren::Omnibox, event)
            }
            // TODO: wtf?
            // After exiting $EDITOR, for some reason we get a termcap issue. iTerm and Apple Terminal
            // exhibit the same behavior. This was the easiest way to solve the problem for now.
            Event::Key(Key::Esc)
            | Event::Char('[')
            | Event::Char('A')
            | Event::Char('B')
            | Event::Char('C')
            | Event::Char('D') => {
                self.key_ring.push_back(event.clone());
                let event = self.ring_to_arrows().unwrap_or(event);
                self.layout.on_event(event)
            }
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

impl OmniboxSubscriber for RootLayout {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> OmniboxResult {
        match cmd {
            OmniboxEvent::Command(OmniboxCommand::Chain(cmds)) => {
                cmds.iter()
                    .try_for_each(|c| self.omnibox_tx.send(OmniboxEvent::Command(c.clone())))
                    .expect("Must send commands");
                Ok(None)
            }
            OmniboxEvent::Command(OmniboxCommand::Quit) => {
                self.cbsink_channel
                    .send(Box::new(|s| {
                        s.quit();
                    }))
                    .expect("Must quit");
                Ok(None)
            }
            // Triggered when toggling to idle
            OmniboxEvent::Command(OmniboxCommand::FocusServiceList) => {
                self.layout
                    .set_focus_index(RootLayoutChildren::ServiceList as usize)
                    .expect("Must focus SL");
                Ok(None)
            }
            OmniboxEvent::Command(OmniboxCommand::Confirm(p, c)) => {
                self.cbsink_channel
                    .send(dialog::show_prompt(self.omnibox_tx.clone(), p, c))
                    .expect("Must show prompt");
                Ok(None)
            }
            OmniboxEvent::Command(OmniboxCommand::DomainSessionPrompt(label, domain_only, f)) => {
                self.cbsink_channel
                    .send(dialog::domain_session_prompt(
                        label,
                        domain_only,
                        self.omnibox_tx.clone(),
                        f,
                    ))
                    .expect("Must show prompt");
                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
