use anyhow::{Result, anyhow};
use ratatui::layout::{Constraint, Direction, Layout};

use crate::tui::types::Tui;

use crate::tui::interface::ext::TuiInterfaceExt;

impl Tui {
    pub async fn draw(&mut self) -> Result<()> {
        let ctx = self.ctx()?;

        {
            // Retrieve terminal.
            let term = self
                .term
                .as_mut()
                .ok_or_else(|| anyhow!("Terminal not initialized"))?;

            // Retrieve state so we can draw the current interface frame.
            let state = self.state.read().await;

            // Retrieve interface type and key mappings for header and footer layouts.
            let interface_type = state.interface.get_type();
            let key_mappings = state.interface.get_key_bindings();

            // Draw the current interface.
            term.draw(|frame| {
                // Retrieve frame area for header and footer drawing.
                let area = frame.area();

                // We'll need to create a root layout for the header, body, and footer.
                let root = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(1),
                        Constraint::Min(0),
                        Constraint::Length(1),
                    ])
                    .split(area);

                // First, let's draw the header that include the top-level key mappings.
                Tui::draw_header(frame, root[0], interface_type);

                state.interface.draw(frame, root[1], ctx.clone());

                // Draw the footer
                Tui::draw_footer(frame, root[2], &key_mappings);
            })
            .map_err(|e| anyhow!("Failed to draw terminal: {}", e))?;
        }

        Ok(())
    }
}
