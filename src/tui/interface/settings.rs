use std::io::Stdout;

use anyhow::Result;
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    crossterm::event::{KeyCode, KeyEvent},
    layout::Rect,
};

use crate::{
    context::Context,
    tui::interface::{context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType},
};

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceSettings {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceSettings> {
    fn title(&self) -> String {
        "Settings".to_string()
    }

    fn is_top_level(&self) -> bool {
        true
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::Settings
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        None
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![(
            "ESC",
            "
        Quit",
        )]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<()> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                ctx.cancel_token.cancel();
            }
            _ => {}
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {
        // Implement the logic to draw the settings interface
    }
}
