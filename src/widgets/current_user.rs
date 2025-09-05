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
    pub top_artists: Option<Page<Artist>>,
    pub top_tracks: Option<Page<Track>>,
}
impl Widget for CurrentUser {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .borders(Borders::ALL)
            .title_top(format!("{}", self.name))
            .title_top(Line::from(format!("{}", self.product)).right_aligned());
        let mut artists = ListWidget {
            title: "Top Artists",
            list_items: self
                .top_artists
                .unwrap()
                .items
                .into_iter()
                .map(|a| ListItem::new(a.unwrap().name))
                .collect(),
            list_state: Default::default(),
        };
        let mut tracks = ListWidget {
            title: "Top Tracks",
            list_items: self
                .top_tracks
                .unwrap()
                .items
                .into_iter()
                .map(|t| ListItem::new(t.unwrap().name))
                .collect(),
            list_state: Default::default(),
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
