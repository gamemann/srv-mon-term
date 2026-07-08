use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};

use crate::server::types::var::ServerVar;

pub fn draw_server_vars(
    frame: &mut Frame<'_>,
    area: Rect,
    vars: &[ServerVar],
    selected: usize,
    focused: bool,
    is_error: bool,
    err_code: Option<u16>,
) {
    if is_error {
        let err = Paragraph::new(Span::styled(
            format!("Error fetching vars list. Code: {:?}", err_code),
            Style::default().fg(Color::Red),
        ));

        frame.render_widget(err, area);

        return;
    }

    let border_color = if focused {
        Color::Yellow
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(
            format!(" Vars ({}) ", vars.len()),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if vars.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "No server vars available",
                Style::default().fg(Color::DarkGray),
            )),
            inner,
        );
        return;
    }

    // Sort alphabetically for predictable scanning.
    let mut sorted: Vec<&ServerVar> = vars.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let header = Row::new(vec![Cell::from("Name"), Cell::from("Value")]).style(
        Style::default()
            .fg(Color::DarkGray)
            .add_modifier(Modifier::BOLD),
    );

    let rows: Vec<Row> = sorted
        .iter()
        .map(|v| {
            Row::new(vec![
                Cell::from(v.name.clone()),
                Cell::from(v.value.clone()),
            ])
        })
        .collect();

    let widths = [Constraint::Percentage(45), Constraint::Percentage(55)];

    let table = Table::new(rows, widths)
        .header(header)
        .row_highlight_style(Style::default().fg(Color::Black).bg(Color::Yellow))
        .highlight_symbol("▶ ");

    let mut state = TableState::default();
    let clamped = selected.min(sorted.len().saturating_sub(1));
    state.select(Some(clamped));

    frame.render_stateful_widget(table, inner, &mut state);
}
