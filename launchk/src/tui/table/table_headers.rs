use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::{Printer, Vec2, View, XY};
use std::cell::RefCell;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::collections::HashMap;

pub struct TableHeaders {
    columns: Vec<String>,
    user_col_sizes: Arc<(HashMap<usize, usize>, usize)>,
    dynamic_col_sz_rx: Receiver<(usize, usize)>,
    dynamic_cols_sz: RefCell<(usize, usize)>,
}

impl TableHeaders {
    pub fn new<S: Into<String>>(
        columns: impl Iterator<Item = S>,
        user_col_sizes: Arc<(HashMap<usize, usize>, usize)>,
        dynamic_col_sz_rx: Receiver<(usize, usize)>,
    ) -> Self {
        Self {
            columns: columns.map(|f| f.into()).collect(),
            dynamic_cols_sz: RefCell::new((0, 0)),
            user_col_sizes,
            dynamic_col_sz_rx,
        }
    }
}

impl View for TableHeaders {
    fn draw(&self, printer: &Printer<'_, '_>) {
        if let Ok(dcs) = self.dynamic_col_sz_rx.try_recv() {
            self.dynamic_cols_sz.replace(dcs);
        }
        
        let bold = Style::from(Color::Dark(BaseColor::Blue)).combine(Effect::Bold);

        let (dyn_max, padding) = *self.dynamic_cols_sz.borrow();
        if dyn_max < 1 {
            return;
        }

        let (ucs, _) = &*self.user_col_sizes;

        let headers: String = self.columns.iter().enumerate().map(|(i, column)| {
            let width = ucs
                .get(&i)
                .map(|s| s.clone())
                .unwrap_or(dyn_max);

            let pad = if ucs.contains_key(&i) {
                width + padding
            } else {
                width
            };

            format!("{:pad$}", column, pad = pad)
        }).collect::<Vec<String>>().join("");

        printer.with_style(bold, |p| p.print(XY::new(0, 0), headers.as_str()));
    }
}
