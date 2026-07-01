use crate::tui::interface::ifaces::{
    about::TuiInterfaceAboutDrawData,
    dashboard::TuiInterfaceDashboardDrawData,
    logs::TuiInterfaceLogsDrawData,
    server::{
        new::TuiInterfaceServerNewDrawData, settings::TuiInterfaceServerSettingsDrawData,
        view::TuiInterfaceServerViewDrawData,
    },
    settings::TuiInterfaceSettingsDrawData,
};

#[derive(Debug, Clone)]
pub enum TuiInterfaceDrawData {
    Dashboard(TuiInterfaceDashboardDrawData),
    Logs(TuiInterfaceLogsDrawData),
    Settings(TuiInterfaceSettingsDrawData),
    About(TuiInterfaceAboutDrawData),

    ServerView(TuiInterfaceServerViewDrawData),
    ServerNew(TuiInterfaceServerNewDrawData),
    ServerSettings(TuiInterfaceServerSettingsDrawData),
}
