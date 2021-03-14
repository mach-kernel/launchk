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

#[derive(Debug, Clone)]
enum OmniboxMode {
    Command,
    Filter,
    Clear,
}

pub struct Omnibox {
    content: RefCell<String>,
    mode: RefCell<OmniboxMode>,
}

impl Omnibox {
    pub fn new() -> Self {
        Self {
            content: RefCell::new("".into()),
            mode: RefCell::new(OmniboxMode::Clear),
        }
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

        match event {
            Event::Char('/') => {
                self.mode.replace(OmniboxMode::Filter);
                EventResult::Consumed(None)
            }
            Event::Char(':') => {
                self.mode.replace(OmniboxMode::Command);
                EventResult::Consumed(None)
            }
            Event::Char(c) => {
                self.content.borrow_mut().push(c);
                EventResult::Consumed(None)
            }
            Event::Key(Key::Backspace) if content_len > 0 => {
                self.content.borrow_mut().truncate(content_len - 1);
                EventResult::Consumed(None)
            }
            Event::CtrlChar('u') => {
                self.content.borrow_mut().truncate(0);
                self.mode.replace(OmniboxMode::Clear);
                EventResult::Consumed(None)
            }
            _ => EventResult::Ignored,
        }
    }

    fn take_focus(&mut self, _source: Direction) -> bool {
        true
    }
}
