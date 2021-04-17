use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key};
use cursive::{Printer, View, XY};
use std::cell::RefCell;

use cursive::theme::{BaseColor, Color, Style};
use std::sync::mpsc::{channel, Receiver, Sender};
use crate::tui::job_type_filter::JobTypeFilter;
use tokio::runtime::Handle;
use std::sync::{RwLock, Arc};
use std::time::{SystemTime, Duration};

use tokio::time::interval;

/// Consumers impl OmniboxSubscriber receive these commands
/// via a channel in a wrapped view
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxEvent {
    NoOp,
    Clear,
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
    pub fn update_existing(&self, mode: Option<OmniboxMode>, name_filter: Option<String>, job_type_filter: Option<JobTypeFilter>) -> OmniboxState {
        OmniboxState {
            tick: SystemTime::now(),
            mode: mode.unwrap_or(self.mode.clone()),
            name_filter: name_filter.unwrap_or(self.name_filter.clone()),
            job_type_filter: job_type_filter.unwrap_or(self.job_type_filter.clone()),
        }
    }

    pub fn now(&self) -> OmniboxState {
        let mut with_now = self.clone();
        with_now.tick = SystemTime::now();
        with_now
    }
}

impl Default for OmniboxState {
    fn default() -> Self {
        Self {
            mode: OmniboxMode::Idle,
            tick: SystemTime::now(),
            name_filter: "".to_string(),
            job_type_filter: JobTypeFilter::default()
        }
    }
}

/// Move OmniboxState back to some time after the user stops
/// interacting with it
async fn omnibox_tick(state: Arc<RwLock<OmniboxState>>, tx: Sender<OmniboxEvent>) {
    let mut tick_rate = interval(Duration::from_millis(1000));

    loop {
        tick_rate.tick().await;
        let read = state.read().expect("Must read state");

        let OmniboxState { mode, tick, .. } = &*read;
        let delta_ms = SystemTime::now().duration_since(*tick).unwrap().as_millis();

        if *mode == OmniboxMode::Idle && delta_ms < 2000 {
            continue;
        }

        let new = read.update_existing(Some(OmniboxMode::Idle), None, None);
        drop(read);
        let mut write = state.write().expect("Must write");

        // Refocus the service list
        tx.send(OmniboxEvent::FocusServiceList);

        // Send the state update
        tx.send(OmniboxEvent::StateUpdate(new.clone()));
        *write = new;
    }
}

/// Omnibox that pipes commands to a channel
/// we consume in the root view
pub struct Omnibox {
    state: Arc<RwLock<OmniboxState>>,
    tx: Sender<OmniboxEvent>,
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
            },
            rx,
        )
    }

    fn handle_active(event: &Event, state: &OmniboxState) -> OmniboxState {
        let OmniboxState { mode, name_filter, .. } = &state;
        
        match (event, mode) {
            (Event::Char(c), OmniboxMode::NameFilter) =>
                state.update_existing(None, Some(format!("{}{}", name_filter, c)), None),
            (Event::Key(Key::Backspace), OmniboxMode::NameFilter) => {
                if name_filter.is_empty() {
                    state.now()
                } else {
                    let mut nf = name_filter.clone();
                    nf.truncate(nf.len() - 1);
                    state.update_existing(
                        None,
                        Some(nf),
                        None
                    )
                }
            },
            (ev, OmniboxMode::JobTypeFilter) => Self::handle_job_type_filter(ev, state),
            _ => state.now()
        }
    }

    fn handle_job_type_filter(event: &Event, state: &OmniboxState) -> OmniboxState {
        match event {
            Event::Char('s') => state.update_existing(Some(OmniboxMode::JobTypeFilter), None, Some(JobTypeFilter::SYSTEM)),
            Event::Char('g') => state.update_existing(Some(OmniboxMode::JobTypeFilter), None, Some(JobTypeFilter::GLOBAL)),
            Event::Char('u') => state.update_existing(Some(OmniboxMode::JobTypeFilter), None, Some(JobTypeFilter::USER)),
            Event::Char('a') => state.update_existing(Some(OmniboxMode::JobTypeFilter), None, Some(JobTypeFilter::AGENT)),
            Event::Char('d') => state.update_existing(Some(OmniboxMode::JobTypeFilter), None, Some(JobTypeFilter::DAEMON)),
            _ => state.now(),
        }
    }

    fn handle_idle(event: &Event, state: &OmniboxState) -> OmniboxState {
        match event {
            Event::Char('/') => state.update_existing(Some(OmniboxMode::NameFilter), Some("".to_string()), None),
            _ => Self::handle_job_type_filter(event, state),
        }
    }
}

impl View for Omnibox {
    fn draw(&self, printer: &Printer<'_, '_>) {
        // for char in "[sguad]".to_string().chars() {
        //     if jtf.contains(char) {
        //         printer.with_style(style_on, |p| p.print(XY::new(begin, 0), char.to_string().as_str()));
        //     } else {
        //         printer.print(XY::new(begin, 0), char.to_string().as_str());
        //     }
        //
        //     begin += 1;
        // }

        let state = &*self.state.read().expect("Must read state");
        let OmniboxState { mode, name_filter, .. } = &state;

        let fmt_cmd = match mode {
            OmniboxMode::NameFilter if name_filter.len() > 0 => "Filter > ".to_string(),
            OmniboxMode::JobTypeFilter => "[s]ystem | [g]lobal | [u]ser | [a]gent | [d]aemon".to_string(),
            _ => "".to_string(),
        };

        let subtle = Style::from(Color::Light(BaseColor::Black));
        let purple = Style::from(Color::Light(BaseColor::Blue));
        let style = if let OmniboxMode::Idle = mode {
            subtle
        } else {
            purple
        };

        printer.with_style(style, |p| p.print(XY::new(0, 0), &fmt_cmd));

        match mode {
            OmniboxMode::NameFilter => {
                printer.print(
                    XY::new(fmt_cmd.chars().count(), 0),
                    name_filter.as_str(),
                );
            }
            _ => {}
        };
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let state = self.state.read().expect("Must read state");
        let mode = &state.mode;

        let new_state = match mode {
            OmniboxMode::Idle => Self::handle_idle(&event, &*state),
            _ => Self::handle_active(&event, &*state),
        };

        drop(state);

        self.tx.send(OmniboxEvent::StateUpdate(new_state.clone()));
        let mut write = self.state.write().expect("Must write state");
        *write = new_state;
        EventResult::Consumed(None)
    }

    fn take_focus(&mut self, _: Direction) -> bool {
        true
    }
}
