use cursive::theme::{BaseColor, Color, Effect, Style};
use cursive::view::ViewWrapper;
use cursive::views::SelectView;
use cursive::{Printer, Vec2, View, XY};
use std::cell::RefCell;

pub trait TableListItem {
    fn as_row(&self) -> Vec<String>;
}

pub struct TableListView<T: TableListItem> {
    columns: Vec<String>,
    select_view: SelectView<T>,
    last_layout_size: RefCell<XY<usize>>,
}

impl<T: 'static + TableListItem> TableListView<T> {
    pub fn new(columns: Vec<String>) -> Self {
        Self {
            columns,
            select_view: SelectView::new(),
            last_layout_size: RefCell::new(XY::new(0, 0)),
        }
    }

    pub fn replace_and_preserve_selection<I>(&mut self, items: I)
    where
        I: IntoIterator<Item = T>,
    {
        let width = self.last_layout_size.borrow().x / self.columns.len();
        let current_selection = self.select_view.selected_id().unwrap_or(0);
        self.select_view.clear();

        for item in items {
            let presented = item.as_row();
            let presented = presented.iter().take(self.columns.len());
            let mut row = format!("");

            for field in presented {
                let mut truncated = field.clone();
                truncated.truncate(width - 1);
                row.push_str(format!("{:pad$}", truncated, pad = width).as_str());
            }

            self.select_view.add_item(row, item);
        }

        self.select_view.set_selection(current_selection);
    }
}

impl<T: 'static + TableListItem> ViewWrapper for TableListView<T> {
    wrap_impl!(self.select_view: SelectView<T>);

    fn wrap_draw(&self, printer: &Printer<'_, '_>) {
        let width = self.last_layout_size.borrow().x / self.columns.len();

        let bold = Style::from(Color::Dark(BaseColor::Blue)).combine(Effect::Bold);
        let mut headers = format!("");

        for column in &self.columns {
            let mut truncated = column.clone();
            truncated.truncate(width - 1);
            headers.push_str(format!("{:pad$}", truncated, pad = width).as_str());
        }

        printer.with_style(bold, |p| p.print(XY::new(0, 0), headers.as_str()));

        // The headers take up one row
        let vertical_offset = XY::new(0, 1);
        let sub = printer
            .offset(vertical_offset)
            .content_offset(vertical_offset);
        self.select_view.draw(&sub);
    }

    fn wrap_layout(&mut self, size: Vec2) {
        self.last_layout_size.replace(size);
        self.select_view.layout(size);
    }
}
