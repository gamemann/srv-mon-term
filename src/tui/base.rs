use std::{io::Stdout, sync::Arc};

use anyhow::{Result, anyhow};
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::sync::RwLock;

use crate::{
    context::Context,
    tui::types::{Tui, state::TuiState},
};

impl Tui {
    pub fn new() -> Self {
        Tui {
            ctx: None,
            term: None,
            draw_task_id: None,

            state: RwLock::new(TuiState::default()),
        }
    }

    pub fn set_ctx(&mut self, ctx: Context) {
        self.ctx = Some(Arc::downgrade(&ctx));
    }

    pub fn ctx(&self) -> Result<Context> {
        self.ctx
            .as_ref()
            .and_then(|ctx| ctx.upgrade())
            .ok_or_else(|| anyhow!("Failed to upgrade context"))
    }

    pub fn terminal(&mut self) -> Result<&mut Terminal<CrosstermBackend<Stdout>>> {
        self.term
            .as_mut()
            .ok_or_else(|| anyhow!("Terminal not initialized"))
    }
}
