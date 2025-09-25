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
            title: self.user_library.directory.title.clone(),
            list: self.user_library.directory.list.clone(),
            list_state: self.user_library.directory.list_state.clone(),
            is_active: matches!(self.route.active_block, ActiveBlock::Directory),
        };
        directory.render(layout[0], buf);
        match self.route.hovered_block {
            ActiveBlock::UserPlaylists => {
                let user_playlists = UserPlaylists {
                    name: self.user_library.user_playlists.name.clone(),
                    list: self.user_library.user_playlists.list.clone(),
                    list_state: self.user_library.user_playlists.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::UserPlaylists),
                };
                user_playlists.render(layout[1], buf);
            }
            ActiveBlock::UserTopTracks => {
                let top_tracks = TopTracks {
                    name: self.user_library.user_top_tracks.name.clone(),
                    list: self.user_library.user_top_tracks.list.clone(),
                    list_state: self.user_library.user_top_tracks.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::UserTopTracks),
                };
                top_tracks.render(layout[1], buf);
            }
            ActiveBlock::UserTopArtists => {
                let top_artists = TopArtists {
                    name: self.user_library.user_top_artists.name.clone(),
                    list: self.user_library.user_top_artists.list.clone(),
                    list_state: self.user_library.user_top_artists.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::UserTopArtists),
                };
                top_artists.render(layout[1], buf);
            }
            ActiveBlock::Playlist => {
                let playlist = Playlist {
                    name: self.user_library.playlist.name.clone(),
                    list: self.user_library.playlist.list.clone(),
                    list_state: self.user_library.playlist.list_state.clone(),
                    is_active: matches!(self.route.active_block, ActiveBlock::Playlist),
                };
                playlist.render(layout[1], buf);
            }
            _ => { /* Do nothing */ }
        }
    }
}
