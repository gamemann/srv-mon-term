use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders},
};

use crate::{
    server::{Server, data::ServerStatus, types::latency::ServerLatency},
    tui::interface::ifaces::dashboard::server::{
        info::draw_server_info, latency::draw_server_latency,
    },
};

pub const ROW_HEIGHT: u16 = 4;

pub fn draw_server_row(
    frame: &mut Frame<'_>,
    area: Rect,
    server: &Server,
    status: ServerStatus,
    latency_history: &VecDeque<ServerLatency>,
    is_selected: bool,
) {
    // Border color: yellow when selected, green/red based on status otherwise.
    let border_color = if is_selected {
        Color::Yellow
    } else if status == ServerStatus::Online {
        Color::Green
    } else {
        Color::Red
    };

    let border_style = Style::default().fg(border_color);
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    if is_selected {
        block = block.border_type(ratatui::widgets::BorderType::Thick);
    }

    // Inner area for content
    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Split inner into left info column and right sparkline column
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(40),    // server info
            Constraint::Length(34), // sparkline (30 bars + some padding)
        ])
        .split(inner);

    draw_server_info(frame, cols[0], server, status == ServerStatus::Online);
    draw_server_latency(frame, cols[1], latency_history);
}
