use crate::widgets::list::ListWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, Widget},
};
use spotify_rs::model::track::Track;

pub struct TopTracks {
    pub name: String,
    pub list: Vec<Option<Track>>,
    pub list_state: ratatui::widgets::ListState,
    pub is_active: bool,
}
impl Widget for TopTracks {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.list.is_empty() {
            let empty_block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("No Artists Found");
            empty_block.render(area, buf);
            return;
        }
        let page = self.list.clone();
        let mut top_tracks = ListWidget {
            title: &format!("{}", self.name),
            list_items: page
                .into_iter()
                .map(|p| {
                    let name = p.unwrap().name.clone();
                    ListItem::new(name)
                })
                .collect(),
            list_state: self.list_state,
            is_active: self.is_active,
        };
        top_tracks.render(area, buf);
    }
}
