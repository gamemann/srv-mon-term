use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::server::types::user::ServerUser;

pub fn draw_server_users(
    frame: &mut Frame<'_>,
    area: Rect,
    users: &[ServerUser],
    selected: usize,
    focused: bool,
    is_error: bool,
    err_code: Option<u16>,
) {
    if is_error {
        let err = Paragraph::new(Span::styled(
            format!("Error fetching users list. Code: {:?}", err_code),
            Style::default().fg(Color::Red),
        ));

        frame.render_widget(err, area);

        return;
    }
    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(" Users ({}) ", users.len()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if users.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No users connected",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );

        return;
    }

    // Sort by score descending for a scoreboard-style display.
    let mut sorted: Vec<&ServerUser> = users.iter().collect();
    sorted.sort_by(|a, b| b.score.cmp(&a.score));

    // Not every protocol reports a ping or a connection time, so only show what we have.
    let has_ping = sorted.iter().any(|u| u.ping.is_some());
    let has_duration = sorted.iter().any(|u| u.duration > 0);

    let mut header_cells = vec![Cell::from("Name"), Cell::from("Score")];
    let mut widths = vec![Constraint::Min(12), Constraint::Length(7)];

    if has_ping {
        header_cells.push(Cell::from("Ping"));
        widths.push(Constraint::Length(6));
    }

    if has_duration {
        header_cells.push(Cell::from("Time"));
        widths.push(Constraint::Length(10));
    }

    let header = Row::new(header_cells).style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = sorted
        .iter()
        .map(|u| {
            let mut cells = vec![Cell::from(u.name.clone()), Cell::from(u.score.to_string())];

            if has_ping {
                cells.push(Cell::from(
                    u.ping.map(|p| p.to_string()).unwrap_or("-".to_string()),
                ));
            }

            if has_duration {
                cells.push(Cell::from(format_duration(u.duration)));
            }

            Row::new(cells)
        })
        .collect();

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
        .highlight_symbol("▶ ");

    let mut state = TableState::default();
    let clamped = selected.min(sorted.len().saturating_sub(1));
    state.select(Some(clamped));

    frame.render_stateful_widget(table, inner, &mut state);
}

fn format_duration(seconds: u64) -> String {
    let h = seconds / 3600;
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;

    if h > 0 {
        format!("{:02}:{:02}:{:02}", h, m, s)
    } else {
        format!("{:02}:{:02}", m, s)
    }
}
