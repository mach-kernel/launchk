use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use std::collections::hash_map::DefaultHasher;
use std::sync::{Arc, RwLock};

use cursive::event::{Event, EventResult};
use cursive::traits::{Resizable, Scrollable};
use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, ResizedView, ScrollView, SelectView};
use cursive::{Vec2, View};

use crate::tui::table::table_headers::TableHeaders;

use super::column_sizer::ColumnSizer;
pub trait TableListItem {
    fn as_row(&self) -> Vec<String>;
}

/// A "table" implemented on top of SelectView<T> where we
/// divvy up x into columns
pub struct TableListView<T> {
    column_sizer: Arc<ColumnSizer>,
    linear_layout: LinearLayout,
    // LinearLayout swallows T from , but we still need it
    inner: PhantomData<T>,
    last_hash: Arc<RwLock<u64>>,
}

impl<T: 'static + TableListItem + Send + Sync> TableListView<T> {
    pub fn new<I, K>(columns: I) -> TableListView<T>
    where
        I: IntoIterator<Item = (K, Option<usize>)> + Clone,
        K: AsRef<str>,
    {
        let column_names = columns
            .clone()
            .into_iter()
            .map(|(n, _)| n.as_ref().to_string());
        let column_sizer = ColumnSizer::new(columns);
        let last_hash = Arc::new(RwLock::new(0u64));

        let mut linear_layout = LinearLayout::vertical();
        linear_layout.add_child(
            TableHeaders::new(column_names, column_sizer.clone())
                .full_width()
                .max_height(1),
        );
        linear_layout.add_child(
            SelectView::<T>::new()
                .full_width()
                .full_height()
                .scrollable(),
        );

        Self {
            linear_layout,
            column_sizer,
            inner: PhantomData::default(),
            last_hash,
        }
    }

    pub fn replace_and_preserve_selection<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>,
        T: Hash,
    {
        let rows: Vec<(String, T)> = items
            .into_iter()
            .map(|item: T| {
                let presented: Vec<String> = item
                    .as_row()
                    .iter()
                    .take(self.column_sizer.num_columns)
                    .enumerate()
                    .map(|(i, field)| {
                        let wfi = self.column_sizer.width_for_index(i).unwrap_or(1);
                        let mut truncated = field.clone();
                        truncated.truncate(wfi - 1);
                        format!("{:with_padding$}", truncated, with_padding = wfi)
                    })
                    .collect();

                (presented.join(""), item)
            })
            .collect();

        let mut row_hasher = DefaultHasher::new();
        rows.hash(&mut row_hasher);
        let hash = row_hasher.finish();

        match self.last_hash.try_read() {
            Ok(lh) => {
                if *lh == hash { return; }
            }
            _ => {}
        }

        log::trace!("Replaced listview items -- new hash {}", hash);

        match self.last_hash.try_write() {
            Ok(mut lh) => *lh = hash,
            _ => {}
        }

        let sv = self.get_mut_selectview();
        let current_selection = sv.selected_id().unwrap_or(0);

        sv.clear();
        sv.add_all(rows);
        sv.set_selection(current_selection);
    }

    pub fn get_highlighted_row(&self) -> Option<Arc<T>> {
        self.get_selectview().selection()
    }

    /// Get the index of the SelectView and unwrap it out of
    /// ScrollView<ResizedView<ResizedView<SelectView<T>>>>
    fn get_mut_selectview(&mut self) -> &mut SelectView<T> {
        self.linear_layout
            .get_child_mut(1)
            .and_then(|c| {
                c.as_any_mut()
                    .downcast_mut::<ScrollView<ResizedView<ResizedView<SelectView<T>>>>>()
            })
            .and_then(|v| Some(v.get_inner_mut()))
            .and_then(|v| Some(v.get_inner_mut()))
            .and_then(|v| Some(v.get_inner_mut()))
            .expect("Unable to get SelectView")
    }

    fn get_selectview(&self) -> &SelectView<T> {
        self.linear_layout
            .get_child(1)
            .and_then(|c| {
                c.as_any()
                    .downcast_ref::<ScrollView<ResizedView<ResizedView<SelectView<T>>>>>()
            })
            .and_then(|v| Some(v.get_inner()))
            .and_then(|v| Some(v.get_inner()))
            .and_then(|v| Some(v.get_inner()))
            .expect("Unable to get SelectView")
    }
}

impl<T: 'static + TableListItem + Send + Sync> ViewWrapper for TableListView<T> {
    wrap_impl!(self.linear_layout: LinearLayout);

    fn wrap_on_event(&mut self, event: Event) -> EventResult {
        if self.get_selectview().is_empty() {
            EventResult::Consumed(None)
        } else {
            self.linear_layout.set_focus_index(1).expect("Must focus");
            self.linear_layout.on_event(event)
        }
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.column_sizer.update_x(size.x).expect("Must update");
        self.linear_layout.layout(size);
    }
}
