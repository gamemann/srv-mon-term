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
pub struct TuiInterfaceServerSettings {
    pub server_id: String,
}

impl TuiInterfaceServerSettings {
    pub fn new(server_id: String) -> Self {
        Self { server_id }
    }
}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerSettings> {
    fn title(&self) -> String {
        "Server Settings".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::ServerSettings
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::ServerView)
    }

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![("ESC", "Back")]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                // Handle quitting the server settings interface
                println!("Quitting server settings interface...");
                // Implement logic to switch to another interface or exit
            }
            _ => {
                // Handle other key events specific to the server settings interface
                println!(
                    "Unhandled key event in server settings interface: {:?}",
                    key
                );
            }
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {}
}
