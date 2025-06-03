use std::collections::VecDeque;

use std::ptr::slice_from_raw_parts;
use std::sync::mpsc::{channel, Receiver, Sender};

use cursive::event::{Event, EventResult, Key};
use cursive::traits::{Resizable, Scrollable};
use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, NamedView, Panel};
use cursive::{Cursive, Vec2, View};

use tokio::runtime::Handle;

use crate::tui::omnibox::command::OmniboxCommand;
use crate::tui::omnibox::subscribed_view::{
    OmniboxResult, OmniboxSubscribedView, OmniboxSubscriber, Subscribable,
};
use crate::tui::omnibox::view::{OmniboxError, OmniboxEvent, OmniboxView};
use crate::tui::pager::show_pager;
use crate::tui::service_list::view::ServiceListView;
use crate::{
    launchd::command::dumpjpcategory,
    tui::dialog::{show_csr_info, show_help},
};
use crate::{launchd::command::dumpstate, tui::dialog};
use std::thread;

pub type CbSinkMessage = Box<dyn FnOnce(&mut Cursive) + Send>;

pub struct RootLayout {
    layout: LinearLayout,
    omnibox_tx: Sender<OmniboxEvent>,
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

async fn poll_omnibox(cb_sink: Sender<CbSinkMessage>, rx: Receiver<OmniboxEvent>) {
    loop {
        let recv = rx.recv().expect("Must receive event");

        log::info!("poll_omnibox: RECV {:?}", recv);

        cb_sink
            .send(Box::new(|siv| {
                siv.call_on_name("root_layout", |v: &mut NamedView<RootLayout>| {
                    v.get_mut().handle_omnibox_event(recv);
                });
            }))
            .expect("Must forward to root")
    }
}

impl RootLayout {
    pub fn new(siv: &mut Cursive, runtime_handle: &Handle) -> Self {
        let (omnibox, omnibox_tx, omnibox_rx) = OmniboxView::new(runtime_handle);
        let cbsink_channel = RootLayout::cbsink_channel(siv);

        runtime_handle.spawn(poll_omnibox(cbsink_channel.clone(), omnibox_rx));

        let mut new = Self {
            omnibox_tx,
            cbsink_channel,
            layout: LinearLayout::vertical(),
            runtime_handle: runtime_handle.clone(),
            key_ring: VecDeque::with_capacity(3),
        };

        new.setup(omnibox);
        new
    }

    fn setup(&mut self, omnibox: OmniboxView) {
        let sysinfo = Panel::new(crate::tui::sysinfo::make_layout());

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
            .unwrap();
    }

    /// Cursive uses a different crate for its channels (?), so this is some glue
    fn cbsink_channel(siv: &mut Cursive) -> Sender<CbSinkMessage> {
        let (tx, rx): (Sender<CbSinkMessage>, Receiver<CbSinkMessage>) = channel();
        let sink = siv.cb_sink().clone();

        thread::spawn(move || loop {
            if let Ok(cb_sink_msg) = rx.recv() {
                sink.send(cb_sink_msg)
                    .expect("Cannot forward CbSink message")
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

    fn handle_omnibox_event(&mut self, recv: OmniboxEvent) {
        let self_event = self.on_omnibox(recv.clone());

        let target = self
            .layout
            .get_child_mut(RootLayoutChildren::ServiceList as usize)
            .and_then(|v| v.as_any_mut().downcast_mut::<OmniboxSubscribedView>())
            .expect("Must forward to ServiceList");

        let omnibox_events = [self_event, target.on_omnibox(recv)];

        for omnibox_event in &omnibox_events {
            match omnibox_event {
                // Forward Omnibox command responses from view
                Ok(Some(c)) => self
                    .omnibox_tx
                    .send(OmniboxEvent::Command(c.clone()))
                    .expect("Must send response commands"),
                Err(OmniboxError::CommandError(s)) => self
                    .cbsink_channel
                    .send(dialog::show_notice(s.clone(), None))
                    .expect("Must show error"),
                _ => {}
            }
        }
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
        log::trace!("on_event: {:?}", event);

        

        match event {
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
        }
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.layout.layout(size)
    }
}

impl OmniboxSubscriber for RootLayout {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> OmniboxResult {
        match cmd {
            OmniboxEvent::Command(OmniboxCommand::Chain(cmds)) => {
                let errors: Vec<OmniboxError> = cmds
                    .iter()
                    .filter_map(|c| self.on_omnibox(OmniboxEvent::Command(c.clone())).err())
                    .collect();

                if errors.is_empty() {
                    Ok(None)
                } else {
                    Err(OmniboxError::Many(errors))
                }
            }
            OmniboxEvent::Command(OmniboxCommand::Quit) => {
                self.cbsink_channel
                    .send(Box::new(|s| {
                        s.quit();
                    }))
                    .expect("Must quit");

                Ok(None)
            }
            OmniboxEvent::Command(OmniboxCommand::Sudo) => {
                clearscreen::clear()
                    .map_err(|_| OmniboxError::CommandError("Cannot clear".to_string()))?;
                sudo::escalate_if_needed()
                    .map_err(|_| OmniboxError::CommandError("Cannot sudo".to_string()))?;
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
            OmniboxEvent::Command(OmniboxCommand::CSRInfo) => {
                self.cbsink_channel
                    .send(show_csr_info())
                    .expect("Must show prompt");

                Ok(None)
            }
            OmniboxEvent::Command(OmniboxCommand::DumpState) => {
                let (size, shmem) =
                    dumpstate().map_err(|e| OmniboxError::CommandError(e.to_string()))?;

                log::info!("shmem response sz {}", size);

                show_pager(&self.cbsink_channel, unsafe {
                    &*slice_from_raw_parts(shmem.region as *mut u8, size)
                })
                .map_err(OmniboxError::CommandError)?;

                Ok(None)
            }
            OmniboxEvent::Command(OmniboxCommand::DumpJetsamPropertiesCategory) => {
                let (size, shmem) =
                    dumpjpcategory().map_err(|e| OmniboxError::CommandError(e.to_string()))?;

                show_pager(&self.cbsink_channel, unsafe {
                    &*slice_from_raw_parts(shmem.region as *mut u8, size)
                })
                .map_err(OmniboxError::CommandError)?;

                Ok(None)
            }
            OmniboxEvent::Command(OmniboxCommand::Help) => {
                self.cbsink_channel
                    .send(show_help())
                    .expect("Must show prompt");

                Ok(None)
            }
            _ => Ok(None),
        }
    }
}
