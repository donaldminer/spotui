use crate::widgets::list::ListWidget;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, ListState, Widget},
};
use spotify_rs::model::{Page, playlist::SimplifiedPlaylist};

pub struct UserPlaylist {
    pub name: String,
    pub list: Option<Page<SimplifiedPlaylist>>,
    pub list_state: ListState,
}

impl Widget for UserPlaylist {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.list.is_none() {
            let empty_block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("No Playlists Found");
            empty_block.render(area, buf);
            return;
        }
        let page = self.list.unwrap();
        let mut playlist = ListWidget {
            title: &format!("Playlist: {}", self.name),
            list_items: page
                .items
                .into_iter()
                .map(|p| {
                    let name = p.unwrap().name.clone();
                    ListItem::new(name)
                })
                .collect(),
            list_state: self.list_state,
        };
        playlist.render(area, buf);
    }
}
