use ratatui::Terminal;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::tui::types::{Tui, state::TuiState};

impl Tui {
    pub fn new() -> Self {
        Tui {
            term: Mutex::new(
                Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))
                    .expect("Failed to create terminal"),
            ),
            state: RwLock::new(TuiState::default()),

            draw_cancel_token: CancellationToken::new(),
        }
    }
}
