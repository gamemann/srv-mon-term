use anyhow::{Result, anyhow, bail};
use ratatui::crossterm::{
    self, execute,
    terminal::{self, EnterAlternateScreen},
};

use crate::{context::Context, tui::types::Tui};

use ratatui::backend::Backend;

impl Tui {
    pub async fn prepare(&self, ctx: Context) -> Result<()> {
        let mut stdout = std::io::stdout();

        // We need to enable raw mode.
        terminal::enable_raw_mode().map_err(|e| anyhow!("Failed to enable raw mode: {}", e))?;

        // Go into the alternate screen and hide the cursor.
        execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)
            .map_err(|e| anyhow!("Failed to enter alternate screen: {}", e))?;

        // Retrieve terminal.
        let mut term = self.term.lock().await;

        // Finally clear the current terminal.
        let term = term.backend_mut();

        term.clear()
            .map_err(|e| anyhow!("Failed to clear terminal: {}", e))?;

        // Setup input handling.
        match Self::setup_input(ctx.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => bail!("Failed to setup input handling: {}", e),
        }
    }
}
