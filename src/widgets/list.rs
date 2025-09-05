use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier},
    widgets::{Block, Borders, List, ListItem, ListState, StatefulWidget, Widget},
};

pub struct ListWidget<'a> {
    pub title: &'a str,
    pub list_items: Vec<ListItem<'a>>,
    pub list_state: ListState,
}

impl Widget for &mut ListWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // let items = List::new(self.list_items.clone())
        //     .block(Block::new().borders(Borders::NONE).title(self.title));
        let items: Vec<ListItem> = self.list_items.iter().cloned().collect();
        let list = List::new(items)
            .block(Block::new().borders(Borders::ALL).title(self.title))
            .highlight_style(
                ratatui::style::Style::default()
                    .fg(Color::Yellow)
                    .bg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        StatefulWidget::render(list, area, buf, &mut self.list_state);
    }
}
