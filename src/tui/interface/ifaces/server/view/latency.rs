use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::server::{
    latency::get_latency_color,
    types::latency::{ServerLatency, ServerLatencyType},
};

// 9 fill levels per row cell: empty + 8 partial/full block heights.
const BLOCKS: &[char] = &[' ', '▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];
const SUB_LEVELS: usize = BLOCKS.len() - 1;

/// A more in-depth latency view than the dashboard's mini sparkline: a
/// multi-row block graph (more vertical resolution) plus min/max/avg/loss
/// stats and the latency check method in use.
pub fn draw_server_latency_detail(
    frame: &mut Frame<'_>,
    area: Rect,
    latency_type: &ServerLatencyType,
    latency_history: &VecDeque<ServerLatency>,
) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray))
        .title(Span::styled(
            " Latency ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 4 || inner.width < 12 {
        return;
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(3),    // graph
            Constraint::Length(1), // stats line
            Constraint::Length(1), // meta line (method + loss)
        ])
        .split(inner);

    let graph_area = rows[0];
    let stats_area = rows[1];
    let meta_area = rows[2];

    let axis_width = 6u16;
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(axis_width), Constraint::Min(0)])
        .split(graph_area);

    let axis_area = cols[0];
    let bars_area = cols[1];

    let width = bars_area.width as usize;
    let height = bars_area.height as usize;

    if width == 0 || height == 0 {
        draw_stats_and_meta(frame, stats_area, meta_area, latency_type, latency_history);
        return;
    }

    let samples: Vec<&ServerLatency> = {
        let mut s: Vec<&ServerLatency> = latency_history.iter().rev().take(width).collect();
        s.reverse();
        s
    };

    if samples.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No latency data yet",
                Style::default().fg(Color::DarkGray),
            )),
            bars_area,
        );
        draw_stats_and_meta(frame, stats_area, meta_area, latency_type, latency_history);
        return;
    }

    let max_val = samples
        .iter()
        .filter(|s| s.online)
        .map(|s| s.val)
        .max()
        .unwrap_or(1)
        .max(1);

    let total_units = (height * SUB_LEVELS) as f64;

    // grid[row][col] = (char, color); row 0 is the top of the graph.
    let mut grid: Vec<Vec<(char, Color)>> = vec![vec![(' ', Color::Reset); samples.len()]; height];

    for (x, sample) in samples.iter().enumerate() {
        if !sample.online {
            for row in grid.iter_mut() {
                row[x] = ('█', Color::Red);
            }
            continue;
        }

        let lat_real = sample.val as f64 / 1000.0;
        let lat_max = max_val as f64 / 1000.0;

        let ratio = (lat_real / lat_max).clamp(0.0, 1.0);
        let units = (ratio * total_units).round() as usize;
        let color = get_latency_color(lat_real);

        for row_from_bottom in 0..height {
            let row_units_start = row_from_bottom * SUB_LEVELS;

            if units <= row_units_start {
                continue;
            }

            let filled_in_row = (units - row_units_start).min(SUB_LEVELS);
            let ch = BLOCKS[filled_in_row];
            let y = height - 1 - row_from_bottom;

            grid[y][x] = (ch, color);
        }
    }

    for (y, row) in grid.iter().enumerate() {
        let spans: Vec<Span> = row
            .iter()
            .map(|(ch, color)| Span::styled(ch.to_string(), Style::default().fg(*color)))
            .collect();

        let row_area = Rect {
            x: bars_area.x,
            y: bars_area.y + y as u16,
            width: bars_area.width,
            height: 1,
        };

        frame.render_widget(Paragraph::new(Line::from(spans)), row_area);
    }

    // Y-axis labels: max at top, 0 at bottom.
    let top_area = Rect {
        x: axis_area.x,
        y: axis_area.y,
        width: axis_area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("{:>4.2} ", max_val as f64 / 1000.0),
            Style::default().fg(Color::DarkGray),
        )),
        top_area,
    );

    let bottom_area = Rect {
        x: axis_area.x,
        y: axis_area.y + height as u16 - 1,
        width: axis_area.width,
        height: 1,
    };
    frame.render_widget(
        Paragraph::new(Span::styled(
            format!("{:>4} ", 0),
            Style::default().fg(Color::DarkGray),
        )),
        bottom_area,
    );

    draw_stats_and_meta(frame, stats_area, meta_area, latency_type, latency_history);
}

fn draw_stats_and_meta(
    frame: &mut Frame<'_>,
    stats_area: Rect,
    meta_area: Rect,
    latency_type: &ServerLatencyType,
    latency_history: &VecDeque<ServerLatency>,
) {
    let online_samples: Vec<u64> = latency_history
        .iter()
        .filter(|s| s.online)
        .map(|s| s.val)
        .collect();

    let total = latency_history.len();
    let offline_count = latency_history.iter().filter(|s| !s.online).count();
    let loss_pct = if total > 0 {
        (offline_count as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    let (cur_str, cur_color) = match latency_history.back() {
        Some(s) => {
            let lat_real = s.val as f64 / 1000.0;

            if s.online {
                (format!("{:.2}ms", lat_real), get_latency_color(lat_real))
            } else {
                ("offline".to_string(), Color::Red)
            }
        }
        None => ("--".to_string(), Color::DarkGray),
    };

    let (min, max, avg) = if !online_samples.is_empty() {
        let min = *online_samples.iter().min().unwrap() as f64 / 1000.0;
        let max = *online_samples.iter().max().unwrap() as f64 / 1000.0;
        let avg = online_samples.iter().sum::<u64>() as f64 / online_samples.len() as f64 / 1000.0;
        (min, max, avg)
    } else {
        (0.0, 0.0, 0.0)
    };

    let stats_line = Line::from(vec![
        Span::styled("Current: ", Style::default().fg(Color::DarkGray)),
        Span::styled(cur_str, Style::default().fg(cur_color)),
        Span::styled("  Min: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:.2}ms", min), Style::default().fg(Color::Gray)),
        Span::styled("  Max: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:.2}ms", max), Style::default().fg(Color::Gray)),
        Span::styled("  Avg: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:.2}ms", avg), Style::default().fg(Color::Gray)),
    ]);

    frame.render_widget(Paragraph::new(stats_line), stats_area);

    let loss_color = if loss_pct > 10.0 {
        Color::Red
    } else if loss_pct > 0.0 {
        Color::Yellow
    } else {
        Color::Green
    };

    let meta_line = Line::from(vec![
        Span::styled("Method: ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            latency_type_label(latency_type),
            Style::default().fg(Color::Cyan),
        ),
        Span::styled("  Loss: ", Style::default().fg(Color::DarkGray)),
        Span::styled(format!("{:.2}%", loss_pct), Style::default().fg(loss_color)),
        Span::styled(
            format!("  ({} samples)", total),
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    frame.render_widget(Paragraph::new(meta_line), meta_area);
}

fn latency_type_label(t: &ServerLatencyType) -> &'static str {
    match t {
        ServerLatencyType::SelfInfo => "Self Info",
        ServerLatencyType::SelfUsers => "Self Users",
        ServerLatencyType::SelfVars => "Self Vars",
        ServerLatencyType::A2sInfo => "A2S Info",
        ServerLatencyType::A2sPlayers => "A2S Players",
        ServerLatencyType::A2sRules => "A2S Rules",
        ServerLatencyType::Icmp => "ICMP",
    }
}
