use crate::app::{ActiveBlock, App};
use crate::widgets::{
    directory::Directory, playlist::Playlist, top_artists::TopArtists, top_tracks::TopTracks,
    user_playlists::UserPlaylists,
};
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Layout, Rect},
    widgets::Widget,
};
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    ratatui::layout::Constraint::Percentage(20),
                    ratatui::layout::Constraint::Percentage(80),
                ]
                .as_ref(),
            )
            .split(area);

        let directory = Directory {
            title: self.directory.title.clone(),
            list: self.directory.list.clone(),
            list_state: self.directory.list_state.clone(),
            is_active: matches!(self.route.active_block, ActiveBlock::Directory),
        };
        directory.render(layout[0], buf);
        match self.route.hovered_block {
            ActiveBlock::UserPlaylists => {
                let user_playlists = UserPlaylists {
                    name: "User Playlists".to_string(),
                    list: self.user_library.user_playlists.list.clone(),
                    list_state: self.user_library.user_playlists.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::UserPlaylists),
                };
                user_playlists.render(layout[1], buf);
            }
            ActiveBlock::UserTopTracks => {
                let top_tracks = TopTracks {
                    name: "User Top Tracks".to_string(),
                    list: self.user_library.user_top_tracks.list.clone(),
                    list_state: self.user_library.user_top_tracks.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::UserTopTracks),
                };
                top_tracks.render(layout[1], buf);
            }
            ActiveBlock::UserTopArtists => {
                let top_artists = TopArtists {
                    name: "User Top Artists".to_string(),
                    list: self.user_library.user_top_artists.list.clone(),
                    list_state: self.user_library.user_top_artists.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::UserTopArtists),
                };
                top_artists.render(layout[1], buf);
            }
            ActiveBlock::Playlist => {
                let playlist = Playlist {
                    name: self
                        .playlist
                        .result
                        .as_ref()
                        .map_or("Playlist", |p| p.name.as_str())
                        .to_string(),
                    list: self.playlist.pages.list.clone(),
                    list_state: self.playlist.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::Playlist),
                };
                playlist.render(layout[1], buf);
            }
            _ => { /* Do nothing */ }
        }
    }
}
