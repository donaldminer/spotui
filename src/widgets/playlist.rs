use crate::{app::TrackList, widgets::list::ListWidget};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, Widget},
};
use spotify_rs::model::{PlayableItem, playlist::Playlist, playlist::PlaylistItem};

pub struct PlaylistWidget {
    pub name: String,
    pub list: Vec<Option<PlaylistItem>>,
    pub list_state: ratatui::widgets::ListState,
    pub is_active: bool,
}
impl PlaylistWidget {
    pub fn new(playlist: TrackList<Playlist, PlaylistItem>, active: bool) -> Self {
        Self {
            name: playlist
                .result
                .as_ref()
                .map_or("Playlist", |p| p.name.as_str())
                .to_string(),
            list: playlist.pages.list,
            list_state: playlist.list_state,
            is_active: active,
        }
    }
}
impl Widget for PlaylistWidget {
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
