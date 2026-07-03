use anyhow::{Result, anyhow};
use ratatui::layout::{Constraint, Direction, Layout};

use crate::context::Context;
use crate::tui::interface::ext::TuiInterfaceExt;
use crate::tui::types::Tui;

impl Tui {
    pub async fn draw(&self, ctx: Context) -> Result<()> {
        {
            // We need to attempt to fetch draw data, type, and key bindings.
            // We clone the state here since we don't intend on editing it and we don't want to hold the lock while drawing.
            let (draw_data, int_type, key_mappings) = {
                let mut state = self.state.write().await;

                let draw_data = state.interface.fetch_snapshot_data(ctx.clone()).await?;

                let int_type = state.interface.get_type().clone();
                let key_mappings = state.interface.get_key_bindings().clone();

                (draw_data, int_type, key_mappings)
            };

            let state = self.state.read().await;

            // Retrieve terminal.
            let mut term = self.term.lock().await;

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

                // First, let's draw the header that includes the top-level key mappings.
                Tui::draw_header(frame, root[0], int_type);

                state
                    .interface
                    .draw(frame, root[1], ctx.clone(), draw_data.as_ref());

                // Draw the footer which is typically the key mappings for the current interface.
                Tui::draw_footer(frame, root[2], &key_mappings);
            })
            .map_err(|e| anyhow!("Failed to draw terminal: {}", e))?;
        }

        Ok(())
    }
}
