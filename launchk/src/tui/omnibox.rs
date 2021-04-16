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
pub enum OmniboxCommand {
    NoOp,
    NameFilter(String),
    JobTypeFilter(JobTypeFilter),
    Clear,
    Refocus,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum OmniboxMode {
    Active,
    Idle,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct OmniboxState(pub OmniboxMode, pub OmniboxCommand, pub SystemTime);

impl Default for OmniboxState {
    fn default() -> Self {
        Self(OmniboxMode::Idle, OmniboxCommand::NoOp, SystemTime::now())
    }
}

/// Move OmniboxState back to some time after the user stops
/// interacting with it
async fn omnibox_tick(state: Arc<RwLock<OmniboxState>>, tx: Sender<OmniboxCommand>) {
    let mut tick_rate = interval(Duration::from_millis(500));

    loop {
        tick_rate.tick().await;

        let read = state.read().expect("Must read state");
        let OmniboxState(ref mode, ref cmd, ref last) = &*read;

        if *mode == OmniboxMode::Idle || SystemTime::now().duration_since(*last).unwrap().as_millis() < 500 {
            continue;
        }

        let cmd = cmd.clone();
        drop(read);

        let mut write = state.write().expect("Must queue write");
        *write = OmniboxState(OmniboxMode::Idle, cmd.clone(), SystemTime::now());

        // Refocus the service list
        tx.send(OmniboxCommand::Refocus);
    }
}

/// Omnibox that pipes commands to a channel
/// we consume in the root view
pub struct Omnibox {
    state: Arc<RwLock<OmniboxState>>,
    tx: Sender<OmniboxCommand>,
}

impl Omnibox {
    /// Create a new Omnibox and receive its rx on create
    pub fn new(handle: &Handle) -> (Self, Receiver<OmniboxCommand>) {
        let (tx, rx): (Sender<OmniboxCommand>, Receiver<OmniboxCommand>) = channel();
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

    fn merge_job_type_filter(cmd: &OmniboxCommand, given: JobTypeFilter) -> OmniboxCommand {
        if let OmniboxCommand::JobTypeFilter(jtf) = cmd {
            let mut new_filter = jtf.clone();
            new_filter.toggle(given);
            OmniboxCommand::JobTypeFilter(new_filter)
        } else {
            OmniboxCommand::JobTypeFilter(JobTypeFilter::default())
        }
    }

    fn handle_active(event: &Event, state: &OmniboxState) -> OmniboxCommand {
        let OmniboxState(_, ref cmd, _) = state;

        match (event, cmd) {
            (Event::Char(c), OmniboxCommand::NameFilter(of)) => OmniboxCommand::NameFilter(format!("{}{}", of, c)),
            (Event::Key(Key::Backspace), OmniboxCommand::NameFilter(of)) => {
                if of.is_empty() {
                    OmniboxCommand::NoOp
                } else {
                    let mut truncate_me = of.clone();
                    truncate_me.truncate(truncate_me.len() - 1);
                    OmniboxCommand::NameFilter(truncate_me)
                }
            }
            _ => OmniboxCommand::NoOp,
        }
    }

    fn handle_idle(event: &Event, state: &OmniboxState) -> OmniboxCommand {
        let OmniboxState(_, ref cmd, _) = state;

        match (event, cmd) {
            // (Event::Char('/'), OmniboxCommand::NameFilter(_)) => cmd.clone(),
            (Event::Char('/'), _)  => OmniboxCommand::NameFilter("".to_string()),
            (Event::Char('s'), _) => Self::merge_job_type_filter(cmd, JobTypeFilter::SYSTEM),
            (Event::Char('g'), _) => Self::merge_job_type_filter(cmd, JobTypeFilter::GLOBAL),
            (Event::Char('u'), _) => Self::merge_job_type_filter(cmd, JobTypeFilter::USER),
            (Event::Char('a'), _) => Self::merge_job_type_filter(cmd, JobTypeFilter::AGENT),
            (Event::Char('d'), _) => Self::merge_job_type_filter(cmd, JobTypeFilter::DAEMON),
            _ => OmniboxCommand::NoOp
        }
    }
}

impl View for Omnibox {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let state = &*self.state.read().expect("Must read state");
        let OmniboxState(mode, cmd, _) = state.clone();

        let fmt_cmd = match cmd {
            OmniboxCommand::Clear => "".into(),
            OmniboxCommand::NameFilter(_) => "Filter > ".to_string(),
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

        match cmd {
            OmniboxCommand::NameFilter(filter) => {
                printer.print(
                    XY::new(fmt_cmd.chars().count(), 0),
                    filter.as_str(),
                );
            }
            _ => {}
        };
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let read = self.state.read().expect("Must read state");
        let state = &read.clone();
        let mut event_result = EventResult::Consumed(None);

        let command = if let Event::CtrlChar('u') = event {
            OmniboxCommand::Clear
        } else if state.0 == OmniboxMode::Active {
            Self::handle_active(&event, &state)
        } else if state.0 == OmniboxMode::Idle {
            self.tx.send(OmniboxCommand::Refocus).expect("Must send Omnibox command");
            Self::handle_idle(&event, &state)
        } else {
            OmniboxCommand::NoOp
        };

        if command == OmniboxCommand::NoOp {
            event_result = EventResult::Ignored;
        }

        drop(read);

        match event_result {
            EventResult::Ignored => return event_result,
            _ => {
                let mut write = self.state.write().expect("Must write state");
                *write = OmniboxState(OmniboxMode::Active, command.clone(), SystemTime::now());
                drop(write);
            }
        };

        self.tx.send(command).expect("Must send Omnibox command");
        event_result
    }

    /// If offered focus, we should always take it
    fn take_focus(&mut self, _source: Direction) -> bool {
        true
    }
}
