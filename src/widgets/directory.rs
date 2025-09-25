use crate::widgets::list::ListWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, Widget},
};

pub struct Directory {
    pub title: String,
    pub list: Vec<ListItem<'static>>,
    pub list_state: ratatui::widgets::ListState,
    pub is_active: bool,
}
impl Widget for Directory {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut directory = ListWidget {
            title: &self.title,
            list_items: self.list,
            list_state: self.list_state,
            is_active: self.is_active,
        };
        directory.render(area, buf);
    }
}
