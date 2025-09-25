use crate::widgets::list::ListWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, ListState, Widget},
};
use spotify_rs::model::playlist::SimplifiedPlaylist;

pub struct UserPlaylists {
    pub name: String,
    pub list: Vec<Option<SimplifiedPlaylist>>,
    pub list_state: ListState,
    pub is_active: bool,
}

impl Widget for UserPlaylists {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.list.is_empty() {
            let empty_block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("No Playlists Found");
            empty_block.render(area, buf);
            return;
        }
        let page = self.list.clone();
        let mut playlist = ListWidget {
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
        playlist.render(area, buf);
    }
}
