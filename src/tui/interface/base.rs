use anyhow::Result;
use ratatui::{Frame, layout::Rect};

use crate::{
    context::Context,
    tui::{
        action::TuiAction,
        interface::{
            context::TuiInterfaceContext,
            ext::TuiInterfaceExt,
            ifaces::{
                about::TuiInterfaceAbout,
                dashboard::TuiInterfaceDashboard,
                logs::TuiInterfaceLogs,
                server::{
                    new::TuiInterfaceServerNew, settings::TuiInterfaceServerSettings,
                    view::TuiInterfaceServerView,
                },
                settings::TuiInterfaceSettings,
            },
            types::{TuiInterfaceDrawData, TuiInterfaceType},
        },
    },
};

#[derive(Debug, Clone)]
pub enum TuiInterface {
    Dashboard(TuiInterfaceContext<TuiInterfaceDashboard>),
    Logs(TuiInterfaceContext<TuiInterfaceLogs>),
    Settings(TuiInterfaceContext<TuiInterfaceSettings>),
    About(TuiInterfaceContext<TuiInterfaceAbout>),

    ServerView(TuiInterfaceContext<TuiInterfaceServerView>),
    ServerNew(TuiInterfaceContext<TuiInterfaceServerNew>),
    ServerSettings(TuiInterfaceContext<TuiInterfaceServerSettings>),
}

impl TuiInterfaceExt for TuiInterface {
    type DrawData = TuiInterfaceDrawData;

    fn title(&self) -> String {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.title(),
            TuiInterface::Logs(ctx) => ctx.title(),
            TuiInterface::Settings(ctx) => ctx.title(),
            TuiInterface::About(ctx) => ctx.title(),
            TuiInterface::ServerView(ctx) => ctx.title(),
            TuiInterface::ServerNew(ctx) => ctx.title(),
            TuiInterface::ServerSettings(ctx) => ctx.title(),
        }
    }

    fn is_top_level(&self) -> bool {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.is_top_level(),
            TuiInterface::Logs(ctx) => ctx.is_top_level(),
            TuiInterface::Settings(ctx) => ctx.is_top_level(),
            TuiInterface::About(ctx) => ctx.is_top_level(),
            TuiInterface::ServerView(ctx) => ctx.is_top_level(),
            TuiInterface::ServerNew(ctx) => ctx.is_top_level(),
            TuiInterface::ServerSettings(ctx) => ctx.is_top_level(),
        }
    }

    fn get_type(&self) -> TuiInterfaceType {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.get_type(),
            TuiInterface::Logs(ctx) => ctx.get_type(),
            TuiInterface::Settings(ctx) => ctx.get_type(),
            TuiInterface::About(ctx) => ctx.get_type(),
            TuiInterface::ServerView(ctx) => ctx.get_type(),
            TuiInterface::ServerNew(ctx) => ctx.get_type(),
            TuiInterface::ServerSettings(ctx) => ctx.get_type(),
        }
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.parent(),
            TuiInterface::Logs(ctx) => ctx.parent(),
            TuiInterface::Settings(ctx) => ctx.parent(),
            TuiInterface::About(ctx) => ctx.parent(),
            TuiInterface::ServerView(ctx) => ctx.parent(),
            TuiInterface::ServerNew(ctx) => ctx.parent(),
            TuiInterface::ServerSettings(ctx) => ctx.parent(),
        }
    }

    async fn prepare(&mut self, ctx: Context) -> Result<()> {
        match self {
            TuiInterface::Dashboard(ictx) => ictx.prepare(ctx).await,
            TuiInterface::Logs(ictx) => ictx.prepare(ctx).await,
            TuiInterface::Settings(ictx) => ictx.prepare(ctx).await,
            TuiInterface::About(ictx) => ictx.prepare(ctx).await,
            TuiInterface::ServerView(ictx) => ictx.prepare(ctx).await,
            TuiInterface::ServerNew(ictx) => ictx.prepare(ctx).await,
            TuiInterface::ServerSettings(ictx) => ictx.prepare(ctx).await,
        }
    }

    async fn cleanup(&mut self, ctx: Context) -> Result<()> {
        match self {
            TuiInterface::Dashboard(ictx) => ictx.cleanup(ctx).await,
            TuiInterface::Logs(ictx) => ictx.cleanup(ctx).await,
            TuiInterface::Settings(ictx) => ictx.cleanup(ctx).await,
            TuiInterface::About(ictx) => ictx.cleanup(ctx).await,
            TuiInterface::ServerView(ictx) => ictx.cleanup(ctx).await,
            TuiInterface::ServerNew(ictx) => ictx.cleanup(ctx).await,
            TuiInterface::ServerSettings(ictx) => ictx.cleanup(ctx).await,
        }
    }

    fn get_key_bindings(&self) -> Vec<(String, String)> {
        match self {
            TuiInterface::Dashboard(ctx) => ctx.get_key_bindings(),
            TuiInterface::Logs(ctx) => ctx.get_key_bindings(),
            TuiInterface::Settings(ctx) => ctx.get_key_bindings(),
            TuiInterface::About(ctx) => ctx.get_key_bindings(),
            TuiInterface::ServerView(ctx) => ctx.get_key_bindings(),
            TuiInterface::ServerNew(ctx) => ctx.get_key_bindings(),
            TuiInterface::ServerSettings(ctx) => ctx.get_key_bindings(),
        }
    }

    async fn handle_input(
        &mut self,
        key: ratatui::crossterm::event::KeyEvent,
        ctx: Context,
    ) -> Result<TuiAction> {
        match self {
            TuiInterface::Dashboard(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::Logs(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::Settings(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::About(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::ServerView(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::ServerNew(ictx) => ictx.handle_input(key, ctx).await,
            TuiInterface::ServerSettings(ictx) => ictx.handle_input(key, ctx).await,
        }
    }

    fn draw(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        ctx: Context,
        draw_data: Option<&Self::DrawData>,
    ) {
        match self {
            TuiInterface::Dashboard(ictx) => ictx.draw(
                frame,
                area,
                ctx,
                draw_data.and_then(|data| match data {
                    TuiInterfaceDrawData::Dashboard(data) => Some(data),
                    _ => None,
                }),
            ),
            TuiInterface::Logs(ictx) => ictx.draw(
                frame,
                area,
                ctx,
                draw_data.and_then(|data| match data {
                    TuiInterfaceDrawData::Logs(data) => Some(data),
                    _ => None,
                }),
            ),
            TuiInterface::Settings(ictx) => ictx.draw(
                frame,
                area,
                ctx,
                draw_data.and_then(|data| match data {
                    TuiInterfaceDrawData::Settings(data) => Some(data),
                    _ => None,
                }),
            ),
            TuiInterface::About(ictx) => ictx.draw(
                frame,
                area,
                ctx,
                draw_data.and_then(|data| match data {
                    TuiInterfaceDrawData::About(data) => Some(data),
                    _ => None,
                }),
            ),
            TuiInterface::ServerView(ictx) => ictx.draw(
                frame,
                area,
                ctx,
                draw_data.and_then(|data| match data {
                    TuiInterfaceDrawData::ServerView(data) => Some(data),
                    _ => None,
                }),
            ),
            TuiInterface::ServerNew(ictx) => ictx.draw(
                frame,
                area,
                ctx,
                draw_data.and_then(|data| match data {
                    TuiInterfaceDrawData::ServerNew(data) => Some(data),
                    _ => None,
                }),
            ),
            TuiInterface::ServerSettings(ictx) => ictx.draw(
                frame,
                area,
                ctx,
                draw_data.and_then(|data| match data {
                    TuiInterfaceDrawData::ServerSettings(data) => Some(data),
                    _ => None,
                }),
            ),
        }
    }

    async fn fetch_snapshot_data(&mut self, ctx: Context) -> Result<Option<TuiInterfaceDrawData>> {
        match self {
            TuiInterface::Dashboard(ictx) => {
                let data = match ictx.fetch_snapshot_data(ctx).await? {
                    Some(data) => data,
                    None => {
                        return Ok(None);
                    }
                };

                Ok(Some(TuiInterfaceDrawData::Dashboard(data.into())))
            }
            TuiInterface::Logs(ictx) => {
                let data = match ictx.fetch_snapshot_data(ctx).await? {
                    Some(data) => data,
                    None => {
                        return Ok(None);
                    }
                };

                Ok(Some(TuiInterfaceDrawData::Logs(data.into())))
            }
            TuiInterface::Settings(ictx) => {
                let data = match ictx.fetch_snapshot_data(ctx).await? {
                    Some(data) => data,
                    None => {
                        return Ok(None);
                    }
                };

                Ok(Some(TuiInterfaceDrawData::Settings(data.into())))
            }
            TuiInterface::About(ictx) => {
                let data = match ictx.fetch_snapshot_data(ctx).await? {
                    Some(data) => data,
                    None => {
                        return Ok(None);
                    }
                };

                Ok(Some(TuiInterfaceDrawData::About(data.into())))
            }
            TuiInterface::ServerView(ictx) => {
                let data = match ictx.fetch_snapshot_data(ctx).await? {
                    Some(data) => data,
                    None => {
                        return Ok(None);
                    }
                };
                Ok(Some(TuiInterfaceDrawData::ServerView(data.into())))
            }
            TuiInterface::ServerNew(ictx) => {
                let data = match ictx.fetch_snapshot_data(ctx).await? {
                    Some(data) => data,
                    None => {
                        return Ok(None);
                    }
                };

                Ok(Some(TuiInterfaceDrawData::ServerNew(data.into())))
            }
            TuiInterface::ServerSettings(ictx) => {
                let data = match ictx.fetch_snapshot_data(ctx).await? {
                    Some(data) => data,
                    None => {
                        return Ok(None);
                    }
                };

                Ok(Some(TuiInterfaceDrawData::ServerSettings(data.into())))
            }
        }
    }
}
