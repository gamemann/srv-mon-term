use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{server::Server, util::truncate_str};

pub fn draw_server_info(frame: &mut Frame<'_>, area: Rect, server: &Server, is_online: bool) {
    let data = &server.data;

    // Line 1: status dot + name/address + map
    let status_dot = if is_online {
        Span::styled("● ", Style::default().fg(Color::Green))
    } else {
        Span::styled("● ", Style::default().fg(Color::Red))
    };

    let display_name = server
        .name
        .clone()
        .or_else(|| data.srv_name.clone())
        .unwrap_or_else(|| format!("{}:{}", server.ip, server.port));

    let name_span = Span::styled(
        format!("{:<32}", truncate_str(&display_name, 32)),
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );

    let map_span = if let Some(map) = &data.map_name {
        Span::styled(
            format!("  {}", truncate_str(map, 20)),
            Style::default().fg(Color::Cyan),
        )
    } else {
        Span::raw("")
    };

    let line1 = Line::from(vec![status_dot, name_span, map_span]);

    // Line 2: players + latency label
    let players = format!("{}/{}", data.users_cur, data.users_max);
    let player_span = Span::styled(
        format!("{:>5} players", players),
        Style::default().fg(Color::Gray),
    );

    let addr_span = Span::styled(
        format!("  {}:{}", server.ip, server.port),
        Style::default().fg(Color::DarkGray),
    );

    let line2 = Line::from(vec![Span::raw("  "), player_span, addr_span]);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);

    frame.render_widget(Paragraph::new(line1), rows[0]);
    frame.render_widget(Paragraph::new(line2), rows[1]);
}
