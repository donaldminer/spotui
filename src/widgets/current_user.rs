use crate::widgets::list::ListWidget;
use ratatui::{
    buffer::Buffer,
    layout::{Layout, Rect},
    text::Line,
    widgets::{Block, Borders, ListItem, Widget},
};
use spotify_rs::model::{Page, artist::Artist, track::Track};

pub struct CurrentUser {
    pub name: String,
    pub product: String,
    pub top_artists: Page<Artist>,
    pub top_tracks: Page<Track>,
    pub is_active: bool,
}
impl Widget for CurrentUser {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title_top(format!("{}", self.name))
            .title_top(Line::from(format!("{}", self.product)).right_aligned());

        let artist_page = self.top_artists.clone();
        let track_page = self.top_tracks.clone();
        let mut artists = ListWidget {
            title: "Top Artists",
            list_items: artist_page
                .items
                .into_iter()
                .map(|a| ListItem::new(a.unwrap().name))
                .collect(),
            list_state: Default::default(),
            is_active: self.is_active,
        };
        let mut tracks = ListWidget {
            title: "Top Tracks",
            list_items: track_page
                .items
                .into_iter()
                .map(|t| ListItem::new(t.unwrap().name))
                .collect(),
            list_state: Default::default(),
            is_active: self.is_active,
        };

        let inner = block.inner(area);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints(
                [
                    ratatui::layout::Constraint::Percentage(50),
                    ratatui::layout::Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(inner);

        block.render(area, buf);
        artists.render(layout[0], buf);
        tracks.render(layout[1], buf);
    }
}
