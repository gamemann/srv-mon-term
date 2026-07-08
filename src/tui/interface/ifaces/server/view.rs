pub mod general;
pub mod latency;
pub mod users;
pub mod vars;

use std::{collections::VecDeque, sync::Arc};

use anyhow::{Result, anyhow};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::{
    cli::QueryMonitor,
    context::Context,
    log_error,
    server::{
        Server, ServerCtx, ServerStatuses, data::ServerStatus, types::latency::ServerLatency,
    },
    tui::{
        action::TuiAction,
        interface::{
            context::TuiInterfaceContext,
            ext::TuiInterfaceExt,
            ifaces::server::view::{
                general::draw_server_general, latency::draw_server_latency_detail,
                users::draw_server_users, vars::draw_server_vars,
            },
            types::TuiInterfaceType,
        },
    },
};

#[derive(Default, Debug, Clone)]
pub struct ServerViewOpts {
    pub server_id: String,
}

impl ServerViewOpts {
    pub fn new(server_id: String) -> Self {
        Self { server_id }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerView {
    pub server_id: String,

    pub focus: ServerViewFocus,
    pub users_selected: usize,
    pub vars_selected: usize,
}

impl From<ServerViewOpts> for TuiInterfaceServerView {
    fn from(opts: ServerViewOpts) -> Self {
        Self {
            server_id: opts.server_id,
            ..Default::default()
        }
    }
}

impl TuiInterfaceServerView {
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            ..Default::default()
        }
    }

    pub fn get_server_by_id(&self, ctx: Context) -> Result<Arc<ServerCtx>> {
        let servers = match ctx.servers.try_read() {
            Ok(servers) => servers,
            Err(_) => {
                return Err(anyhow!("Failed to acquire read lock on servers"));
            }
        };

        let srv_ctx = servers
            .iter()
            .find(|s| s.id == self.server_id)
            .cloned()
            .ok_or_else(|| anyhow!("Failed to find server context"))?;

        Ok(srv_ctx)
    }
}

/// Which of the two bottom (list) panels currently has input focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ServerViewFocus {
    #[default]
    Users,
    Vars,
}

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerViewDrawData {
    pub id: String,
    pub server: Server,
    pub latency_history: VecDeque<ServerLatency>,
    pub statuses: ServerStatuses,
    pub status: ServerStatus,
}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerView> {
    type DrawData = TuiInterfaceServerViewDrawData;

    fn title(&self) -> String {
        "Server View".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::ServerView
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::Dashboard)
    }

    async fn prepare(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(String, String)> {
        vec![
            ("Esc".to_string(), "Back".to_string()),
            ("Tab".to_string(), "Switch panel".to_string()),
            ("↑↓".to_string(), "Select".to_string()),
            ("Home/End".to_string(), "Jump".to_string()),
            ("e".to_string(), "Edit".to_string()),
        ]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                return Ok(TuiAction::ChangeInterface(
                    TuiInterfaceType::Dashboard,
                    None,
                ));
            }
            KeyCode::Tab => {
                self.interface.focus = match self.interface.focus {
                    ServerViewFocus::Users => ServerViewFocus::Vars,
                    ServerViewFocus::Vars => ServerViewFocus::Users,
                };
            }

            KeyCode::Up | KeyCode::Char('k') => {
                let sel = match self.interface.focus {
                    ServerViewFocus::Users => &mut self.interface.users_selected,
                    ServerViewFocus::Vars => &mut self.interface.vars_selected,
                };

                *sel = sel.saturating_sub(1);
            }

            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.focused_list_len(ctx.clone()).await;

                let sel = match self.interface.focus {
                    ServerViewFocus::Users => &mut self.interface.users_selected,
                    ServerViewFocus::Vars => &mut self.interface.vars_selected,
                };

                if len > 0 && *sel + 1 < len {
                    *sel += 1;
                }
            }

            KeyCode::Home => {
                let sel = match self.interface.focus {
                    ServerViewFocus::Users => &mut self.interface.users_selected,
                    ServerViewFocus::Vars => &mut self.interface.vars_selected,
                };

                *sel = 0;
            }

            KeyCode::End => {
                let len = self.focused_list_len(ctx.clone()).await;

                let sel = match self.interface.focus {
                    ServerViewFocus::Users => &mut self.interface.users_selected,
                    ServerViewFocus::Vars => &mut self.interface.vars_selected,
                };

                *sel = len.saturating_sub(1);
            }

            _ => {}
        }

        Ok(TuiAction::None)
    }

    fn draw(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        ctx: Context,
        draw_data: Option<&Self::DrawData>,
    ) {
        let draw_data = match draw_data {
            Some(data) => data,
            None => return,
        };

        let server = &draw_data.server;
        let latency_history = &draw_data.latency_history;

        // Split into a 2x2 grid: general/latency on top, users/vars on bottom.
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(area);

        let top_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(rows[0]);

        let bottom_cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(rows[1]);

        // Retrieve server status.
        let status = &draw_data.status;

        draw_server_general(frame, top_cols[0], &server, status.clone());
        draw_server_latency_detail(frame, top_cols[1], &server.latency_type, &latency_history);

        let user_status = &draw_data.statuses.query_users;
        let user_err_num = if let ServerStatus::Error(code) = user_status {
            Some(code.clone())
        } else {
            None
        };

        draw_server_users(
            frame,
            bottom_cols[0],
            &server.data.users,
            self.interface.users_selected,
            self.interface.focus == ServerViewFocus::Users,
            matches!(user_status, ServerStatus::Error(_)),
            user_err_num,
        );

        let vars_status = &draw_data.statuses.query_vars;
        let vars_err_num = if let ServerStatus::Error(code) = vars_status {
            Some(code.clone())
        } else {
            None
        };

        draw_server_vars(
            frame,
            bottom_cols[1],
            &server.data.vars,
            self.interface.vars_selected,
            self.interface.focus == ServerViewFocus::Vars,
            matches!(vars_status, ServerStatus::Error(_)),
            vars_err_num,
        );
    }

    async fn fetch_snapshot_data(&mut self, ctx: Context) -> Result<Option<Self::DrawData>> {
        // Fetch servers.
        let id = self.interface.server_id.clone();

        let srv_ctx = ServerCtx::get_server_ctx_by_id(ctx.clone(), &id).await?;
        let server = srv_ctx.server.read().await.clone();
        let statuses = srv_ctx.statuses.read().await;

        let status = srv_ctx.get_status(statuses.clone(), ctx.clone()).await;

        Ok(Some(Self::DrawData {
            id,
            server,
            latency_history: srv_ctx.latency.read().await.clone(),
            statuses: statuses.clone(),
            status,
        }))
    }
}

impl TuiInterfaceContext<TuiInterfaceServerView> {
    /// Retrieves the length of whichever list (users/vars) currently has focus.
    async fn focused_list_len(&self, ctx: Context) -> usize {
        let srv_ctx = match self.interface.get_server_by_id(ctx) {
            Ok(srv_ctx) => srv_ctx,
            Err(_) => return 0,
        };

        let server = srv_ctx.server.read().await;

        match self.interface.focus {
            ServerViewFocus::Users => server.data.users.len(),
            ServerViewFocus::Vars => server.data.vars.len(),
        }
    }
}
