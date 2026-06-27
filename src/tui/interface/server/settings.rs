use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::interface::{
    context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType,
};

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerSettings {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerSettings> {
    fn title(&self) -> String {
        "Server Settings".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::ServerView)
    }

    async fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
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

    async fn draw(&mut self) -> Result<()> {
        // Implement the logic to draw the server settings interface
        println!("Drawing server settings interface...");
        Ok(())
    }
}
