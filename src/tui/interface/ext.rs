use anyhow::Result;
use ratatui::{Frame, crossterm::event::KeyEvent, layout::Rect};

use crate::{context::Context, tui::interface::types::TuiInterfaceType};

#[allow(async_fn_in_trait)]
pub trait TuiInterfaceExt {
    fn title(&self) -> String;
    fn is_top_level(&self) -> bool;

    fn get_type(&self) -> TuiInterfaceType;
    fn parent(&self) -> Option<TuiInterfaceType>;

    fn get_key_bindings(&self) -> Vec<(&str, &str)>;

    fn draw<'a>(&self, frame: &mut Frame<'a>, area: Rect, ctx: Context);

    async fn handle_input(&mut self, ev: KeyEvent, ctx: Context) -> Result<()>;
}
