use crate::{app::PageEndpoint, widgets::list::ListWidget};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{ListItem, Widget},
};
use spotify_rs::model::artist::Artist;

pub struct TopArtistsWidget {
    pub name: String,
    pub list: Vec<Option<Artist>>,
    pub list_state: ratatui::widgets::ListState,
    pub is_active: bool,
}

impl TopArtistsWidget {
    pub fn new(top_artists: PageEndpoint<Artist>, active: bool) -> Self {
        Self {
            name: "User Top Artists".to_string(),
            list: top_artists.list,
            list_state: top_artists.list_state,
            is_active: active,
        }
    }
}
impl Widget for TopArtistsWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if self.list.is_empty() {
            let empty_block = ratatui::widgets::Block::default()
                .borders(ratatui::widgets::Borders::ALL)
                .title("No Artists Found");
            empty_block.render(area, buf);
            return;
        }
        let page = self.list.clone();
        let mut top_artists = ListWidget {
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
        top_artists.render(area, buf);
    }
}
