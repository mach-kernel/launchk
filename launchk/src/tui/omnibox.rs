use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key};
use cursive::{Printer, Vec2, View, XY};
use std::cell::RefCell;

use crate::tui::job_type_filter::JobTypeFilter;
use cursive::theme::{BaseColor, Color, Effect, Style};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::runtime::Handle;

use tokio::time::interval;

/// Consumers impl OmniboxSubscriber receive these commands
/// via a channel in a wrapped view
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxEvent {
    FocusServiceList,
    StateUpdate(OmniboxState),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxMode {
    NameFilter,
    JobTypeFilter,
    Idle,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OmniboxState {
    pub mode: OmniboxMode,
    pub tick: SystemTime,
    pub name_filter: String,
    pub job_type_filter: JobTypeFilter,
}

impl OmniboxState {
    pub fn update_existing(
        &self,
        mode: Option<OmniboxMode>,
        name_filter: Option<String>,
        job_type_filter: Option<JobTypeFilter>,
    ) -> OmniboxState {
        OmniboxState {
            tick: SystemTime::now(),
            mode: mode.unwrap_or(self.mode.clone()),
            name_filter: name_filter.unwrap_or(self.name_filter.clone()),
            job_type_filter: job_type_filter.unwrap_or(self.job_type_filter.clone()),
        }
    }
}

impl Default for OmniboxState {
    fn default() -> Self {
        Self {
            mode: OmniboxMode::Idle,
            tick: SystemTime::now(),
            name_filter: "".to_string(),
            job_type_filter: JobTypeFilter::default(),
        }
    }
}

/// Move OmniboxState back to some time after the user stops
/// interacting with it
async fn omnibox_tick(state: Arc<RwLock<OmniboxState>>, tx: Sender<OmniboxEvent>) {
    let mut tick_rate = interval(Duration::from_millis(500));

    loop {
        tick_rate.tick().await;

        let read = state.read().expect("Must read state");
        let OmniboxState { mode, tick, .. } = &*read;

        if *mode == OmniboxMode::Idle || tick.elapsed().unwrap() > Duration::from_secs(1) {
            continue;
        }

        let new = read.update_existing(Some(OmniboxMode::Idle), None, None);
        drop(read);
        let mut write = state.write().expect("Must write");

        let out = [
            tx.send(OmniboxEvent::FocusServiceList),
            tx.send(OmniboxEvent::StateUpdate(new.clone())),
        ];

        for msg in out.iter() {
            msg.as_ref().expect("Must send");
        }

        *write = new;
    }
}

/// Omnibox that pipes commands to a channel
/// we consume in the root view
pub struct Omnibox {
    state: Arc<RwLock<OmniboxState>>,
    tx: Sender<OmniboxEvent>,
    last_size: RefCell<XY<usize>>,
}

impl Omnibox {
    /// Create a new Omnibox and receive its rx on create
    pub fn new(handle: &Handle) -> (Self, Receiver<OmniboxEvent>) {
        let (tx, rx): (Sender<OmniboxEvent>, Receiver<OmniboxEvent>) = channel();
        let state = Arc::new(RwLock::new(OmniboxState::default()));

        let tx_state = state.clone();
        let tx_tick = tx.clone();

        handle.spawn(async move { omnibox_tick(tx_state, tx_tick).await });

        (
            Self {
                state,
                tx,
                last_size: RefCell::new(XY::new(0, 0)),
            },
            rx,
        )
    }

    fn handle_active(event: &Event, state: &OmniboxState) -> Option<OmniboxState> {
        let OmniboxState {
            mode, name_filter, ..
        } = &state;

        match (event, mode) {
            (Event::Char(c), OmniboxMode::NameFilter) => {
                Some(state.update_existing(None, Some(format!("{}{}", name_filter, c)), None))
            }
            (Event::Key(Key::Backspace), OmniboxMode::NameFilter) => {
                if name_filter.is_empty() {
                    None
                } else {
                    let mut nf = name_filter.clone();
                    nf.truncate(nf.len() - 1);
                    Some(state.update_existing(None, Some(nf), None))
                }
            }
            (ev, OmniboxMode::JobTypeFilter) => Self::handle_job_type_filter(ev, state),
            _ => None,
        }
    }

    fn handle_job_type_filter(event: &Event, state: &OmniboxState) -> Option<OmniboxState> {
        let mut jtf = state.job_type_filter.clone();

        match event {
            Event::Char('s') => jtf.toggle(JobTypeFilter::SYSTEM),
            Event::Char('g') => jtf.toggle(JobTypeFilter::GLOBAL),
            Event::Char('u') => jtf.toggle(JobTypeFilter::USER),
            Event::Char('a') => jtf.toggle(JobTypeFilter::AGENT),
            Event::Char('d') => jtf.toggle(JobTypeFilter::DAEMON),
            Event::Char('l') => jtf.toggle(JobTypeFilter::LOADED),
            _ => return None,
        };

        Some(state.update_existing(Some(OmniboxMode::JobTypeFilter), None, Some(jtf)))
    }

    fn handle_idle(event: &Event, state: &OmniboxState) -> Option<OmniboxState> {
        match event {
            Event::Char('/') => Some(state.update_existing(
                Some(OmniboxMode::NameFilter),
                Some("".to_string()),
                None,
            )),
            _ => Self::handle_job_type_filter(event, state),
        }
    }

    fn draw_command_header(&self, printer: &Printer<'_, '_>) {
        let read = self.state.read().expect("Must read state");
        let OmniboxState {
            name_filter, mode, ..
        } = &*read;

        let cmd_header = if name_filter.len() > 0 || *mode == OmniboxMode::NameFilter {
            "Filter > "
        } else {
            ""
        };

        let subtle = Style::from(Color::Light(BaseColor::Black));
        let purple = Style::from(Color::Light(BaseColor::Blue));

        let modal_hilight = if let OmniboxMode::Idle = mode {
            subtle
        } else {
            purple
        };

        // Print command header
        let width = (self.last_size.borrow().x / 2) - cmd_header.len();
        printer.with_style(modal_hilight, |p| p.print(XY::new(0, 0), cmd_header));

        // Print cmd header value
        printer.print(
            XY::new(cmd_header.len(), 0),
            format!("{:width$}", name_filter, width = width).as_str(),
        );
    }

    fn draw_job_type_filter(&self, printer: &Printer<'_, '_>) {
        let read = self.state.read().expect("Must read state");
        let OmniboxState {
            job_type_filter,
            mode,
            ..
        } = &*read;

        let jtf_ofs = if *mode != OmniboxMode::JobTypeFilter {
            // "[sguadl]"
            8
        } else {
            // "[system global user agent daemon loaded]"
            40
        };

        let mut jtf_ofs = self.last_size.borrow().x - jtf_ofs;
        printer.print(XY::new(jtf_ofs, 0), "[");
        jtf_ofs += 1;

        let inactive = Style::from(Color::Light(BaseColor::Black));
        let active = Style::from(Color::Light(BaseColor::Blue)).combine(Effect::Bold);

        for mask in [
            JobTypeFilter::SYSTEM,
            JobTypeFilter::GLOBAL,
            JobTypeFilter::USER,
            JobTypeFilter::AGENT,
            JobTypeFilter::DAEMON,
            JobTypeFilter::LOADED,
        ]
        .iter()
        {
            let mut mask_string = format!("{:?} ", mask).to_ascii_lowercase();
            if *mode != OmniboxMode::JobTypeFilter {
                mask_string.truncate(1);
            }

            if *mask == JobTypeFilter::LOADED && *mode == OmniboxMode::JobTypeFilter {
                mask_string.truncate(mask_string.len() - 1);
            }

            let style = if job_type_filter.contains(*mask) {
                active
            } else {
                inactive
            };

            printer.with_style(style, |p| {
                p.print(XY::new(jtf_ofs, 0), mask_string.as_str())
            });
            jtf_ofs += mask_string.len();
        }

        printer.print(XY::new(jtf_ofs, 0), "]");
    }
}

impl View for Omnibox {
    fn draw(&self, printer: &Printer<'_, '_>) {
        self.draw_command_header(printer);
        self.draw_job_type_filter(printer);
    }

    fn layout(&mut self, sz: Vec2) {
        self.last_size.replace(sz);
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let state = self.state.read().expect("Must read state");
        let mode = &state.mode;

        let new_state = match (event, mode) {
            (Event::CtrlChar('u'), _) => {
                self.tx
                    .send(OmniboxEvent::FocusServiceList)
                    .expect("Must focus");
                Some(OmniboxState::default())
            }
            (e, OmniboxMode::Idle) => Self::handle_idle(&e, &*state),
            (e, _) => Self::handle_active(&e, &*state),
        };

        if new_state.is_none() {
            return EventResult::Ignored;
        }

        let new_state = new_state.unwrap();
        self.tx
            .send(OmniboxEvent::StateUpdate(new_state.clone()))
            .expect("Must broadcast state");

        drop(state);
        let mut write = self.state.write().expect("Must write state");
        *write = new_state;

        EventResult::Consumed(None)
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }
}
