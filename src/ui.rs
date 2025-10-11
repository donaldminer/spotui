use crate::app::{ActiveBlock, App, SelectedTab};
use crate::widgets::{
    nav_list::NavList, playlist::PlaylistWidget, top_artists::TopArtistsWidget,
    top_tracks::TopTracksWidget, user_playlists::UserPlaylistsWidget,
};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Clear, Tabs, Widget},
};
use tui_logger::{TuiLoggerLevelOutput, TuiLoggerWidget};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = Layout::new(
            Direction::Vertical,
            [Constraint::Percentage(10), Constraint::Percentage(90)],
        )
        .margin(0)
        .split(area);

        let content_layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(20), Constraint::Percentage(80)],
        )
        .margin(1)
        .split(main_layout[1]);
        self.render_tabs(main_layout[0], buf);
        match self.selected_tab {
            SelectedTab::Main => {
                self.render_directory(content_layout[0], buf);
                self.render_main_content(content_layout[1], buf);
                if matches!(self.route.active_block, ActiveBlock::Popup) {
                    self.render_track_popup(content_layout[1], buf);
                }
            }
            SelectedTab::Logger => {
                self.render_logger(main_layout[1], buf);
                return;
            }
        }
    }
}

impl App {
    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = ["Main", "Logger"];
        let selected = match self.selected_tab {
            SelectedTab::Main => 0,
            SelectedTab::Logger => 1,
        };
        let highlight_style = Style::default().fg(Color::Yellow);
        Tabs::new(titles)
            .select(selected)
            .highlight_style(highlight_style)
            .block(
                Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Tabs"),
            )
            .render(area, buf);
    }
    fn render_logger(&self, area: Rect, buf: &mut Buffer) {
        let state = &self.logger_state;

        TuiLoggerWidget::default()
            .block(Block::bordered().title("Logs"))
            .style(Style::default().fg(Color::White))
            .style_debug(Style::default().fg(Color::Green))
            .style_error(Style::default().fg(Color::Red))
            .style_warn(Style::default().fg(Color::Yellow))
            .output_separator('|')
            .output_timestamp(Some("%Y-%m-%d %H:%M:%S".to_string()))
            .output_level(Some(TuiLoggerLevelOutput::Abbreviated))
            .output_target(true)
            .output_file(false)
            .output_line(false)
            .state(&state)
            .render(area, buf);
    }

    fn render_track_popup(&self, area: Rect, buf: &mut Buffer) {
        let popup_block = NavList {
            title: self.track_popup.title.clone(),
            list: self.track_popup.list.clone(),
            list_state: self.track_popup.list_state.clone(),
            is_active: matches!(self.route.active_block, ActiveBlock::Popup),
        };

        let popup_area = Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 4,
            width: area.width / 4,
            height: popup_block.list.len() as u16 + 2,
        };
        Clear.render(popup_area, buf); // Clear the area first
        popup_block.render(popup_area, buf);
    }

    fn render_directory(&self, area: Rect, buf: &mut Buffer) {
        let directory = NavList {
            title: self.directory.title.clone(),
            list: self.directory.list.clone(),
            list_state: self.directory.list_state.clone(),
            is_active: matches!(self.route.active_block, ActiveBlock::Directory),
        };

        directory.render(area, buf);
    }
    fn render_main_content(&self, area: Rect, buf: &mut Buffer) {
        match self.route.hovered_block {
            ActiveBlock::UserPlaylists => {
                let user_playlists = UserPlaylistsWidget::new(
                    self.user_library.user_playlists.clone(),
                    matches!(self.route.active_block, ActiveBlock::UserPlaylists),
                );
                user_playlists.render(area, buf);
            }
            ActiveBlock::UserTopTracks => {
                let top_tracks = TopTracksWidget::new(
                    self.user_library.user_top_tracks.clone(),
                    matches!(self.route.active_block, ActiveBlock::UserTopTracks),
                );
                top_tracks.render(area, buf);
            }
            ActiveBlock::UserTopArtists => {
                let top_artists = TopArtistsWidget::new(
                    self.user_library.user_top_artists.clone(),
                    matches!(self.route.active_block, ActiveBlock::UserTopArtists),
                );
                top_artists.render(area, buf);
            }
            ActiveBlock::Playlist => {
                let playlist = PlaylistWidget::new(
                    self.playlist.clone(),
                    matches!(self.route.active_block, ActiveBlock::Playlist),
                );
                playlist.render(area, buf);
            }
            _ => { /* Do nothing */ }
        }
    }
}
