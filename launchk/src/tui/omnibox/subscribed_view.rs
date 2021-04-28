use cursive::view::{Selector, ViewWrapper};
use cursive::views::{Panel, ResizedView, ScrollView};
use cursive::{Printer, Rect, Vec2, View};

use cursive::direction::Direction;
use cursive::event::{Event, EventResult};
use crate::tui::omnibox::view::{OmniboxEvent, OmniboxError, OmniboxCommand};

/// Boxed view we can match against for sending Omnibox events
pub struct OmniboxSubscribedView {
    inner: Box<dyn OmniboxSubscriber>,
}

/// Implement ViewWrapper to forward to inner
impl ViewWrapper for OmniboxSubscribedView {
    type V = dyn OmniboxSubscriber;

    fn with_view<F, R>(&self, f: F) -> Option<R> where
        F: FnOnce(&Self::V) -> R {
        Some(f(&*self.inner))
    }

    fn with_view_mut<F, R>(&mut self, f: F) -> Option<R> where
        F: FnOnce(&mut Self::V) -> R {
        Some(f(&mut *self.inner))
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

pub type OmniboxResult = Result<Option<OmniboxCommand>, OmniboxError>;

/// Implement for a view to be able to invoke subscribable()
pub trait OmniboxSubscriber: View {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> OmniboxResult;
}

impl<T: OmniboxSubscriber> OmniboxSubscriber for ResizedView<T> {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> OmniboxResult {
        self.with_view_mut(|v| v.on_omnibox(cmd)).unwrap_or(Err(OmniboxError::StandardError))
    }
}

impl<T: OmniboxSubscriber> OmniboxSubscriber for ScrollView<T> {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> OmniboxResult {
        self.get_inner_mut().on_omnibox(cmd)
    }
}

impl<T: OmniboxSubscriber> OmniboxSubscriber for Panel<T> {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> OmniboxResult {
        self.get_inner_mut().on_omnibox(cmd)
    }
}

impl OmniboxSubscriber for OmniboxSubscribedView {
    fn on_omnibox(&mut self, cmd: OmniboxEvent) -> OmniboxResult {
        self.inner.on_omnibox(cmd)
    }
}
