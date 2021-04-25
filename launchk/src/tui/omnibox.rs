
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
    LabelFilter,
    JobTypeFilter,
    Idle,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OmniboxState {
    pub mode: OmniboxMode,
    pub tick: SystemTime,
    pub label_filter: String,
    pub command_filter: String,
    pub job_type_filter: JobTypeFilter,
}

impl OmniboxState {
    /// Produce new state
    pub fn with_new(
        &self,
        mode: Option<OmniboxMode>,
        label_filter: Option<String>,
        command_filter: Option<String>,
        job_type_filter: Option<JobTypeFilter>,
    ) -> OmniboxState {
        OmniboxState {
            tick: SystemTime::now(),
            mode: mode.unwrap_or(self.mode.clone()),
            label_filter: label_filter.unwrap_or(self.label_filter.clone()),
            command_filter: command_filter.unwrap_or(self.command_filter.clone()),
            job_type_filter: job_type_filter.unwrap_or(self.job_type_filter.clone()),
        }
    }

    /// Suggest a command based on name filter
    pub fn suggest_command(&self) -> Option<(OmniboxCommand, &str)> {
        let OmniboxState { mode, command_filter, .. } = self;

        if *mode != OmniboxMode::CommandFilter || command_filter.is_empty() {
            return None;
        }

        OMNIBOX_COMNANDS
            .iter()
            .filter(|(c, _)| c.to_string().contains(command_filter))
            .next()
            .map(|s| s.clone())
    }
}

impl Default for OmniboxState {
    fn default() -> Self {
        Self {
            mode: OmniboxMode::Idle,
            tick: SystemTime::now(),
            label_filter: "".to_string(),
            command_filter: "".to_string(),
            job_type_filter: JobTypeFilter::launchk_default(),
        }
    }
}

/// Move OmniboxState back to idle some time after the user stops
/// interacting with it
async fn tick(state: Arc<RwLock<OmniboxState>>, tx: Sender<OmniboxEvent>) {
    let mut tick_rate = interval(Duration::from_millis(500));

    loop {
        tick_rate.tick().await;

        let read = state.read().expect("Must read state");
        let OmniboxState { mode, tick, .. } = &*read;

        log::trace!("[omnibox/tick]: {:?}", &*read);

        // Confirm command immediately
        if let OmniboxMode::CommandConfirm(cmd) = mode {
            tx.send(OmniboxEvent::Command(cmd.clone())).expect("Must confirm command");
        } else if *mode == OmniboxMode::Idle || tick.elapsed().unwrap() < Duration::from_secs(2) {
            continue;
        }

        let new = match *mode {
            OmniboxMode::CommandFilter| OmniboxMode::CommandConfirm(_) => read.with_new(Some(OmniboxMode::Idle), None, Some("".to_string()), None),
            _ => read.with_new(Some(OmniboxMode::Idle), None, None, None),
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

        log::debug!("[omnibox/tick]: New state: {:?}", &new);

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

        handle.spawn(async move { tick(tx_state, tx_tick).await });

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
            mode, label_filter, command_filter, ..
        } = &state;

        let suggested_command = state.suggest_command();

        let matched_command = suggested_command
            .as_ref()
            .filter(|(cmd, _)| cmd.to_string() == *command_filter)
            .map(|(cmd, _)| cmd.clone());

        let (lf_char, cf_char) = match (event, mode) {
            (Event::Char(c), OmniboxMode::LabelFilter) => (Some(format!("{}{}", label_filter, c)), None),
            (Event::Char(c), OmniboxMode::CommandFilter) => (None, Some(format!("{}{}", command_filter, c))),
            _ => (None, None)
        };

        match (event, mode) {
            (ev, OmniboxMode::JobTypeFilter) => Self::handle_job_type_filter(ev, state),
            // User -> string filters
            (Event::Char(_), OmniboxMode::LabelFilter) | (Event::Char(_), OmniboxMode::CommandFilter)  => {
                Some(state.with_new(None, lf_char, cf_char, None))
            },
            (Event::Key(Key::Backspace), OmniboxMode::LabelFilter) if !label_filter.is_empty() => {
                let mut lf = label_filter.clone();
                lf.truncate(lf.len() - 1);
                Some(state.with_new(None, Some(lf), None, None))
            },
            (Event::Key(Key::Backspace), OmniboxMode::CommandFilter) if !command_filter.is_empty() => {
                let mut cf = command_filter.clone();
                cf.truncate(cf.len() - 1);
                Some(state.with_new(None, None, Some(cf), None))
            },
            // Complete suggestion
            (Event::Key(Key::Tab), OmniboxMode::CommandFilter) if suggested_command.is_some() => {
                let (cmd, _) = suggested_command.unwrap();

                // Can submit from here, but catching a glimpse of the whole command
                // highlighting before flushing back out is confirmation that it did something
                Some(state.with_new(None, None, Some(cmd.to_string()), None))
            },
            // Submit command only if string filter eq suggestion (i.e. requires you to tab-complete first)
            (Event::Key(Key::Enter), OmniboxMode::CommandFilter) if matched_command.is_some() => {
                Some(state.with_new(Some(OmniboxMode::CommandConfirm(matched_command.unwrap())), None, None, None))
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

        Some(state.with_new(Some(OmniboxMode::JobTypeFilter), None, None, Some(jtf)))
    }

    fn handle_idle(event: &Event, state: &OmniboxState) -> Option<OmniboxState> {
        match event {
            Event::Char('/') => Some(state.with_new(
                Some(OmniboxMode::LabelFilter),
                Some("".to_string()),
                None,
                None,
            )),
            Event::Char(':') => Some(state.with_new(
               Some(OmniboxMode::CommandFilter),
                None,
                Some("".to_string()),
                None
            )),
            _ => Self::handle_job_type_filter(event, state),
        }
    }

    fn draw_command_header(&self, printer: &Printer<'_, '_>) {
        let read = self.state.read().expect("Must read state");
        let OmniboxState {
            command_filter, label_filter, mode, ..
        } = &*read;

        let cmd_header =  match *mode {
            OmniboxMode::LabelFilter => "Filter > ",
            OmniboxMode::CommandFilter => "Command > ",
            OmniboxMode::CommandConfirm(_) => "OK! > ",
            _ if command_filter.len() < 1 && label_filter.len() > 0 => "Filter > ",
            _ => "",
        };

        let subtle = Style::from(Color::Light(BaseColor::Black));
        let purple = Style::from(Color::Light(BaseColor::Blue));

        let modal_hilight = if let OmniboxMode::Idle = mode {
            subtle
        } else {
            purple
        };

        let visible_filter = if command_filter.len() > 0 || *mode == OmniboxMode::CommandFilter {
            command_filter
        } else {
            label_filter
        };

        // Print command header
        printer.with_style(modal_hilight, |p| p.print(XY::new(0, 0), cmd_header));

        // Print string filter
        printer.print(XY::new(cmd_header.len(), 0), visible_filter);

        // Print command suggestion
        if let OmniboxMode::CommandFilter = mode {
            let sub = printer.offset(XY::new(cmd_header.len() + visible_filter.len(), 0));
            self.draw_command_suggestion(&sub);
        };
    }

    fn draw_command_suggestion(&self, printer: &Printer<'_, '_>) {
        let state = self.state.read().expect("Must read");
        let suggestion = state.suggest_command();

        if suggestion.is_none() { return; }
        let (cmd, desc) = suggestion.unwrap();
        let cmd_string = cmd.to_string().replace(&state.command_filter, "");

        printer.with_style(
            Style::from(Color::Light(BaseColor::Black)),
            |p| p.print(XY::new(0, 0), cmd_string.as_str())
        );

        let start = cmd_string.len() + 1;
        printer.print(XY::new(start, 0), format!("({})", desc).as_str());
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

            // No space at end if expanded
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
            (Event::Key(Key::Esc), _) => {
                self.tx
                    .send(OmniboxEvent::FocusServiceList)
                    .expect("Must focus");
                Some(state.with_new(Some(OmniboxMode::Idle), None, None, None))
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
