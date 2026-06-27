pub mod state;

use std::io::Stdout;

use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::RwLock;

use crate::{
    context::ContextWeak,
    tui::{interface::types::TuiInterface, types::state::TuiState},
};

pub struct Tui {
    pub ctx: Option<ContextWeak>,
    pub state: RwLock<TuiState>,
    pub interface: RwLock<TuiInterface>,

    pub term: Option<Terminal<CrosstermBackend<Stdout>>>,

    pub draw_task_id: Option<u128>,
}
