use anyhow::Result;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    context::Context,
    tui::{
        action::TuiAction,
        interface::{context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType},
    },
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceAbout {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceAbout> {
    fn title(&self) -> String {
        "About".to_string()
    }

    fn is_top_level(&self) -> bool {
        true
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::About
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        None
    }

    async fn prepare(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("Esc", "Quit")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => Ok(TuiAction::Exit),
            _ => Ok(TuiAction::None),
        }
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, _ctx: Context) {
        let lines = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    "srv-mon-term",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Author: ", Style::default().fg(Color::DarkGray)),
                Span::styled("Christian Deacon (", Style::default().fg(Color::White)),
                Span::styled("@gamemann", Style::default().fg(Color::Cyan)),
                Span::styled(")", Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Version: ", Style::default().fg(Color::DarkGray)),
                Span::styled(VERSION, Style::default().fg(Color::White)),
            ]),
            Line::raw(""),
            Line::from(Span::styled(
                "This is an open source tool that allows you to monitor servers through the terminal.",
                Style::default().fg(Color::Gray),
            )),
        ];

        frame.render_widget(Paragraph::new(lines), area);
    }
}
