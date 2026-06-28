use anyhow::Result;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::{
    context::Context,
    logger::{buffer::LogBufferData, level::LogLevel},
    tui::interface::{context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType},
};

#[derive(Debug, Clone)]
pub struct TuiInterfaceLogs {
    pub scroll_offset: usize,
}

impl Default for TuiInterfaceLogs {
    fn default() -> Self {
        Self { scroll_offset: 0 }
    }
}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceLogs> {
    fn title(&self) -> String {
        "Logs".to_string()
    }

    fn is_top_level(&self) -> bool {
        true
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::Logs
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        None
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("Esc", "Quit"), ("↑↓", "Scroll"), ("End", "Jump to latest")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                ctx.cancel_token.cancel();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.interface.scroll_offset += 1;
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.interface.scroll_offset = self.interface.scroll_offset.saturating_sub(1);
            }
            KeyCode::End => {
                self.interface.scroll_offset = 0;
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {
        // Attempt to read the log buffer.
        let logger = match ctx.logger.try_read() {
            Ok(logger) => logger,
            Err(_) => {
                return;
            }
        };

        let buffer = match logger.buffer.try_read() {
            Ok(buffer) => buffer,
            Err(_) => {
                return;
            }
        };

        let height = area.height as usize;

        let lines: Vec<Line> = buffer.iter().map(|en| parse_log_data(en.clone())).collect();

        let lines_tot = lines.len();

        // Clamp the scroll.
        let max_scroll = lines_tot.saturating_sub(height);
        let scroll_offset = self.interface.scroll_offset.min(max_scroll);

        // Slice the visible window and make recent log appear at the bottom.
        let visible_start = lines_tot.saturating_sub(height + scroll_offset);
        let visible_end = lines_tot.saturating_sub(scroll_offset);
        let visible_lines = lines[visible_start..visible_end].to_vec();

        // Draw the scroll indicator.
        let scroll_indicator = if scroll_offset > 0 {
            format!("↑ {} lines above", scroll_offset)
        } else {
            String::new()
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                scroll_indicator,
                Style::default().fg(Color::Yellow),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let para = Paragraph::new(visible_lines).wrap(Wrap { trim: false });
        frame.render_widget(para, inner);
    }
}

/// Parse a pre-formatted "[LEVEL] message" string into a colored ratatui Line.
fn parse_log_data(data: LogBufferData) -> Line<'static> {
    let level = data.level;
    let msg = data.message;
    let ts = data.timestamp;

    let level_str = format!("{:?}", level).to_uppercase();
    let ts_str = ts.format("%Y-%m-%d %H:%M:%S").to_string();

    let (level_color, level_style) = level_color(level.clone());

    let level_span = Span::styled(
        format!("[{}]", level_str),
        Style::default().fg(level_color).add_modifier(level_style),
    );
    let ts_span = Span::styled(format!("{}", ts_str), Style::default().fg(Color::DarkGray));

    let msg_span = Span::styled(msg.to_string(), Style::default().fg(msg_color(level)));

    Line::from(vec![
        level_span,
        Span::raw(" "),
        ts_span,
        Span::raw(" "),
        msg_span,
    ])
}

fn level_color(level: LogLevel) -> (Color, Modifier) {
    match level {
        LogLevel::Trace => (Color::DarkGray, Modifier::empty()),
        LogLevel::Debug => (Color::Blue, Modifier::empty()),
        LogLevel::Info => (Color::Cyan, Modifier::empty()),
        LogLevel::Warn => (Color::Yellow, Modifier::BOLD),
        LogLevel::Error => (Color::Red, Modifier::BOLD),
        LogLevel::Fatal => (Color::Magenta, Modifier::BOLD),
    }
}

fn msg_color(level: LogLevel) -> Color {
    match level {
        LogLevel::Trace | LogLevel::Debug => Color::DarkGray,
        LogLevel::Warn | LogLevel::Info => Color::White,
        LogLevel::Error | LogLevel::Fatal => Color::LightRed,
    }
}
