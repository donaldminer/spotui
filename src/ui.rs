use crate::app::App;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    layout::{Constraint, Layout},
    symbols,
    widgets::{Block, Borders, Widget},
};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [left, right] =
            Layout::horizontal([Constraint::Percentage(49), Constraint::Percentage(51)])
                .areas(area);
        // use a 49/51 split to ensure that any extra space is on the bottom
        let [top_right, bottom_right] =
            Layout::vertical([Constraint::Percentage(49), Constraint::Percentage(51)]).areas(right);
        // ANCHOR_END: layout

        // ANCHOR: left_block
        let left_block = Block::new()
            // don't render the right border because it will be rendered by the right block
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .title("Left Block");
        // ANCHOR_END: left_block

        // ANCHOR: top_right_block
        // top right block must render the top left border to join with the left block
        let top_right_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.horizontal_down,
            ..symbols::border::PLAIN
        };
        let top_right_block = Block::new()
            .border_set(top_right_border_set)
            // don't render the bottom border because it will be rendered by the bottom block
            .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
            .title("Top Right Block");
        // ANCHOR_END: top_right_block

        // ANCHOR: bottom_right_block
        // bottom right block must render:
        // - top left border to join with the left block and top right block
        // - top right border to join with the top right block
        // - bottom left border to join with the left block
        let collapsed_top_and_left_border_set = symbols::border::Set {
            top_left: symbols::line::NORMAL.vertical_right,
            top_right: symbols::line::NORMAL.vertical_left,
            bottom_left: symbols::line::NORMAL.horizontal_up,
            ..symbols::border::PLAIN
        };
        let bottom_right_block = Block::new()
            .border_set(collapsed_top_and_left_border_set)
            .borders(Borders::ALL)
            .title("Bottom Right Block");
        // ANCHOR_END: bottom_right_block

        // ANCHOR: render
        left_block.render(left, buf);
        top_right_block.render(top_right, buf);
        bottom_right_block.render(bottom_right, buf);

        // frame.render_widget(left_block, left);
        // frame.render_widget(top_right_block, top_right);
        // frame.render_widget(bottom_right_block, bottom_right);
        // ANCHOR_END: render}
    }
}
