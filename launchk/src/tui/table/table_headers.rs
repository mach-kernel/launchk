use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::{Printer, Vec2, View, XY};
use std::cell::RefCell;

pub struct TableHeaders {
    columns: Vec<String>,
    last_layout_size: RefCell<XY<usize>>,
}

impl TableHeaders {
    pub fn new<S: Into<String>>(columns: impl Iterator<Item = S>) -> Self {
        Self {
            columns: columns.map(|f| f.into()).collect(),
            last_layout_size: RefCell::new(XY::new(0, 0)),
        }
    }
}

impl View for TableHeaders {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let width = self.last_layout_size.borrow().x / self.columns.len();
        let bold = Style::from(Color::Dark(BaseColor::Blue)).combine(Effect::Bold);

        let mut headers = format!("");

        for column in &self.columns {
            let mut truncated = column.clone();
            truncated.truncate(width - 1);
            headers.push_str(format!("{:<pad$}", truncated, pad = width).as_str());
        }

        printer.with_style(bold, |p| p.print(XY::new(0, 0), headers.as_str()));
    }

    fn layout(&mut self, sz: Vec2) {
        self.last_layout_size.replace(sz);
    }
}
