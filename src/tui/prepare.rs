use anyhow::{Result, anyhow, bail};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        self, execute,
        terminal::{self, EnterAlternateScreen},
    },
};

use crate::tui::types::Tui;

use ratatui::backend::Backend;

impl Tui {
    pub async fn prepare(&mut self) -> Result<()> {
        {
            let stdout = std::io::stdout();

            let backend = CrosstermBackend::new(stdout);

            self.term = Some(
                Terminal::new(backend).map_err(|e| anyhow!("Failed to create terminal: {}", e))?,
            );
        }

        {
            let mut stdout = std::io::stdout();

            // We need to enable raw mode.
            terminal::enable_raw_mode().map_err(|e| anyhow!("Failed to enable raw mode: {}", e))?;

            // Go into the alternate screen and hide the cursor.
            execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)
                .map_err(|e| anyhow!("Failed to enter alternate screen: {}", e))?;

            let term = self.terminal()?;

            // Finally clear the current terminal.
            let term = term.backend_mut();

            term.clear()
                .map_err(|e| anyhow!("Failed to clear terminal: {}", e))?;

            let ctx = self.ctx()?;

            // Setup input handling.
            match Self::setup_input(ctx.clone()).await {
                Ok(_) => Ok(()),
                Err(e) => bail!("Failed to setup input handling: {}", e),
            }
        }
    }
}
