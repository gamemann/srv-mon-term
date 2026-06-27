use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

use crate::tui::interface::{
    context::TuiInterfaceContext, ext::TuiInterfaceExt, types::TuiInterfaceType,
};

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceDashboard {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceDashboard> {
    fn title(&self) -> String {
        "Dashboard".to_string()
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
                // Handle quitting the dashboard interface
                println!("Quitting dashboard interface...");
                // Implement logic to switch to another interface or exit
            }
            _ => {
                // Handle other key events specific to the dashboard
                println!("Unhandled key event in dashboard: {:?}", key);
            }
        }

        Ok(())
    }

    async fn draw(&mut self) -> Result<()> {
        // Implement the logic to draw the dashboard interface
        println!("Drawing dashboard interface...");
        Ok(())
    }
}
