use std::sync::Arc;

use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::{Printer, View, XY};

use super::column_sizer::ColumnSizer;

/// Draw column headers from their names + a column sizer
pub struct TableHeaders {
    columns: Vec<String>,
    column_sizer: Arc<ColumnSizer>,
}

impl TableHeaders {
    pub fn new<S: Into<String>>(
        columns: impl Iterator<Item = S>,
        column_sizer: Arc<ColumnSizer>,
    ) -> Self {
        Self {
            columns: columns.map(|f| f.into()).collect(),
            column_sizer,
        }
    }
}

impl View for TableHeaders {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let bold = Style::from(Color::Dark(BaseColor::Blue)).combine(Effect::Bold);

        let headers: String = self
            .columns
            .iter()
            .enumerate()
            .map(|(i, column)| {
                format!(
                    "{:with_padding$}",
                    column,
                    with_padding = self.column_sizer.width_for_index(i)
                )
            })
            .collect::<Vec<String>>()
            .join("");

        printer.with_style(bold, |p| p.print(XY::new(0, 0), headers.as_str()));
    }
}
