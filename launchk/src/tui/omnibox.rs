
use std::cell::RefCell;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use std::fmt;

use tokio::runtime::Handle;
use tokio::time::interval;

use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key};
use cursive::{Printer, Vec2, View, XY};
use cursive::theme::{BaseColor, Color, Effect, Style};

use crate::tui::job_type_filter::JobTypeFilter;
use std::fmt::Formatter;

/// Consumers impl OmniboxSubscriber receive these commands
/// via a channel in a wrapped view
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxEvent {
    FocusServiceList,
    StateUpdate(OmniboxState),
    Command(OmniboxCommand),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Load,
    Unload,
    Edit,
}

impl fmt::Display for OmniboxCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

static OMNIBOX_COMNANDS: [(OmniboxCommand, &str); 3] = [
    (OmniboxCommand::Load, "Load highlighted job"),
    (OmniboxCommand::Unload, "Unload highlighted job"),
    (OmniboxCommand::Edit, "Edit plist with $EDITOR, then reload job")
];

#[derive(Debug, Clone)]
pub enum OmniboxError {
    StandardError,
    CommandError(String),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxMode {
    CommandFilter,
    CommandConfirm(OmniboxCommand),
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

    /// Suggest a command based on name filter
    pub fn suggest_command(&self) -> Option<(OmniboxCommand, &str)> {
        let OmniboxState { mode, name_filter, .. } = self;

        if *mode != OmniboxMode::CommandFilter || name_filter.is_empty() {
            return None;
        }

        OMNIBOX_COMNANDS
            .iter()
            .filter(|(c, d)| c.to_string().contains(name_filter))
            .next()
            .map(|s| s.clone())
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

        if let OmniboxMode::CommandConfirm(cmd) = mode {
            tx.send(OmniboxEvent::Command(cmd.clone())).expect("Must confirm command");
        } else if *mode == OmniboxMode::Idle || tick.elapsed().unwrap() < Duration::from_secs(2) {
            continue;
        }

        // If you don't confirm command before tick, too slow!
        let name_filter_update = match *mode {
            OmniboxMode::CommandFilter | OmniboxMode::CommandConfirm(_) => Some("".to_string()),
            _ => None
        };

        let new = match *mode {
            OmniboxMode::CommandConfirm(_) => OmniboxState::default(),
            _ => read.update_existing(Some(OmniboxMode::Idle), name_filter_update, None)
        };

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

        let matched_command = state
            .suggest_command()
            .filter(|(cmd, _)| cmd.to_string() == *name_filter)
            .map(|(cmd, _)| cmd);

        match (event, mode) {
            (Event::Char(c), OmniboxMode::NameFilter)
            | (Event::Char(c), OmniboxMode::CommandFilter) => {
                Some(state.update_existing(None, Some(format!("{}{}", name_filter, c)), None))
            }
            (Event::Key(Key::Backspace), OmniboxMode::NameFilter)
            | (Event::Key(Key::Backspace), OmniboxMode::CommandFilter) => {
                if name_filter.is_empty() {
                    None
                } else {
                    let mut nf = name_filter.clone();
                    nf.truncate(nf.len() - 1);
                    Some(state.update_existing(None, Some(nf), None))
                }
            }
            (ev, OmniboxMode::JobTypeFilter) => Self::handle_job_type_filter(ev, state),
            (Event::Key(Key::Tab), OmniboxMode::CommandFilter) => {
                let suggestion = state.suggest_command();
                if suggestion.is_none() { return None; }
                let (cmd, _) = suggestion.unwrap();

                // Can submit from here, but catching a glimpse of the whole command
                // highlighting before flushing back out is confirmation that it did something
                Some(state.update_existing(None, Some(cmd.to_string()), None))
            },
            (Event::Key(Key::Enter), OmniboxMode::CommandFilter) if matched_command.is_some() => {
                Some(state.update_existing(Some(OmniboxMode::CommandConfirm(matched_command.unwrap())), None, None))
            }
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
            Event::Char(':') => Some(state.update_existing(
               Some(OmniboxMode::CommandFilter),
                Some("".to_string()),
                None
            )),
            _ => Self::handle_job_type_filter(event, state),
        }
    }

    fn draw_command_header(&self, printer: &Printer<'_, '_>) {
        let read = self.state.read().expect("Must read state");
        let OmniboxState {
            name_filter, mode, ..
        } = &*read;

        let cmd_header =  match *mode {
            OmniboxMode::NameFilter => "Filter > ",
            OmniboxMode::CommandFilter => "Command > ",
            OmniboxMode::CommandConfirm(_) => "OK! > ",
            _ if name_filter.len() > 1 => "Filter > ",
            _ => "",
        };

        let subtle = Style::from(Color::Light(BaseColor::Black));
        let purple = Style::from(Color::Light(BaseColor::Blue));

        let modal_hilight = if let OmniboxMode::Idle = mode {
            subtle
        } else {
            purple
        };

        // Print command header
        // let width = (self.last_size.borrow().x / 2) - cmd_header.len();
        printer.with_style(modal_hilight, |p| p.print(XY::new(0, 0), cmd_header));

        // Print string filter
        printer.print(XY::new(cmd_header.len(), 0),name_filter);

        if let OmniboxMode::CommandFilter = mode {
            let mut start = cmd_header.len() + name_filter.len();
            let suggestion = read.suggest_command();
            if suggestion.is_none() { return; }
            let (cmd, desc) = suggestion.unwrap();

            let cmd_string = cmd.to_string().replace(name_filter, "");

            printer.with_style(
                subtle,
                |p| p.print(XY::new(start, 0), cmd_string.as_str())
            );

            start += cmd_string.len() + 1;

            printer.print(XY::new(start, 0), format!("({})", desc).as_str());
        };
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
            // TODO: also only not loaded
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
