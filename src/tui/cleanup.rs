use ratatui::crossterm;

use crate::tui::types::Tui;

impl Tui {
    pub async fn cleanup(&mut self) {
        if let Some(term) = self.term.take() {
            let mut stdout = std::io::stdout();

            // Show the cursor and leave the alternate screen.
            let _ = crossterm::execute!(
                stdout,
                crossterm::cursor::Show,
                crossterm::terminal::LeaveAlternateScreen
            );

            // Disable raw mode.
            let _ = crossterm::terminal::disable_raw_mode();

            // Drop the terminal to clean up resources.
            drop(term);
        }
    }
}
