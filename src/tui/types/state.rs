use crate::tui::interface::types::TuiInterface;

#[derive(Default)]
pub struct TuiState {
    pub interface: Option<TuiInterface>,
}
