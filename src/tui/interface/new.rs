use anyhow::{Result, anyhow};

use crate::tui::interface::{
    context::TuiInterfaceContext,
    server::{settings::ServerSettingsOpts, view::ServerViewOpts},
    types::{TuiInterface, TuiInterfaceType},
};

#[derive(Debug, Clone)]
pub enum TuiInterfaceOpts {
    ServerView(ServerViewOpts),
    ServerSettings(ServerSettingsOpts),
}

impl TuiInterface {
    pub fn new_interface<T>(interface_type: TuiInterfaceType, opts: Option<T>) -> Result<Self>
    where
        T: Into<TuiInterfaceOpts>,
    {
        match interface_type {
            TuiInterfaceType::Dashboard => Ok(Self::Dashboard(TuiInterfaceContext::new())),
            TuiInterfaceType::Logs => Ok(Self::Logs(TuiInterfaceContext::new())),
            TuiInterfaceType::Settings => Ok(Self::Settings(TuiInterfaceContext::new())),
            TuiInterfaceType::About => Ok(Self::About(TuiInterfaceContext::new())),

            TuiInterfaceType::ServerView => {
                if let Some(opts) = opts {
                    if let TuiInterfaceOpts::ServerView(server_view_opts) = opts.into() {
                        Ok(Self::ServerView(TuiInterfaceContext::new_with_opts(
                            server_view_opts,
                        )))
                    } else {
                        Err(anyhow!(
                            "Expected ServerViewOpts for ServerView interface, got different options"
                        ))
                    }
                } else {
                    Err(anyhow!(
                        "Expected ServerViewOpts for ServerView interface, got None"
                    ))
                }
            }
            TuiInterfaceType::ServerSettings => {
                if let Some(opts) = opts {
                    if let TuiInterfaceOpts::ServerSettings(server_settings_opts) = opts.into() {
                        Ok(Self::ServerSettings(TuiInterfaceContext::new_with_opts(
                            server_settings_opts,
                        )))
                    } else {
                        Err(anyhow!(
                            "Expected ServerSettingsOpts for ServerSettings interface, got different options"
                        ))
                    }
                } else {
                    Err(anyhow!(
                        "Expected ServerSettingsOpts for ServerSettings interface, got None"
                    ))
                }
            }
        }
    }
}
