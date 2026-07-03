pub mod state;

use std::io::Stdout;

use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::tui::types::state::TuiState;

pub struct Tui {
    pub state: RwLock<TuiState>,

    pub term: Mutex<Terminal<CrosstermBackend<Stdout>>>,

    pub draw_cancel_token: CancellationToken,
}
