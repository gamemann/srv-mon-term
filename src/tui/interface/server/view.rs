pub mod general;
pub mod latency;
pub mod users;
pub mod vars;

use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::interface::{
    context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType,
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

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::Dashboard)
    }

    async fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
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

    async fn draw(&mut self) -> Result<()> {
        // Implement the logic to draw the server view interface
        println!("Drawing server view interface...");
        Ok(())
    }
}
