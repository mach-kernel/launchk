use std::sync::RwLock;
use std::{collections::HashMap, sync::Arc};
use std::cmp::{max, min};
use std::ops::Deref;

/// Width oriented column sizing utility
pub struct ColumnSizer {
    /// Non user defined columns are an even split of space remaining from
    /// x - user_sizes_total
    pub dynamic_column_size: Arc<RwLock<usize>>,
    pub padding: Arc<RwLock<usize>>,
    /// Column index -> width
    pub user_sizes: HashMap<usize, usize>,
    pub num_columns: usize,

    num_dynamic_columns: usize,
    /// Sum of user size widths
    user_sizes_total: usize,
}

#[derive(Debug)]
pub enum ColumnSizerError {
    UpdateError,
    ReadError,
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
            padding: Arc::new(RwLock::new(1)),
        };

        Arc::new(cs)
    }

    /// Get the width for a column by index
    pub fn width_for_index(&self, i: usize) -> Result<usize, ColumnSizerError> {
        let size = self.user_sizes.get(&i).copied().unwrap_or(
            *self
                .dynamic_column_size
                .try_read()
                .map_err(|_| ColumnSizerError::ReadError)?,
        );

        // I have 'sized' my user defined columns around how much
        // space I need to just display the font, and the rest by
        // blindly dividing space, only apply padding to UDCs
        let size = if self.user_sizes.contains_key(&i) {
            size + *self
                .padding
                .try_read()
                .map_err(|_| ColumnSizerError::ReadError)?
        } else {
            size
        };

        let final_size = if size > 1 { size } else { 1 };

        Ok(final_size)
    }

    /// Call when x changes to recompute dynamic_column_size and padding
    pub fn update_x(&self, x: usize) -> Result<(), ColumnSizerError> {
        let current_padding = self.padding.read().unwrap().clone();
        let padding_used =
            (self.user_sizes.len() + self.num_dynamic_columns - 1) * current_padding;

        let remaining_dynamic = x
            .saturating_sub(padding_used)
            .saturating_sub(self.user_sizes_total);

        let new_dynamic_column_size = min(
            remaining_dynamic / self.num_dynamic_columns,
            32
        );

        let remaining = x
            .saturating_sub(self.num_dynamic_columns * new_dynamic_column_size)
            .saturating_sub(self.user_sizes_total);

        let new_padding = remaining / (self.num_dynamic_columns + self.user_sizes.len());

        match (
            self.dynamic_column_size.try_write(),
            self.padding.try_write(),
        ) {
            (Ok(mut dcs), Ok(mut pad)) => {
                *dcs = new_dynamic_column_size;
                *pad = new_padding;

                Ok(())
            }
            _ => Err(ColumnSizerError::UpdateError),
        }
    }
}
