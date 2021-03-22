/*
Gonna go basement shit

- register(vec<children>) hook for creating an omnibox
- when focused, on key we go through children that impl update_filterable()
- can impl filterable for View<T>

- how can we do autoregistration (i.e. for any view)
- [or] call setup from init in each filterable view, your hw
- look up filterable by siv name?

- [or] our trait for a view with extensions like ncspot, with
  an into() for all views that would perform the registration step
*/

use cursive::direction::Direction;
use cursive::event::{Event, EventResult, Key};
use cursive::{Printer, View, XY};
use std::borrow::Borrow;
use std::cell::RefCell;

use cursive::theme::{BaseColor, Color, Style};
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxMode {
    Command,
    Filter,
    Clear,
}

pub struct Omnibox {
    content: RefCell<String>,
    mode: RefCell<OmniboxMode>,
    tx: Sender<OmniboxCommand>,
}

pub type OmniboxState = (OmniboxMode, String);

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum OmniboxCommand {
    Filter(String),
    Clear,
}

impl Omnibox {
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

        match event {
            Event::Char('/') => {
                self.mode.replace(OmniboxMode::Filter);
                self.tx.send(OmniboxCommand::Clear);
            }
            Event::Char(':') => {
                self.mode.replace(OmniboxMode::Command);
                self.tx.send(OmniboxCommand::Clear);
            }
            Event::Char(c) if *self.mode.borrow() != OmniboxMode::Clear => {
                self.content.borrow_mut().push(c);
                self.tx
                    .send(OmniboxCommand::Filter(self.content.borrow_mut().clone()));
            }
            Event::Key(Key::Backspace) if content_len > 0 => {
                self.content.borrow_mut().truncate(content_len - 1);
                self.tx
                    .send(OmniboxCommand::Filter(self.content.borrow_mut().clone()));
            }
            Event::CtrlChar('u') => {
                self.content.borrow_mut().truncate(0);
                self.mode.replace(OmniboxMode::Clear);
                self.tx.send(OmniboxCommand::Clear);
            }
            _ => event_result = EventResult::Ignored,
        }

        event_result
    }

    fn take_focus(&mut self, _source: Direction) -> bool {
        true
    }
}
