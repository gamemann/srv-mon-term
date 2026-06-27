use anyhow::{Result, anyhow};
use ratatui::{Terminal, backend::CrosstermBackend};

use crate::tui::types::Tui;

impl Tui {
    pub async fn prepare(&mut self) -> Result<()> {
        let stdout = std::io::stdout();

        let backend = CrosstermBackend::new(stdout);

        self.term =
            Some(Terminal::new(backend).map_err(|e| anyhow!("Failed to create terminal: {}", e))?);

        Ok(())
    }
}
