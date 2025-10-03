use crate::app::{ActiveBlock, App};
use crate::widgets::{
    directory::Directory, playlist::PlaylistWidget, top_artists::TopArtistsWidget,
    top_tracks::TopTracksWidget, user_playlists::UserPlaylistsWidget,
};
use ratatui::layout::Constraint;
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Layout, Rect},
    widgets::Widget,
};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(20), Constraint::Percentage(80)],
        )
        .margin(0)
        .split(area);

        self.render_main_content(main_layout[1], buf);
    }
}

impl App {
    fn render_main_content(&self, area: Rect, buf: &mut Buffer) {
        let content_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(20), Constraint::Percentage(80)],
        )
        .margin(1)
        .split(area);
        let directory = Directory {
            title: self.directory.title.clone(),
            list: self.directory.list.clone(),
            list_state: self.directory.list_state.clone(),
            is_active: matches!(self.route.active_block, ActiveBlock::Directory),
        };

        directory.render(content_layout[0], buf);

        match self.route.hovered_block {
            ActiveBlock::UserPlaylists => {
                let user_playlists = UserPlaylistsWidget::new(
                    self.user_library.user_playlists.clone(),
                    matches!(self.route.active_block, ActiveBlock::UserPlaylists),
                );
                user_playlists.render(content_layout[1], buf);
            }
            ActiveBlock::UserTopTracks => {
                let top_tracks = TopTracksWidget::new(
                    self.user_library.user_top_tracks.clone(),
                    matches!(self.route.active_block, ActiveBlock::UserTopTracks),
                );
                top_tracks.render(content_layout[1], buf);
            }
            ActiveBlock::UserTopArtists => {
                let top_artists = TopArtistsWidget::new(
                    self.user_library.user_top_artists.clone(),
                    matches!(self.route.active_block, ActiveBlock::UserTopArtists),
                );
                top_artists.render(content_layout[1], buf);
            }
            ActiveBlock::Playlist => {
                let playlist = PlaylistWidget::new(
                    self.playlist.clone(),
                    matches!(self.route.active_block, ActiveBlock::Playlist),
                );
                playlist.render(content_layout[1], buf);
            }
            _ => { /* Do nothing */ }
        }
    }
}
