pub mod state;

use std::io::Stdout;

use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::{context::ContextWeak, tui::types::state::TuiState};

pub struct Tui {
    pub ctx: Option<ContextWeak>,
    pub state: RwLock<TuiState>,

    pub term: Option<Terminal<CrosstermBackend<Stdout>>>,

    pub draw_cancel_token: CancellationToken,
}
