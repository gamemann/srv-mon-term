use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::tui::types::{Tui, state::TuiState};

impl Default for Tui {
    fn default() -> Self {
        Self::new()
    }
}

impl Tui {
    pub fn new() -> Self {
        Tui {
            term: Mutex::new(None),
            state: RwLock::new(TuiState::default()),

            draw_cancel_token: CancellationToken::new(),
        }
    }
}
