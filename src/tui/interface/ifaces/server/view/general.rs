use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::server::{Server, data::ServerStatus};

pub fn draw_server_general(
    frame: &mut Frame<'_>,
    area: Rect,
    server: &Server,
    status: ServerStatus,
) {
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
        .unwrap_or_else(|| server.to_addr());

    let (status_text, status_color) = match status {
        ServerStatus::Online => ("Online".to_string(), Color::Green),
        ServerStatus::Error(code) => (format!("Error ({})", code), Color::Yellow),
        ServerStatus::Offline => ("Offline".to_string(), Color::Red),
        ServerStatus::Unknown => ("Unknown".to_string(), Color::DarkGray),
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

    lines.push(kv_line("Status", status_text, status_color));
    lines.push(kv_line("Address", server.to_addr(), Color::Gray));

    // Only worth showing when queries go somewhere other than the game port.
    let query_value = if server.query_port() != server.port {
        format!("{} (port {})", server.query_type, server.query_port())
    } else {
        server.query_type.to_string()
    };

    lines.push(kv_line("Query", query_value, Color::Gray));

    if let Some(game) = &data.game_name {
        lines.push(kv_line("Game", game.clone(), Color::Gray));
    }

    if let Some(game_dir) = &data.game_dir {
        lines.push(kv_line("Game Dir", game_dir.clone(), Color::DarkGray));
    }

    if let Some(map) = &data.map_name {
        lines.push(kv_line("Map", map.clone(), Color::Cyan));
    }

    let players = match data.bots_cur {
        Some(bots) if bots > 0 => format!("{}/{} ({} bots)", data.users_cur, data.users_max, bots),
        _ => format!("{}/{}", data.users_cur, data.users_max),
    };

    lines.push(kv_line("Players", players, Color::White));

    if let Some(os) = &data.os {
        lines.push(kv_line("OS", os.to_string(), Color::Gray));
    }

    if let Some(version) = &data.version {
        lines.push(kv_line("Version", version.clone(), Color::Gray));
    }

    let mut flags = Vec::new();

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
        lines.push(kv_line("Flags", flags.join(", "), Color::Gray));
    }

    lines.push(kv_line(
        "Updated",
        format_last_updated(data.last_updated),
        Color::DarkGray,
    ));

    frame.render_widget(Paragraph::new(lines), inner);
}

/// Formats how long ago the server data was refreshed.
fn format_last_updated(last_updated: Option<u64>) -> String {
    let last_updated = match last_updated {
        Some(ts) => ts,
        None => return "Unknown".to_string(),
    };

    let now = chrono::Utc::now().timestamp_millis() as u64;
    let secs_ago = now.saturating_sub(last_updated) / 1000;

    if secs_ago < 60 {
        format!("{}s ago", secs_ago)
    } else if secs_ago < 3600 {
        format!("{}m ago", secs_ago / 60)
    } else if secs_ago < 86400 {
        format!("{}h ago", secs_ago / 3600)
    } else {
        format!("{}d ago", secs_ago / 86400)
    }
}

fn kv_line<'a>(label: &'a str, value: String, value_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled(
            format!("{:<10}", label),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(value, Style::default().fg(value_color)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_relative_times() {
        assert_eq!(format_last_updated(None), "Unknown");

        let now = chrono::Utc::now().timestamp_millis() as u64;

        assert_eq!(format_last_updated(Some(now)), "0s ago");
        assert_eq!(format_last_updated(Some(now - 120_000)), "2m ago");
        assert_eq!(format_last_updated(Some(now - 7_200_000)), "2h ago");
    }
}
