use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::interface::{
    context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType,
};

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceLogs {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceLogs> {
    fn title(&self) -> String {
        "Logs".to_string()
    }

    fn is_top_level(&self) -> bool {
        true
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        None
    }

    async fn handle_input(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('q') => {
                // Handle quitting the logs interface
                println!("Quitting logs interface...");
                // Implement logic to switch to another interface or exit
            }
            _ => {
                // Handle other key events specific to the logs interface
                println!("Unhandled key event in logs interface: {:?}", key);
            }
        }

        Ok(())
    }

    async fn draw(&mut self) -> Result<()> {
        // Implement the logic to draw the logs interface
        println!("Drawing logs interface...");
        Ok(())
    }
}
