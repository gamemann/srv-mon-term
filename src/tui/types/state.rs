use crate::tui::interface::{
    new::TuiInterfaceOpts,
    types::{TuiInterface, TuiInterfaceType},
};

pub struct TuiState {
    pub interface: TuiInterface,
}

impl Default for TuiState {
    fn default() -> Self {
        TuiState {
            interface: TuiInterface::new_interface::<TuiInterfaceOpts>(
                TuiInterfaceType::Dashboard,
                None,
            )
            .expect("Failed to create default interface"),
        }
    }
}
