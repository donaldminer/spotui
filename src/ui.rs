use crate::app::App;
// use crate::widgets::{current_user::CurrentUser, user_playlist::UserPlaylist};
use crate::widgets::user_playlist::UserPlaylist;
use ratatui::{
    buffer::Buffer,
    layout::{Direction, Layout, Rect},
    // style::{Color, Style},
    widgets::Widget,
};
use tui_logger::*;

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                [
                    // ratatui::layout::Constraint::Percentage(20),
                    // ratatui::layout::Constraint::Percentage(50),
                    // ratatui::layout::Constraint::Percentage(30),
                    ratatui::layout::Constraint::Percentage(50),
                    ratatui::layout::Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(area);

        let user_playlists = UserPlaylist {
            name: "My Playlist".to_string(),
            list: self.current_user.user_playlists.playlists.clone(),
            list_state: self.current_user.user_playlists.playlists_state.clone(),
        };
        // let user_profile = CurrentUser {
        //     name: self.current_user.user_profile.display_name.clone(),
        //     product: self.current_user.user_profile.product.clone(),
        //     top_artists: self.current_user.user_top_items.artists.clone(),
        //     top_tracks: self.current_user.user_top_items.tracks.clone(),
        // };
        //
        // match &self.active_widget {
        //     crate::app::ActiveWidget::UserProfile => {
        //         // Highlight current user section
        //         let highlight_block = ratatui::widgets::Block::default()
        //             .borders(ratatui::widgets::Borders::ALL)
        //             .border_style(ratatui::style::Style::default());
        //         highlight_block.render(layout[0], buf);
        //     }
        //     crate::app::ActiveWidget::UserPlaylists => {
        //         // Highlight user playlists section
        //         let highlight_block = ratatui::widgets::Block::default()
        //             .borders(ratatui::widgets::Borders::ALL)
        //             .border_style(
        //                 ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
        //             );
        //         highlight_block.render(layout[1], buf);
        //     }
        //     _ => {}
        // }
        let smart_logger = TuiLoggerWidget::default();
        smart_logger.render(layout[0], buf);

        // user_profile.render(layout[0], buf);
        user_playlists.render(layout[1], buf);
    }
}
