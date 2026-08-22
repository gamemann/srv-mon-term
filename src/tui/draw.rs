use anyhow::{Result, anyhow};
use ratatui::layout::{Constraint, Direction, Layout};

use crate::context::Context;
use crate::tui::interface::ext::TuiInterfaceExt;
use crate::tui::types::Tui;

impl Tui {
    pub async fn draw(&self, ctx: Context) -> Result<()> {
        {
            // The lock is held across the fetch and the render so the interface can't change
            // underneath us, which would leave us drawing another interface's data.
            let mut state = self.state.write().await;

            let draw_data = state.interface.fetch_snapshot_data(ctx.clone()).await?;

            let int_type = state.interface.get_type();
            let key_mappings = state.interface.get_key_bindings();

            // Retrieve terminal. Nothing to do when the TUI was never prepared (basic mode).
            let mut term = self.term.lock().await;

            let term = match term.as_mut() {
                Some(term) => term,
                None => return Ok(()),
            };

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
