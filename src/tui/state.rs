use anyhow::{Result, bail};

use crate::{
    context::Context,
    log_debug,
    logger::level::LogLevel,
    tui::{
        interface::{base::TuiInterface, new::TuiInterfaceOpts, types::TuiInterfaceType},
        types::Tui,
    },
};

use crate::tui::interface::ext::TuiInterfaceExt;

impl Tui {
    pub async fn change_interface(
        &self,
        interface_type: TuiInterfaceType,
        opts: Option<TuiInterfaceOpts>,
        ctx: Context,
    ) -> Result<()> {
        match TuiInterface::new_interface(interface_type, opts) {
            Ok(interface) => {
                let mut state = self.state.write().await;

                // First, cleanup the current interface.
                match state.interface.cleanup(ctx.clone()).await {
                    Ok(_) => {
                        log_debug!(
                            ctx.logger.write().await,
                            "Successfully cleaned up current interface"
                        );
                    }
                    Err(e) => bail!("Failed to cleanup current interface: {}", e),
                }

                // Update internal state.
                state.interface = interface;

                // Then, prepare the new interface.
                match state.interface.prepare(ctx.clone()).await {
                    Ok(_) => {
                        log_debug!(
                            ctx.logger.write().await,
                            "Successfully prepared new interface"
                        );
                    }
                    Err(e) => bail!("Failed to prepare new interface: {}", e),
                }

                Ok(())
            }
            Err(e) => bail!("Failed to create new interface: {}", e),
        }
    }
}
