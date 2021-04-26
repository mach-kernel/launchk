use cursive::view::{Selector, ViewWrapper};
use cursive::views::{Panel, ResizedView, ScrollView};
use cursive::{Printer, Rect, Vec2, View};

use cursive::direction::Direction;
use cursive::event::{Event, EventResult};
use crate::tui::omnibox::view::{OmniboxEvent, OmniboxError};

/// Boxed view we can match against for sending Omnibox events
pub struct OmniboxSubscribedView {
    inner: Box<dyn OmniboxSubscriber>,
}

impl View for OmniboxSubscribedView {
    fn draw(&self, printer: &Printer<'_, '_>) {
        self.inner.draw(printer)
    }

    fn layout(&mut self, size: Vec2) {
        self.inner.layout(size)
    }

    fn needs_relayout(&self) -> bool {
        self.inner.needs_relayout()
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        self.inner.required_size(constraint)
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        self.inner.on_event(event)
    }

    fn call_on_any<'a>(&mut self, sel: &Selector<'_>, f: &'a mut dyn FnMut(&mut dyn View)) {
        self.inner.call_on_any(sel, f)
    }

    fn focus_view(&mut self, sel: &Selector<'_>) -> Result<(), ()> {
        self.inner.focus_view(sel)
    }

    fn take_focus(&mut self, source: Direction) -> bool {
        self.inner.take_focus(source)
    }

    fn important_area(&self, view_size: Vec2) -> Rect {
        self.inner.important_area(view_size)
    }

    fn type_name(&self) -> &'static str {
        self.inner.type_name()
    }
}

impl OmniboxSubscribedView {
    pub fn new(view: impl OmniboxSubscriber) -> Self {
        Self {
            inner: Box::new(view),
        }
    }
}

/// Lift views implementing OmniboxSubscriber into matchable container
pub trait Subscribable: View + OmniboxSubscriber {
    fn subscribable(self) -> OmniboxSubscribedView
    where
        Self: Sized,
    {
        OmniboxSubscribedView::new(self)
    }
}

impl<T> Subscribable for T where T: OmniboxSubscriber {}

/// Implement for a view to be able to invoke subscribable()
pub trait OmniboxSubscriber: View {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> Result<(), OmniboxError>;
}

impl<T: OmniboxSubscriber> OmniboxSubscriber for ResizedView<T> {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> Result<(), OmniboxError> {
        self.with_view_mut(|v| v.on_omnibox(cmd)).unwrap_or(Err(OmniboxError::StandardError))
    }
}

impl<T: OmniboxSubscriber> OmniboxSubscriber for ScrollView<T> {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> Result<(), OmniboxError> {
        self.get_inner_mut().on_omnibox(cmd)
    }
}

impl<T: OmniboxSubscriber> OmniboxSubscriber for Panel<T> {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> Result<(), OmniboxError> {
        self.get_inner_mut().on_omnibox(cmd)
    }
}

impl OmniboxSubscriber for OmniboxSubscribedView {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> Result<(), OmniboxError> {
        self.inner.on_omnibox(cmd)
    }
}
