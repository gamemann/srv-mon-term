use anyhow::{Result, anyhow, bail};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::crossterm::{
    self, execute,
    terminal::{self, EnterAlternateScreen},
};

use crate::{context::Context, tui::types::Tui};

use ratatui::backend::Backend;

impl Tui {
    pub async fn prepare(&self, ctx: Context) -> Result<()> {
        let mut stdout = std::io::stdout();

        // Create the terminal here so basic mode never needs a TTY.
        let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))
            .map_err(|e| anyhow!("Failed to create terminal: {}", e))?;

        // We need to enable raw mode.
        terminal::enable_raw_mode().map_err(|e| anyhow!("Failed to enable raw mode: {}", e))?;

        // Go into the alternate screen and hide the cursor.
        execute!(stdout, EnterAlternateScreen, crossterm::cursor::Hide)
            .map_err(|e| anyhow!("Failed to enter alternate screen: {}", e))?;

        // Store the terminal so the draw loop can use it.
        let mut term = self.term.lock().await;

        *term = Some(terminal);

        // Finally clear the current terminal.
        term.as_mut()
            .expect("terminal was just stored")
            .backend_mut()
            .clear()
            .map_err(|e| anyhow!("Failed to clear terminal: {}", e))?;

        // Setup input handling.
        match Self::setup_input(ctx.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => bail!("Failed to setup input handling: {}", e),
        }
    }
}
