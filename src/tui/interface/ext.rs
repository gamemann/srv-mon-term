use anyhow::Result;
use ratatui::crossterm::event::KeyEvent;

use crate::tui::interface::types::TuiInterfaceType;

#[allow(async_fn_in_trait)]
pub trait TuiInterfaceExt {
    fn title(&self) -> String;
    fn is_top_level(&self) -> bool;
    fn parent(&self) -> Option<TuiInterfaceType>;

    async fn draw(&mut self) -> Result<()>;

    async fn handle_input(&mut self, ev: KeyEvent) -> Result<()>;
}
