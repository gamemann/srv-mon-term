use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::server::{Server, data::ServerStatus};

pub fn draw_server_general(frame: &mut Frame<'_>, area: Rect, server: &Server) {
    let data = &server.data;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " General ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let display_name = server
        .display_name
        .clone()
        .or_else(|| data.srv_name.clone())
        .unwrap_or_else(|| format!("{}:{}", server.ip, server.port));

    let (status_text, status_color) = if data.status == ServerStatus::Online {
        ("Online", Color::Green)
    } else if data.status == ServerStatus::Error {
        ("Error", Color::Yellow)
    } else {
        ("Offline", Color::Red)
    };

    let mut lines: Vec<Line> = Vec::new();

    // Name header.
    lines.push(Line::from(vec![
        Span::styled("● ", Style::default().fg(status_color)),
        Span::styled(
            display_name,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    lines.push(Line::from(""));

    let (ip, port) = (server.ip.clone(), server.port);
    let addr = format!("{}:{}", ip, port);

    lines.push(kv_line("Status", status_text, status_color));
    lines.push(kv_line("Address", &addr, Color::Gray));

    if let Some(game) = &data.game_name {
        lines.push(kv_line("Game", game, Color::Gray));
    }

    if let Some(game_dir) = &data.game_dir {
        lines.push(kv_line("Game Dir", game_dir, Color::DarkGray));
    }

    if let Some(map) = &data.map_name {
        lines.push(kv_line("Map", map, Color::Cyan));
    }

    let players = match data.bots_cur {
        Some(bots) if bots > 0 => format!("{}/{} ({} bots)", data.users_cur, data.users_max, bots),
        _ => format!("{}/{}", data.users_cur, data.users_max),
    };
    lines.push(kv_line("Players", &players, Color::White));

    let os = match &data.os {
        Some(os) => os.to_string(),
        None => "Unknown".to_string(),
    };

    lines.push(kv_line("OS", &os, Color::Gray));

    if let Some(version) = &data.version {
        lines.push(kv_line("Version", version, Color::Gray));
    }

    let mut flags = Vec::new();
    let flags_str = flags.join(", ");

    if data.is_secure {
        flags.push("Secure");
    }
    if data.is_dedicated {
        flags.push("Dedicated");
    }
    if data.is_public {
        flags.push("Public");
    }
    if !flags.is_empty() {
        lines.push(kv_line("Flags", &flags_str, Color::Gray));
    }

    let updated_str = if let Some(last_updated) = data.last_updated {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        let secs_ago = now.saturating_sub(last_updated) / 1000;

        if secs_ago < 60 {
            format!("{}s", secs_ago)
        } else if secs_ago < 3600 {
            format!("{}m", secs_ago / 60)
        } else if secs_ago < 86400 {
            format!("{}h", secs_ago / 3600)
        } else {
            format!("{}d", secs_ago / 86400)
        }
    } else {
        "Unknown".to_string()
    };

    lines.push(kv_line("Updated", &updated_str, Color::DarkGray));

    frame.render_widget(Paragraph::new(lines), inner);
}

fn kv_line<'a>(label: &'a str, value: &'a str, value_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{:<10}", label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(value.to_string(), Style::default().fg(value_color)),
    ])
}
