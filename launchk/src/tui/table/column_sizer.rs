use std::{cell::Cell, collections::HashMap, sync::Arc};

/// Width oriented column sizing utility
pub struct ColumnSizer {
    /// Non user defined columns are an even split of space remaining from
    /// x - user_sizes_total
    pub dynamic_column_size: Cell<usize>,
    /// TODO; wtf do I mean by padding
    pub padding: Cell<usize>,
    /// Column index -> width
    pub user_sizes: HashMap<usize, usize>,
    pub num_columns: usize,

    num_dynamic_columns: usize,
    /// Sum of user size widths
    user_sizes_total: usize,
}

impl ColumnSizer {
    /// Create a new ColumnSizer
    pub fn new<I, K>(columns: I) -> Arc<Self>
    where
        I: IntoIterator<Item = (K, Option<usize>)> + Clone,
        K: AsRef<str>,
    {
        let num_columns = columns.clone().into_iter().count();
        let column_iter = columns.into_iter();

        let user_sizes: HashMap<usize, usize> = column_iter
            .zip(0..num_columns)
            .filter_map(|((_, user_len), i)| user_len.map(|ul| (i, ul)))
            .collect();

        let user_sizes_total = user_sizes.values().sum();
        let num_dynamic_columns = num_columns - user_sizes.len();

        let cs = Self {
            num_dynamic_columns,
            num_columns,
            user_sizes,
            user_sizes_total,
            dynamic_column_size: Default::default(),
            padding: Default::default(),
        };

        Arc::new(cs)
    }

    /// Get the width for a column by index
    pub fn width_for_index(&self, i: usize) -> usize {
        let size = self
            .user_sizes
            .get(&i)
            .map(Clone::clone)
            .unwrap_or(self.dynamic_column_size.get());

        // I have 'sized' my user defined columns around how much
        // space I need to just display the font, and the rest by
        // blindly dividing space, only apply padding to UDCs

        if self.user_sizes.contains_key(&i) {
            size + self.padding.get()
        } else {
            size
        }
    }

    /// Call when x changes to recompute dynamic_column_size and padding
    pub fn update_x(&self, x: usize) {
        let mut remaining = x - self.user_sizes_total;

        let mut dcs = remaining / self.num_dynamic_columns;
        if dcs > 35 {
            dcs = 35;
        }

        remaining = remaining - (self.num_dynamic_columns * dcs);

        self.dynamic_column_size.set(dcs);
        self.padding
            .set(remaining / (self.num_dynamic_columns + self.user_sizes.len()));
    }
}
