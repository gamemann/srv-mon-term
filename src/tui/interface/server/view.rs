pub mod general;
pub mod latency;
pub mod users;
pub mod vars;

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
pub struct TuiInterfaceServerView {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerView> {
    fn title(&self) -> String {
        "Server View".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::ServerView
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::Dashboard)
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("ESC", "Back")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                // Handle quitting the server view interface
                println!("Quitting server view interface...");
                // Implement logic to switch to another interface or exit
            }
            _ => {
                // Handle other key events specific to the server view interface
                println!("Unhandled key event in server view interface: {:?}", key);
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {
        // Implement the logic to draw the server view interface
    }
}
