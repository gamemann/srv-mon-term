use crate::tui::interface::types::{TuiInterface, TuiInterfaceType};

pub struct TuiState {
    pub interface: TuiInterface,
}

impl Default for TuiState {
    fn default() -> Self {
        TuiState {
            interface: TuiInterface::new_interface(TuiInterfaceType::Dashboard),
        }
    }
}
