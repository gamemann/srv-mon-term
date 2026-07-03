use std::collections::VecDeque;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::server::types::latency::ServerLatency;

// How many latency history points to show in the mini sparkline.
const SPARK_WIDTH: usize = 30;

pub fn draw_server_latency(
    frame: &mut Frame<'_>,
    area: Rect,
    latency_history: &VecDeque<ServerLatency>,
) {
    if area.width < 4 || area.height < 1 {
        return;
    }

    // Take last SPARK_WIDTH samples
    let samples: Vec<_> = latency_history
        .iter()
        .rev()
        .take(SPARK_WIDTH)
        .rev()
        .collect();

    if samples.is_empty() {
        let placeholder = Paragraph::new(Span::styled(
            "no data",
            Style::default().fg(Color::DarkGray),
        ));

        frame.render_widget(placeholder, area);
        return;
    }

    let max_val = samples
        .iter()
        .filter(|s| s.online)
        .map(|s| s.val)
        .max()
        .unwrap_or(1)
        .max(1); // avoid div by zero

    draw_sparkline_paragraph(frame, area, &samples, max_val);
}

/// Renders a Unicode block-character sparkline with red segments for offline.
fn draw_sparkline_paragraph(
    frame: &mut Frame<'_>,
    area: Rect,
    samples: &[&ServerLatency],
    max_val: u64,
) {
    // 8 block levels: ▁▂▃▄▅▆▇█
    const BLOCKS: &[char] = &['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

    let mut spans: Vec<Span> = Vec::with_capacity(samples.len() + 8);

    // Current latency value on the left
    if let Some(last) = samples.last() {
        let lat = last.val as f64 / 1000.0;
        let lat_col = latency_color(lat);

        let val_str = if last.online {
            format!(" {:.2}ms", lat)
        } else {
            " offline".to_string()
        };

        let color = if last.online { lat_col } else { Color::Red };

        spans.push(Span::styled(val_str, Style::default().fg(color)));
    }

    spans.push(Span::raw(" "));

    for sample in samples {
        if !sample.online {
            // Offline: full red column character
            spans.push(Span::styled("█", Style::default().fg(Color::Red)));
        } else {
            let ratio = (sample.val as f64 / max_val as f64).clamp(0.0, 1.0);
            let idx = (ratio * (BLOCKS.len() - 1) as f64).round() as usize;
            let ch = BLOCKS[idx];

            let color = latency_color(sample.val as f64 / 1000.0);
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
        }
    }

    let line = Line::from(spans);

    // Vertically center in the available area
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(1) / 2),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .split(area);

    frame.render_widget(Paragraph::new(line), rows[1]);
}

fn latency_color(ms: f64) -> Color {
    if ms < 80.0 {
        Color::Green
    } else if ms < 150.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}
