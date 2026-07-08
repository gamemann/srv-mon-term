pub mod server;

use std::collections::VecDeque;

use anyhow::Result;
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    context::Context,
    log_debug,
    logger::Logger,
    logger::level::LogLevel,
    server::{Server, data::ServerStatus, types::latency::ServerLatency},
    tui::{
        action::TuiAction,
        interface::{
            context::TuiInterfaceContext,
            ext::TuiInterfaceExt,
            ifaces::{
                dashboard::server::{ROW_HEIGHT, draw_server_row},
                server::view::ServerViewOpts,
            },
            new::TuiInterfaceOpts,
            types::TuiInterfaceType,
        },
    },
};

#[derive(Debug, Clone)]
pub struct TuiInterfaceDashboard {
    pub selected: usize,
}

impl Default for TuiInterfaceDashboard {
    fn default() -> Self {
        Self { selected: 0 }
    }
}

#[derive(Debug, Clone)]
pub struct ServerTableSnapshow {
    pub id: String,
    pub latency_history: VecDeque<ServerLatency>,
    pub server: Server,
    pub status: ServerStatus,
}

#[derive(Debug, Clone, Default)]
pub struct TuiInterfaceDashboardDrawData {
    pub server_snapshots: Vec<ServerTableSnapshow>,
}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceDashboard> {
    type DrawData = TuiInterfaceDashboardDrawData;

    fn title(&self) -> String {
        "Dashboard".to_string()
    }

    fn is_top_level(&self) -> bool {
        true
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::Dashboard
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        None
    }

    async fn prepare(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(String, String)> {
        vec![("Esc".to_string(), "Quit".to_string())]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        match key.code {
            // Handle exitting.
            KeyCode::Esc | KeyCode::Char('q') => return Ok(TuiAction::Exit),

            // Menu controls.
            KeyCode::Up | KeyCode::Char('k') => {
                if self.interface.selected > 0 {
                    self.interface.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.interface.selected += 1;
            }

            // Handle selecting a server.
            KeyCode::Enter => {
                log_debug!(ctx, "Enter key pressed in dashboard interface");

                // Attempt to read servers list.
                let servers = ctx.servers.read().await;

                // Retrieve the selected index, ensuring it is within bounds of the servers list.
                let selected = self.interface.selected.min(servers.len().saturating_sub(1));

                log_debug!(ctx, "Selected server index: {}", selected);

                // Try to read the server context, if we can't, skip this server.
                let srv_ctx = match servers.get(selected) {
                    Some(ctx) => ctx.clone(),
                    None => return Ok(TuiAction::None),
                };

                log_debug!(
                    ctx,
                    "Switching to server view interface for server ID: '{}'",
                    srv_ctx.id
                );

                // Change to the server view interface.
                return Ok(TuiAction::ChangeInterface(
                    TuiInterfaceType::ServerView,
                    Some(TuiInterfaceOpts::ServerView(ServerViewOpts {
                        server_id: srv_ctx.id.clone(),
                    })),
                ));
            }

            _ => {}
        }

        Ok(TuiAction::None)
    }

    fn draw(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        _ctx: Context,
        draw_data: Option<&Self::DrawData>,
    ) {
        // Attempt to read server snapshots from draw data.
        let servers = match draw_data {
            Some(data) => &data.server_snapshots,
            None => return,
        };

        // Check if our server snapshots list is empty, if so display a message to the user.
        if servers.is_empty() {
            let msg = Paragraph::new("No servers configured. Use the CLI to add servers.")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::NONE));

            frame.render_widget(msg, area);

            return;
        }

        // Retrieve the selected index, ensuring it is within bounds of the server snapshots list.
        let selected = self.interface.selected.min(servers.len().saturating_sub(1));

        // Create constraints and rows.
        let constraints: Vec<Constraint> = servers
            .iter()
            .map(|_| Constraint::Length(ROW_HEIGHT))
            .collect();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        for (i, srv_ss) in servers.iter().enumerate() {
            let is_selected = i == selected;

            // Try to read the server context, if we can't, skip this server.
            let server = &srv_ss.server;

            let latency_history = srv_ss.latency_history.clone();

            if i >= rows.len() {
                break;
            }

            // Draw the server row.
            draw_server_row(
                frame,
                rows[i],
                server,
                srv_ss.status.clone(),
                &latency_history,
                is_selected,
            );
        }
    }

    async fn fetch_snapshot_data(&mut self, ctx: Context) -> Result<Option<Self::DrawData>> {
        // Retrieve server snapshots.
        let snapshots = {
            let servers = ctx.servers.read().await;

            let mut snapshots = Vec::new();

            for srv_ctx in servers.iter() {
                let server = srv_ctx.server.read().await;
                let latency_history = srv_ctx.latency.read().await;
                let statuses = srv_ctx.statuses.read().await;

                let status = srv_ctx.get_status(statuses.clone(), ctx.clone()).await;

                snapshots.push(ServerTableSnapshow {
                    id: srv_ctx.id.clone(),
                    latency_history: latency_history.clone(),
                    server: server.clone(),
                    status,
                });
            }

            snapshots
        };

        Ok(Some(Self::DrawData {
            server_snapshots: snapshots,
        }))
    }
}
