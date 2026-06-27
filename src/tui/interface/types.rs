use crate::tui::interface::{
    context::TuiInterfaceContext,
    dashboard::TuiInterfaceDashboard,
    ext::TuiInterfaceExt,
    logs::TuiInterfaceLogs,
    server::{settings::TuiInterfaceServerSettings, view::TuiInterfaceServerView},
    settings::TuiInterfaceSettings,
};

pub enum TuiInterfaceType {
    Dashboard,
    Logs,
    Settings,

    ServerView,
    ServerSettings,
}

pub enum TuiInterface {
    Dashboard(TuiInterfaceContext<TuiInterfaceDashboard>),
    Logs(TuiInterfaceContext<TuiInterfaceLogs>),
    Settings(TuiInterfaceContext<TuiInterfaceSettings>),

    ServerView(TuiInterfaceContext<TuiInterfaceServerView>),
    ServerSettings(TuiInterfaceContext<TuiInterfaceServerSettings>),
}

impl TuiInterfaceExt for TuiInterface {
    fn title(&self) -> String {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.title(),
            TuiInterface::Logs(ctx) => ctx.title(),
            TuiInterface::Settings(ctx) => ctx.title(),

            TuiInterface::ServerView(ctx) => ctx.title(),
            TuiInterface::ServerSettings(ctx) => ctx.title(),
        }
    }

    fn is_top_level(&self) -> bool {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.is_top_level(),
            TuiInterface::Logs(ctx) => ctx.is_top_level(),
            TuiInterface::Settings(ctx) => ctx.is_top_level(),

            TuiInterface::ServerView(ctx) => ctx.is_top_level(),
            TuiInterface::ServerSettings(ctx) => ctx.is_top_level(),
        }
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.parent(),
            TuiInterface::Logs(ctx) => ctx.parent(),
            TuiInterface::Settings(ctx) => ctx.parent(),

            TuiInterface::ServerView(ctx) => ctx.parent(),
            TuiInterface::ServerSettings(ctx) => ctx.parent(),
        }
    }

    async fn handle_input(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
    ) -> anyhow::Result<()> {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.handle_input(key).await,
            TuiInterface::Logs(ctx) => ctx.handle_input(key).await,
            TuiInterface::Settings(ctx) => ctx.handle_input(key).await,
            TuiInterface::ServerView(ctx) => ctx.handle_input(key).await,
            TuiInterface::ServerSettings(ctx) => ctx.handle_input(key).await,
        }
    }

    async fn draw(&mut self) -> anyhow::Result<()> {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.draw().await,
            TuiInterface::Logs(ctx) => ctx.draw().await,
            TuiInterface::Settings(ctx) => ctx.draw().await,
            TuiInterface::ServerView(ctx) => ctx.draw().await,
            TuiInterface::ServerSettings(ctx) => ctx.draw().await,
        }
    }
}

impl TuiInterface {
    pub fn new_interface(interface_type: TuiInterfaceType) -> Self {
        match interface_type {
            TuiInterfaceType::Dashboard => Self::Dashboard(TuiInterfaceContext::new()),
            TuiInterfaceType::Logs => Self::Logs(TuiInterfaceContext::new()),
            TuiInterfaceType::Settings => Self::Settings(TuiInterfaceContext::new()),
            TuiInterfaceType::ServerView => Self::ServerView(TuiInterfaceContext::new()),
            TuiInterfaceType::ServerSettings => Self::ServerSettings(TuiInterfaceContext::new()),
        }
    }
}
