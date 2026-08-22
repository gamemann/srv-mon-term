use std::time::Duration;

use anyhow::{Result, bail};
use ratatui::{
    Frame,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    context::Context,
    query::{Query, proto::net::resolve},
    server::{
        setup::server_add,
        types::{
            DEFAULT_QUERY_INTERVAL, DEFAULT_QUERY_TIMEOUT, latency::ServerLatencyType,
            query::ServerQueryType,
        },
    },
    store::server::ServerStore,
    tui::{
        action::TuiAction,
        interface::{
            context::TuiInterfaceContext, ext::TuiInterfaceExt,
            ifaces::server::view::ServerViewOpts, new::TuiInterfaceOpts, types::TuiInterfaceType,
        },
    },
};

/// How long we're willing to wait while checking that the address resolves.
const RESOLVE_TIMEOUT: Duration = Duration::from_secs(2);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerNewField {
    Address,
    Port,
    QueryType,
    QueryPort,
    Name,
    Interval,
    Timeout,
    LatencyType,
    Save,
}

impl ServerNewField {
    pub const ALL: [ServerNewField; 9] = [
        ServerNewField::Address,
        ServerNewField::Port,
        ServerNewField::QueryType,
        ServerNewField::QueryPort,
        ServerNewField::Name,
        ServerNewField::Interval,
        ServerNewField::Timeout,
        ServerNewField::LatencyType,
        ServerNewField::Save,
    ];

    fn label(&self) -> &'static str {
        match self {
            ServerNewField::Address => "Address",
            ServerNewField::Port => "Port",
            ServerNewField::QueryType => "Query Type",
            ServerNewField::QueryPort => "Query Port",
            ServerNewField::Name => "Name",
            ServerNewField::Interval => "Interval",
            ServerNewField::Timeout => "Timeout",
            ServerNewField::LatencyType => "Latency",
            ServerNewField::Save => "Save",
        }
    }

    fn hint(&self) -> &'static str {
        match self {
            ServerNewField::Address => "IP or hostname of the server",
            ServerNewField::Port => "Port players connect to",
            ServerNewField::QueryType => "← → to change, auto is guessed from the port",
            ServerNewField::QueryPort => "Only when queries go to another port (optional)",
            ServerNewField::Name => "Shown instead of the reported name (optional)",
            ServerNewField::Interval => "How often the server is queried, in milliseconds",
            ServerNewField::Timeout => "How long to wait for a reply, in milliseconds",
            ServerNewField::LatencyType => "← → to change how latency is measured",
            ServerNewField::Save => "← → to toggle whether the server is persisted",
        }
    }

    /// Whether the field is edited by typing rather than cycling through values.
    fn is_text(&self) -> bool {
        !matches!(
            self,
            ServerNewField::QueryType | ServerNewField::LatencyType | ServerNewField::Save
        )
    }

    fn is_numeric(&self) -> bool {
        matches!(
            self,
            ServerNewField::Port
                | ServerNewField::QueryPort
                | ServerNewField::Interval
                | ServerNewField::Timeout
        )
    }
}

#[derive(Debug, Clone)]
pub struct TuiInterfaceServerNew {
    pub address: String,
    pub port: String,
    /// `None` means the query type is guessed from the port.
    pub query_type: Option<ServerQueryType>,
    pub query_port: String,
    pub name: String,
    pub interval: String,
    pub timeout: String,
    pub latency_type: ServerLatencyType,
    pub save: bool,

    pub focus: usize,
    pub error: Option<String>,
}

impl Default for TuiInterfaceServerNew {
    fn default() -> Self {
        Self {
            address: String::new(),
            port: String::new(),
            query_type: None,
            query_port: String::new(),
            name: String::new(),
            interval: DEFAULT_QUERY_INTERVAL.to_string(),
            timeout: DEFAULT_QUERY_TIMEOUT.to_string(),
            latency_type: ServerLatencyType::default(),
            save: true,

            focus: 0,
            error: None,
        }
    }
}

impl TuiInterfaceServerNew {
    fn field(&self) -> ServerNewField {
        ServerNewField::ALL[self.focus.min(ServerNewField::ALL.len() - 1)]
    }

    fn text_mut(&mut self, field: ServerNewField) -> Option<&mut String> {
        match field {
            ServerNewField::Address => Some(&mut self.address),
            ServerNewField::Port => Some(&mut self.port),
            ServerNewField::QueryPort => Some(&mut self.query_port),
            ServerNewField::Name => Some(&mut self.name),
            ServerNewField::Interval => Some(&mut self.interval),
            ServerNewField::Timeout => Some(&mut self.timeout),
            _ => None,
        }
    }

    fn value(&self, field: ServerNewField) -> String {
        match field {
            ServerNewField::Address => self.address.clone(),
            ServerNewField::Port => self.port.clone(),
            ServerNewField::QueryType => match self.query_type {
                Some(query_type) => format!("{} ({})", query_type, query_type.description()),
                None => "Auto (from port)".to_string(),
            },
            ServerNewField::QueryPort => self.query_port.clone(),
            ServerNewField::Name => self.name.clone(),
            ServerNewField::Interval => self.interval.clone(),
            ServerNewField::Timeout => self.timeout.clone(),
            ServerNewField::LatencyType => self.latency_type.name().to_string(),
            ServerNewField::Save => if self.save { "Yes" } else { "No" }.to_string(),
        }
    }

    fn focus_next(&mut self) {
        self.focus = (self.focus + 1) % ServerNewField::ALL.len();
    }

    fn focus_prev(&mut self) {
        self.focus = (self.focus + ServerNewField::ALL.len() - 1) % ServerNewField::ALL.len();
    }

    /// Moves choice fields to the next/previous value.
    fn cycle(&mut self, forward: bool) {
        match self.field() {
            ServerNewField::QueryType => {
                // The list is the auto option followed by every supported protocol.
                let len = ServerQueryType::ALL.len() + 1;

                let current = match self.query_type {
                    Some(query_type) => ServerQueryType::ALL
                        .iter()
                        .position(|t| *t == query_type)
                        .map(|idx| idx + 1)
                        .unwrap_or(0),
                    None => 0,
                };

                let next = if forward {
                    (current + 1) % len
                } else {
                    (current + len - 1) % len
                };

                self.query_type = if next == 0 {
                    None
                } else {
                    Some(ServerQueryType::ALL[next - 1])
                };
            }
            ServerNewField::LatencyType => {
                let len = ServerLatencyType::ALL.len();

                let current = ServerLatencyType::ALL
                    .iter()
                    .position(|t| *t == self.latency_type)
                    .unwrap_or(0);

                let next = if forward {
                    (current + 1) % len
                } else {
                    (current + len - 1) % len
                };

                self.latency_type = ServerLatencyType::ALL[next];
            }
            ServerNewField::Save => self.save = !self.save,
            _ => {}
        }
    }

    fn insert(&mut self, c: char) {
        let field = self.field();

        if !field.is_text() {
            return;
        }

        if field.is_numeric() && !c.is_ascii_digit() {
            return;
        }

        if c.is_control() {
            return;
        }

        if let Some(value) = self.text_mut(field) {
            value.push(c);
        }
    }

    fn backspace(&mut self) {
        let field = self.field();

        if let Some(value) = self.text_mut(field) {
            value.pop();
        }
    }

    fn parse_port(value: &str, label: &str) -> Result<u16> {
        match value.trim().parse::<u16>() {
            Ok(port) if port > 0 => Ok(port),
            _ => bail!("{} must be a number between 1 and 65535.", label),
        }
    }

    fn parse_interval(value: &str, label: &str) -> Result<u64> {
        match value.trim().parse::<u64>() {
            Ok(v) if v > 0 => Ok(v),
            _ => bail!("{} must be a number greater than 0.", label),
        }
    }

    /// Validates the form and turns it into a store record.
    async fn build_record(&self) -> Result<ServerStore> {
        let address = self.address.trim().to_string();

        if address.is_empty() {
            bail!("An address is required.");
        }

        if address.contains(char::is_whitespace) || address.contains(':') {
            bail!("Enter the address without a port, e.g. 127.0.0.1.");
        }

        let port = Self::parse_port(&self.port, "Port")?;

        let query_port = if self.query_port.trim().is_empty() {
            None
        } else {
            Some(Self::parse_port(&self.query_port, "Query port")?)
        };

        let query_type = match self.query_type {
            Some(query_type) => query_type,
            None => {
                Query::get_query_type_from_port(query_port.unwrap_or(port)).ok_or_else(|| {
                    anyhow::anyhow!(
                        "Can't guess a query type for port {}. Pick one instead.",
                        query_port.unwrap_or(port)
                    )
                })?
            }
        };

        // Catch typos here instead of showing an offline server forever.
        match tokio::time::timeout(RESOLVE_TIMEOUT, resolve(&address, port)).await {
            Ok(Ok(_)) => {}
            Ok(Err(e)) => bail!("Failed to resolve '{}': {}", address, e),
            Err(_) => bail!("Timed out resolving '{}'.", address),
        }

        let display_name = {
            let name = self.name.trim();

            if name.is_empty() {
                None
            } else {
                Some(name.to_string())
            }
        };

        Ok(ServerStore {
            ip: address,
            port,
            port_query: query_port,
            display_name,
            query_type,
            query_interval: Self::parse_interval(&self.interval, "Interval")?,
            query_timeout: Self::parse_interval(&self.timeout, "Timeout")?,
            latency_type: self.latency_type,
            ..Default::default()
        })
    }

    async fn submit(&mut self, ctx: Context) -> TuiAction {
        let record = match self.build_record().await {
            Ok(record) => record,
            Err(e) => {
                self.error = Some(e.to_string());

                return TuiAction::None;
            }
        };

        match server_add(ctx, record, self.save).await {
            Ok(srv_ctx) => TuiAction::ChangeInterface(
                TuiInterfaceType::ServerView,
                Some(TuiInterfaceOpts::ServerView(ServerViewOpts {
                    server_id: srv_ctx.id.clone(),
                })),
            ),
            Err(e) => {
                self.error = Some(e.to_string());

                TuiAction::None
            }
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct TuiInterfaceServerNewDrawData {}

impl TuiInterfaceExt for TuiInterfaceContext<TuiInterfaceServerNew> {
    type DrawData = TuiInterfaceServerNewDrawData;

    fn title(&self) -> String {
        "Add Server".to_string()
    }

    fn is_top_level(&self) -> bool {
        false
    }

    fn get_type(&self) -> TuiInterfaceType {
        TuiInterfaceType::ServerNew
    }

    fn parent(&self) -> Option<TuiInterfaceType> {
        Some(TuiInterfaceType::Dashboard)
    }

    async fn prepare(&mut self, _ctx: Context) -> Result<()> {
        Ok(())
    }

    async fn cleanup(&mut self, _ctx: Context) -> Result<()> {
        Ok(())
    }

    fn get_key_bindings(&self) -> Vec<(String, String)> {
        vec![
            ("Esc".to_string(), "Cancel".to_string()),
            ("↑↓/Tab".to_string(), "Field".to_string()),
            ("←→".to_string(), "Change".to_string()),
            ("Enter".to_string(), "Add".to_string()),
        ]
    }

    async fn handle_input(&mut self, key: KeyEvent, ctx: Context) -> Result<TuiAction> {
        let form = &mut self.interface;

        match key.code {
            KeyCode::Esc => {
                return Ok(TuiAction::ChangeInterface(
                    TuiInterfaceType::Dashboard,
                    None,
                ));
            }
            KeyCode::Enter => return Ok(form.submit(ctx).await),
            KeyCode::Tab | KeyCode::Down => form.focus_next(),
            KeyCode::BackTab | KeyCode::Up => form.focus_prev(),
            KeyCode::Right => form.cycle(true),
            KeyCode::Left => form.cycle(false),
            KeyCode::Backspace => form.backspace(),
            // Space toggles choice fields but is a normal character everywhere else.
            KeyCode::Char(' ') if !form.field().is_text() => form.cycle(true),
            KeyCode::Char(c) => form.insert(c),
            _ => {}
        }

        Ok(TuiAction::None)
    }

    fn draw(
        &self,
        frame: &mut Frame<'_>,
        area: Rect,
        _ctx: Context,
        _draw_data: Option<&Self::DrawData>,
    ) {
        let form = &self.interface;

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(Span::styled(
                " Add Server ",
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ));

        let inner = block.inner(area);
        frame.render_widget(block, area);

        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(3)])
            .split(inner);

        let mut lines: Vec<Line> = Vec::new();

        for (idx, field) in ServerNewField::ALL.iter().enumerate() {
            let focused = idx == form.focus;

            let label_style = if focused {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };

            let value = form.value(*field);

            let (value, value_style) = if value.is_empty() {
                ("(empty)".to_string(), Style::default().fg(Color::DarkGray))
            } else {
                (value, Style::default().fg(Color::White))
            };

            let mut spans = vec![
                Span::styled(if focused { "▶ " } else { "  " }, label_style),
                Span::styled(format!("{:<12}", field.label()), label_style),
                Span::styled(value, value_style),
            ];

            // Only text fields have a caret to type at.
            if focused && field.is_text() {
                spans.push(Span::styled("▏", Style::default().fg(Color::Yellow)));
            }

            lines.push(Line::from(spans));
        }

        frame.render_widget(Paragraph::new(lines), rows[0]);

        let footer = match &form.error {
            Some(error) => Line::from(Span::styled(
                format!("  {}", error),
                Style::default().fg(Color::LightRed),
            )),
            None => Line::from(Span::styled(
                format!("  {}", form.field().hint()),
                Style::default().fg(Color::DarkGray),
            )),
        };

        frame.render_widget(
            Paragraph::new(vec![
                footer,
                Line::from(Span::styled(
                    "  Enter adds the server, Esc goes back.",
                    Style::default().fg(Color::DarkGray),
                )),
            ]),
            rows[1],
        );
    }

    async fn fetch_snapshot_data(&mut self, _ctx: Context) -> Result<Option<Self::DrawData>> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> TuiInterfaceServerNew {
        TuiInterfaceServerNew {
            address: "127.0.0.1".to_string(),
            port: "27015".to_string(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn guesses_the_query_type_from_the_port() {
        let record = form().build_record().await.expect("failed to build");

        assert_eq!(record.query_type, ServerQueryType::A2s);
        assert_eq!(record.port, 27015);
        assert_eq!(record.query_interval, DEFAULT_QUERY_INTERVAL);
        assert_eq!(record.query_timeout, DEFAULT_QUERY_TIMEOUT);
        assert_eq!(record.display_name, None);
    }

    #[tokio::test]
    async fn uses_the_query_port_when_given() {
        let mut form = form();

        form.query_port = "27016".to_string();
        form.query_type = Some(ServerQueryType::Quake3);
        form.name = " My Server ".to_string();

        let record = form.build_record().await.expect("failed to build");

        assert_eq!(record.port_query, Some(27016));
        assert_eq!(record.query_type, ServerQueryType::Quake3);
        assert_eq!(record.display_name.as_deref(), Some("My Server"));
    }

    #[tokio::test]
    async fn rejects_bad_input() {
        let mut empty = form();
        empty.address = String::new();
        assert!(empty.build_record().await.is_err());

        let mut with_port = form();
        with_port.address = "127.0.0.1:27015".to_string();
        assert!(with_port.build_record().await.is_err());

        let mut bad_port = form();
        bad_port.port = "0".to_string();
        assert!(bad_port.build_record().await.is_err());

        // Port 1234 matches no protocol, so the user has to pick one.
        let mut unknown_port = form();
        unknown_port.port = "1234".to_string();
        assert!(unknown_port.build_record().await.is_err());

        let mut zero_interval = form();
        zero_interval.interval = "0".to_string();
        assert!(zero_interval.build_record().await.is_err());
    }

    #[test]
    fn cycles_choice_fields() {
        let mut form = form();

        form.focus = ServerNewField::ALL
            .iter()
            .position(|f| *f == ServerNewField::QueryType)
            .unwrap();

        form.cycle(true);
        assert_eq!(form.query_type, Some(ServerQueryType::ALL[0]));

        form.cycle(false);
        assert_eq!(form.query_type, None);

        form.cycle(false);
        assert_eq!(
            form.query_type,
            Some(ServerQueryType::ALL[ServerQueryType::ALL.len() - 1])
        );

        form.focus = ServerNewField::ALL
            .iter()
            .position(|f| *f == ServerNewField::Save)
            .unwrap();

        assert!(form.save);
        form.cycle(true);
        assert!(!form.save);
    }

    #[test]
    fn only_accepts_digits_in_numeric_fields() {
        let mut form = form();

        form.focus = ServerNewField::ALL
            .iter()
            .position(|f| *f == ServerNewField::Port)
            .unwrap();

        form.port.clear();
        form.insert('2');
        form.insert('a');
        form.insert('7');

        assert_eq!(form.port, "27");

        form.focus = ServerNewField::ALL
            .iter()
            .position(|f| *f == ServerNewField::Name)
            .unwrap();

        form.insert('a');
        form.insert('!');

        assert_eq!(form.name, "a!");
    }
}
