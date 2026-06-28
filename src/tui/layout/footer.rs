use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::types::Tui;

impl Tui {
    pub fn draw_footer(frame: &mut Frame<'_>, area: Rect, bindings: &[(&str, &str)]) {
        let mut spans: Vec<Span> = Vec::new();

        for (key, desc) in bindings {
            spans.push(Span::styled(
                format!("[{}]", key),
                Style::default().fg(Color::DarkGray),
            ));
            spans.push(Span::styled(
                format!(" {} ", desc),
                Style::default().fg(Color::Gray),
            ));
            spans.push(Span::raw(" "));
        }

        frame.render_widget(Paragraph::new(Line::from(spans)), area);
    }
}
