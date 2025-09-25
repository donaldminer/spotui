use crate::widgets::list::ListWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, Widget},
};
use spotify_rs::model::{PlayableItem, playlist::PlaylistItem};

pub struct Playlist {
    pub name: String,
    pub list: Vec<Option<PlaylistItem>>,
    pub list_state: ratatui::widgets::ListState,
    pub is_active: bool,
}
impl Widget for Playlist {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.list.is_empty() {
            let empty_block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("No Artists Found");
            empty_block.render(area, buf);
            return;
        }
        let page = self.list.clone();
        let mut playlist = ListWidget {
            title: &format!("{}", self.name),
            list_items: page
                .into_iter()
                .map(|p| {
                    let name = p.unwrap().track.clone();
                    ListItem::new(match name {
                        PlayableItem::Track(t) => t.name,
                        PlayableItem::Episode(e) => e.name,
                    })
                })
                .collect(),
            list_state: self.list_state,
            is_active: self.is_active,
        };
        playlist.render(area, buf);
    }
}
