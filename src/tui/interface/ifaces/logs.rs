use std::{
    collections::VecDeque,
    sync::atomic::{AtomicUsize, Ordering},
};

use anyhow::Result;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    context::Context,
    logger::{Logger, buffer::LogBufferData, level::LogLevel},
    tui::{
        action::TuiAction,
        interface::{context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType},
    },
};

#[derive(Debug)]
pub struct TuiInterfaceLogs {
    pub scroll_offset: usize,

    content_len: AtomicUsize,
    viewport_height: AtomicUsize,

    previous_buffer: VecDeque<LogBufferData>,
}

impl Clone for TuiInterfaceLogs {
    fn clone(&self) -> Self {
        Self {
            scroll_offset: self.scroll_offset,
            content_len: AtomicUsize::new(self.content_len.load(Ordering::Relaxed)),
            viewport_height: AtomicUsize::new(self.viewport_height.load(Ordering::Relaxed)),
            previous_buffer: self.previous_buffer.clone(),
        }
    }
}

impl Default for TuiInterfaceLogs {
    fn default() -> Self {
        Self {
            scroll_offset: 0,
            content_len: AtomicUsize::new(0),
            viewport_height: AtomicUsize::new(0),
            previous_buffer: VecDeque::new(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TuiInterfaceLogsDrawData {
    pub buffer_snapshot: VecDeque<LogBufferData>,
}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceLogs> {
    type DrawData = TuiInterfaceLogsDrawData;

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

    async fn prepare(&mut self, _ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, _ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(String, String)> {
        vec![
            ("Esc".to_string(), "Quit".to_string()),
            ("↑↓".to_string(), "Scroll".to_string()),
            ("End".to_string(), "Jump to latest".to_string()),
        ]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        let max_scroll = self
            .interface
            .content_len
            .load(Ordering::Relaxed)
            .saturating_sub(self.interface.viewport_height.load(Ordering::Relaxed));

        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                ctx.cancel_token.cancel();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.interface.scroll_offset = (self.interface.scroll_offset + 1).min(max_scroll);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.interface.scroll_offset = self.interface.scroll_offset.saturating_sub(1);
            }
            KeyCode::End => {
                self.interface.scroll_offset = 0;
            }
            _ => {}
        }

        Ok(TuiAction::None)
    }

    fn draw(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        _ctx: Context,
        draw_data: Option<&Self::DrawData>,
    ) {
        // Try to read the internal state's snapshot.
        let buffer = match draw_data {
            Some(data) => &data.buffer_snapshot,
            None => return, // We don't have data yet, return;
        };

        let lines: Vec<Line> = buffer.iter().map(|en| parse_log_data(en.clone())).collect();
        let lines_tot = lines.len();

        // Create temp block to get inner height (without borders)
        let block = Block::default().borders(Borders::ALL);
        let inner = block.inner(area);

        let inner_height = inner.height as usize;

        // Cache the content length and viewport height for scroll calculations.
        self.interface
            .content_len
            .store(lines_tot, Ordering::Relaxed);
        self.interface
            .viewport_height
            .store(inner_height, Ordering::Relaxed);

        // Clamp the scroll.
        let max_scroll = lines_tot.saturating_sub(inner_height);
        let scroll_offset = self.interface.scroll_offset.min(max_scroll);
        let scroll_from_top = lines_tot.saturating_sub(inner_height + scroll_offset);

        // Draw the scroll indicator.
        let scroll_indicator = if scroll_offset > 0 {
            format!("↑ {} lines above", scroll_offset)
        } else {
            String::new()
        };

        let block = block.title(Span::styled(
            scroll_indicator,
            Style::default().fg(Color::Yellow),
        ));

        frame.render_widget(block, area);

        // Draw logs.
        let para = Paragraph::new(lines).scroll((scroll_from_top as u16, 0));

        frame.render_widget(para, inner);
    }

    async fn fetch_snapshot_data(&mut self, ctx: Context) -> Result<Option<Self::DrawData>> {
        if self.interface.scroll_offset > 0 {
            return Ok(Some(TuiInterfaceLogsDrawData {
                buffer_snapshot: self.interface.previous_buffer.clone(),
            }));
        }

        let buffer = match ctx.logger.buffer.try_read() {
            Ok(buffer) => buffer.clone(),
            Err(_) => {
                return Ok(None);
            }
        };

        // Save previous buffer.
        self.interface.previous_buffer = buffer.clone();

        Ok(Some(TuiInterfaceLogsDrawData {
            buffer_snapshot: buffer,
        }))
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
