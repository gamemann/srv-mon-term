use anyhow::{Result, bail};

use crate::tui::{
    interface::{
        new::TuiInterfaceOpts,
        types::{TuiInterface, TuiInterfaceType},
    },
    types::Tui,
};

impl Tui {
    pub async fn change_interface(
        &self,
        interface_type: TuiInterfaceType,
        opts: Option<TuiInterfaceOpts>,
    ) -> Result<()> {
        match TuiInterface::new_interface(interface_type, opts) {
            Ok(interface) => {
                let mut state = self.state.write().await;

                // Update internal state.
                state.interface = interface;

                Ok(())
            }
            Err(e) => bail!("Failed to create new interface: {}", e),
        }
    }
}
