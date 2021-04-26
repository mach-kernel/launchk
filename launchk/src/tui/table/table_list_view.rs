use crate::tui::table::table_headers::TableHeaders;
use cursive::event::{Event, EventResult, Key};
use cursive::traits::{Resizable, Scrollable};
use cursive::view::ViewWrapper;
use cursive::views::{LinearLayout, ResizedView, ScrollView, SelectView};
use cursive::{Vec2, View, XY};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel, Sender};
use std::rc::Rc;

pub trait TableListItem {
    fn as_row(&self) -> Vec<String>;
}

pub struct TableListView<T> {
    linear_layout: LinearLayout,
    last_layout_size: RefCell<XY<usize>>,
    num_columns: usize,
    // User override cols, total size
    user_col_sizes: Arc<(HashMap<usize, usize>, usize)>,
    // Precompute dynamic sizes once on replace
    // (Max dynamic col size, Padding between columns)
    dynamic_cols_sz: RefCell<(usize, usize)>,
    // Share it
    dynamic_cols_sz_tx: Sender<(usize, usize)>,
    // Don't swallow type, presumably needed later
    inner: PhantomData<T>,
}

impl<T: 'static + TableListItem> TableListView<T> {
    fn build_user_col_sizes(columns: &Vec<(&str, Option<usize>)>) -> Arc<(HashMap<usize, usize>, usize)> {
        let mut user_col_sizes: HashMap<usize, usize> = HashMap::new();
        let mut user_col_size_total: usize = 0;

        for (i, (_, sz)) in columns.iter().enumerate() {
            if sz.is_none() { continue; }
            let sz = sz.unwrap();
            user_col_size_total += sz;
            user_col_sizes.insert(i, sz);
        }

        Arc::new((user_col_sizes, user_col_size_total))
    }

    pub fn new(columns: Vec<(&str, Option<usize>)>) -> Self {
        let (dynamic_cols_sz_tx, rx): (Sender<(usize, usize)>, Receiver<(usize, usize)>) = channel();
        let user_col_sizes = Self::build_user_col_sizes(&columns);

        let mut linear_layout = LinearLayout::vertical();
        linear_layout.add_child(TableHeaders::new(columns.iter().map(|(n, _)| n.to_string()), user_col_sizes.clone(), rx).full_width().max_height(1));
        linear_layout.add_child(
            SelectView::<T>::new()
                .full_width()
                .full_height()
                .scrollable(),
        );

        Self {
            linear_layout,
            user_col_sizes,
            dynamic_cols_sz_tx,
            dynamic_cols_sz: RefCell::new((0, 0)),
            last_layout_size: RefCell::new(XY::new(0, 0)),
            num_columns: *&columns.len(),
            inner: PhantomData::default(),
        }
    }

    pub fn replace_and_preserve_selection<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>,
    {
        // self.compute_sizes();

        let (dyn_max, padding) = *self.dynamic_cols_sz.borrow();
        let (user_col_sizes, _) = &*self.user_col_sizes;

        let rows: Vec<(String, T)> = items.into_iter().map(|item: T| {
            let presented: Vec<String> = item
                .as_row()
                .iter()
                .take(self.num_columns)
                .enumerate()
                .map(|(i, field)| {
                    let mut truncated = field.clone();
                    let field_width = user_col_sizes
                        .get(&i)
                        .clone()
                        .map(|s| s.clone())
                        .unwrap_or(dyn_max.clone());

                    let pad = if user_col_sizes.contains_key(&i) {
                        field_width + padding
                    } else {
                        field_width
                    };

                    truncated.truncate(field_width - 1);
                    format!("{:pad$}", truncated, pad = pad)
                })
                .collect();

            (presented.join(""), item)
        }).collect();

        let sv = self.get_mut_selectview();
        let current_selection = sv.selected_id().unwrap_or(0);

        sv.clear();
        sv.add_all(rows);
        sv.set_selection(current_selection);
    }

    pub fn get_highlighted_row(&self) -> Option<Rc<T>> {
        self.get_selectview().selection()
    }

    /// "Responsive"
    fn compute_sizes(&mut self) {
        let (user_col_sizes, user_col_sizes_total) = &*self.user_col_sizes;

        let num_dynamic = self.num_columns - user_col_sizes.len();

        // All sizes are static
        if num_dynamic < 1 {
            return
        }

        let remaining = self.last_layout_size.borrow().x - user_col_sizes_total;
        let mut per_dynamic_col = remaining / num_dynamic;

        // Max col sz = 35
        if per_dynamic_col > 35 {
            per_dynamic_col = 35;
        }

        // After user col reservations, remove dyn cols, and distribute that space btw
        // the user provided column sizes.
        let remain_padding = (remaining - (per_dynamic_col * num_dynamic)) / (self.num_columns - num_dynamic);
        self.dynamic_cols_sz.replace((per_dynamic_col, remain_padding));
        self.dynamic_cols_sz_tx.send((per_dynamic_col, remain_padding)).expect(
            "Must update dynamic cols"
        );
    }

    /// Get the index of the SelectView and unwrap it out of
    /// ScrollView<ResizedView<ResizedView<SelectView<T>>>>
    fn get_mut_selectview(&mut self) -> &mut SelectView<T> {
        self
            .linear_layout
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
        self
            .linear_layout
            .get_child(1)
            .and_then(|c| {
                c.as_any().downcast_ref::<ScrollView<ResizedView<ResizedView<SelectView<T>>>>>()
            })
            .and_then(|v| Some(v.get_inner()))
            .and_then(|v| Some(v.get_inner()))
            .and_then(|v| Some(v.get_inner()))
            .expect("Unable to get SelectView")
    }
}

impl<T: 'static + TableListItem> ViewWrapper for TableListView<T> {
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
        self.last_layout_size.replace(size);
        self.linear_layout.layout(size);
        self.compute_sizes();
    }
}
