use crate::tui::interface::{new::TuiInterfaceOpts, types::TuiInterfaceType};

pub enum TuiAction {
    None,
    ChangeInterface(TuiInterfaceType, Option<TuiInterfaceOpts>),
    Exit,
}
