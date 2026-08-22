pub mod state;

use std::io::Stdout;

use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::tui::types::state::TuiState;

pub struct Tui {
    pub state: RwLock<TuiState>,

    /// Only set once the TUI has been prepared. Basic mode never creates a terminal, which
    /// also means the program keeps working when stdout isn't a TTY.
    pub term: Mutex<Option<Terminal<CrosstermBackend<Stdout>>>>,

    pub draw_cancel_token: CancellationToken,
}
