use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key};
use cursive::{Printer, View, XY};
use std::cell::RefCell;

use cursive::theme::{BaseColor, Color, Style};
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Eq, PartialEq)]
enum OmniboxMode {
    Command,
    Filter,
    Clear,
}

/// Omnibox that pipes commands to a channel
/// we consume in the root view
pub struct Omnibox {
    content: RefCell<String>,
    mode: RefCell<OmniboxMode>,
    tx: Sender<OmniboxCommand>,
}

/// Consumers receive OmniboxCommand messages without
/// having to look at its internal state
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    NoOp,
    Filter(String),
    Clear,
}

impl Omnibox {
    /// Create a new Omnibox and receive its rx on create
    pub fn new() -> (Self, Receiver<OmniboxCommand>) {
        let (tx, rx): (Sender<OmniboxCommand>, Receiver<OmniboxCommand>) = channel();

        (
            Self {
                content: RefCell::new("".into()),
                mode: RefCell::new(OmniboxMode::Clear),
                tx,
            },
            rx,
        )
    }
}

impl View for Omnibox {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let current_mode = self.mode.borrow();
        let fmt_mode = match *current_mode {
            OmniboxMode::Clear => "".into(),
            _ => format!("{:?} > ", current_mode),
        };

        let subtle = Style::from(Color::Light(BaseColor::Black));
        printer.with_style(subtle, |p| p.print(XY::new(0, 0), &fmt_mode));
        printer.print(
            XY::new(fmt_mode.chars().count(), 0),
            self.content.borrow().as_str(),
        );
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        let content_len = self.content.borrow().chars().count();
        let mut event_result = EventResult::Consumed(None);

        let command = match event {
            Event::Char('/') => {
                self.mode.replace(OmniboxMode::Filter);
                OmniboxCommand::Clear
            }
            // TODO: Overkill and unused
            // Event::Char(':') => {
            //     self.mode.replace(OmniboxMode::Command);
            //     OmniboxCommand::Clear
            // }
            Event::Char(c) if *self.mode.borrow() != OmniboxMode::Clear => {
                self.content.borrow_mut().push(c);
                OmniboxCommand::Filter(self.content.borrow_mut().clone())
            }
            Event::Key(Key::Backspace) if content_len > 0 => {
                self.content.borrow_mut().truncate(content_len - 1);
                OmniboxCommand::Filter(self.content.borrow_mut().clone())
            }
            Event::CtrlChar('u') => {
                self.content.borrow_mut().truncate(0);
                self.mode.replace(OmniboxMode::Clear);
                OmniboxCommand::Clear
            }
            _ => {
                event_result = EventResult::Ignored;
                OmniboxCommand::NoOp
            }
        };

        let sent = self.tx.send(command);
        if sent.is_err() {
            panic!("Unable to send Omnibox command");
        }
        sent.unwrap();

        event_result
    }

    /// If offered focus, we should always take it
    fn take_focus(&mut self, _source: Direction) -> bool {
        true
    }
}
