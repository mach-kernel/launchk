use crate::tui::table::table_headers::TableHeaders;
use cursive::event::{Event, EventResult};
use cursive::traits::{Resizable, Scrollable};
use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, ResizedView, ScrollView, SelectView};
use cursive::{Vec2, View, XY};
use std::cell::RefCell;
use std::marker::PhantomData;

pub trait TableListItem {
    fn as_row(&self) -> Vec<String>;
}

pub struct TableListView<T> {
    linear_layout: LinearLayout,
    last_layout_size: RefCell<XY<usize>>,
    num_columns: usize,
    inner: PhantomData<T>,
}

impl<T: 'static + TableListItem> TableListView<T> {
    pub fn new(columns: Vec<String>) -> Self {
        let num_columns = &columns.len();

        let mut linear_layout = LinearLayout::vertical();
        linear_layout.add_child(TableHeaders::new(columns.iter()).full_width().max_height(1));
        linear_layout.add_child(
            SelectView::<T>::new()
                .full_width()
                .full_height()
                .scrollable(),
        );
        Self {
            linear_layout,
            last_layout_size: RefCell::new(XY::new(0, 0)),
            num_columns: *num_columns,
            inner: PhantomData::default(),
        }
    }

    pub fn replace_and_preserve_selection<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>,
    {
        // Get the index of the SelectView and unwrap it out of
        // ScrollView<ResizedView<ResizedView<SelectView<T>>>>
        let sv = self
            .linear_layout
            .get_child_mut(1)
            .and_then(|c| {
                c.as_any_mut()
                    .downcast_mut::<ScrollView<ResizedView<ResizedView<SelectView<T>>>>>()
            })
            .and_then(|v| Some(v.get_inner_mut()))
            .and_then(|v| Some(v.get_inner_mut()))
            .and_then(|v| Some(v.get_inner_mut()))
            .expect("Unable to get SelectView");

        let width = self.last_layout_size.borrow().x / self.num_columns;
        let current_selection = sv.selected_id().unwrap_or(0);

        sv.clear();

        for item in items {
            let presented = item.as_row();
            let presented = presented.iter().take(self.num_columns);
            let mut row = format!("");

            for field in presented {
                let mut truncated = field.clone();
                truncated.truncate(width - 1);
                row.push_str(format!("{:pad$}", truncated, pad = width).as_str());
            }

            sv.add_item(row, item);
        }

        sv.set_selection(current_selection);
    }
}

impl<T: 'static + TableListItem> ViewWrapper for TableListView<T> {
    wrap_impl!(self.linear_layout: LinearLayout);

    fn wrap_on_event(&mut self, ch: Event) -> EventResult {
        self.linear_layout.set_focus_index(1).expect("Must focus");
        self.linear_layout.on_event(ch)
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.last_layout_size.replace(size);
        self.linear_layout.layout(size);
    }
}
