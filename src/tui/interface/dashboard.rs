pub mod server;

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
    tui::interface::{
        context::TuiInterfaceContext,
        dashboard::server::{ROW_HEIGHT, draw_server_row},
        ext::TuiInterfaceExt,
        types::TuiInterfaceType,
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

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceDashboard> {
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

    fn get_key_bindings(&self) -> Vec<(&str, &str)> {
        vec![(
            "ESC",
            "
        Quit",
        )]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<()> {
        match key.code {
            // Handle exitting.
            KeyCode::Esc | KeyCode::Char('q') => {
                ctx.cancel_token.cancel();
            }

            // Menu controls.
            KeyCode::Up | KeyCode::Char('k') => {
                if self.interface.selected > 0 {
                    self.interface.selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.interface.selected += 1;
            }

            _ => {}
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame<'_>, area: Rect, ctx: Context) {
        // Attempt to read servers list.
        let srv_guard = match ctx.servers.try_read() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        // Check if our servers list is empty, if so display a message to the user.
        if srv_guard.is_empty() {
            let msg = Paragraph::new("No servers configured. Use the CLI to add servers.")
                .style(Style::default().fg(Color::DarkGray))
                .block(Block::default().borders(Borders::NONE));

            frame.render_widget(msg, area);

            return;
        }

        // Retrieve the selected index, ensuring it is within bounds of the servers list.
        let selected = self
            .interface
            .selected
            .min(srv_guard.len().saturating_sub(1));

        // Create constraints and rows.
        let constraints: Vec<Constraint> = srv_guard
            .iter()
            .map(|_| Constraint::Length(ROW_HEIGHT))
            .collect();

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        for (i, srv_ctx) in srv_guard.iter().enumerate() {
            let is_selected = i == selected;

            // Try to read the server context, if we can't, skip this server.
            let server = match srv_ctx.server.try_read() {
                Ok(g) => g,
                Err(_) => continue,
            };

            // Retrieve latency history.
            let latency_history = match srv_ctx.latency.try_read() {
                Ok(g) => g,
                Err(_) => continue,
            };

            if i >= rows.len() {
                break;
            }

            // Draw the server row.
            draw_server_row(frame, rows[i], &server, &latency_history, is_selected);
        }
    }
}
