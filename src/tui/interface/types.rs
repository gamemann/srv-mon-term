use anyhow::Result;
use ratatui::{Frame, layout::Rect};

use crate::{
    context::Context,
    tui::interface::{
        context::TuiInterfaceContext,
        dashboard::TuiInterfaceDashboard,
        ext::TuiInterfaceExt,
        logs::TuiInterfaceLogs,
        server::{settings::TuiInterfaceServerSettings, view::TuiInterfaceServerView},
        settings::TuiInterfaceSettings,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

    fn get_type(&self) -> TuiInterfaceType {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.get_type(),
            TuiInterface::Logs(ctx) => ctx.get_type(),
            TuiInterface::Settings(ctx) => ctx.get_type(),

            TuiInterface::ServerView(ctx) => ctx.get_type(),
            TuiInterface::ServerSettings(ctx) => ctx.get_type(),
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

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.get_key_bindings(),
            TuiInterface::Logs(ctx) => ctx.get_key_bindings(),
            TuiInterface::Settings(ctx) => ctx.get_key_bindings(),

            TuiInterface::ServerView(ctx) => ctx.get_key_bindings(),
            TuiInterface::ServerSettings(ctx) => ctx.get_key_bindings(),
        }
    }

    async fn handle_input(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
        ctx: Context,
    ) -> Result<()> {
        match self {
            TuiInterface::Dashboard(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::Logs(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::Settings(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::ServerView(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::ServerSettings(ictx) => ictx.handle_input(key, ctx).await,
        }
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {
        match self {
            TuiInterface::Dashboard(ictx) => ictx.draw(frame, area, ctx),
            TuiInterface::Logs(ictx) => ictx.draw(frame, area, ctx),
            TuiInterface::Settings(ictx) => ictx.draw(frame, area, ctx),
            TuiInterface::ServerView(ictx) => ictx.draw(frame, area, ctx),
            TuiInterface::ServerSettings(ictx) => ictx.draw(frame, area, ctx),
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
