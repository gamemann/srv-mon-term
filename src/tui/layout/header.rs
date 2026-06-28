use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::tui::{interface::types::TuiInterfaceType, types::Tui};

impl Tui {
    pub fn draw_header(frame: &mut Frame<'_>, area: Rect, active: TuiInterfaceType) {
        let tabs = [
            (TuiInterfaceType::Dashboard, "F1", "Dashboard"),
            (TuiInterfaceType::Settings, "F2", "Settings"),
            (TuiInterfaceType::Logs, "F3", "Logs"),
        ];

        let mut spans = vec![Span::styled(
            " srv-mon  ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )];

        for (iface, key, label) in &tabs {
            let is_active = *iface == active;

            let key_span = Span::styled(
                format!("[{}]", key),
                Style::default()
                    .fg(if is_active {
                        Color::Black
                    } else {
                        Color::DarkGray
                    })
                    .bg(if is_active { Color::Cyan } else { Color::Reset }),
            );
            let label_span = Span::styled(
                format!(" {} ", label),
                Style::default()
                    .fg(if is_active {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    })
                    .add_modifier(if is_active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            );

            spans.push(key_span);
            spans.push(label_span);
            spans.push(Span::raw(" "));
        }

        let line = Line::from(spans);

        frame.render_widget(
            Paragraph::new(line).style(Style::default().bg(Color::Reset)),
            area,
        );
    }
}
