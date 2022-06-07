use crate::ui::app::App;
use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, Row, Table};
use tui::Frame;

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, rect: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Gray);
    let header_cells = ["Account", "OTP", "Expires In"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(Color::Blue)));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let display_rows = app
        .state
        .items
        .iter()
        .filter(|(account_name, _)| {
            app.state.filter.is_empty()
                || account_name
                    .to_lowercase()
                    .contains(&app.state.filter.to_lowercase())
        })
        .map(|(account_name, generator)| {
            let (code, expiry) = generator.generate(None).unwrap();
            (account_name.to_string(), code, expiry)
        })
        .collect::<Vec<_>>();
    app.state.display_otps = display_rows.clone();
    let rows = display_rows
        .iter()
        .cloned()
        .map(|(account_name, code, expiry)| {
            let height = 1;
            let color = if expiry > 15 {
                Color::Green
            } else if expiry > 5 {
                Color::Yellow
            } else {
                Color::Red
            };
            let cells = vec![
                Cell::from(account_name),
                Cell::from(code),
                Cell::from(expiry.to_string()).style(Style::default().fg(color)),
            ];
            Row::new(cells).height(height as u16).bottom_margin(0)
        });

    let t = Table::new(rows)
        .header(header)
        .block(Block::default().borders(Borders::ALL).title("TOTP"))
        .highlight_style(selected_style)
        .highlight_symbol(">> ")
        .widths(&[
            Constraint::Percentage(75),
            Constraint::Length(6),
            Constraint::Min(6),
        ]);
    if !app.state.items.is_empty() && app.table_state.selected().is_none() {
        app.table_state.select(Some(0));
    }
    frame.render_stateful_widget(t, rect, &mut app.table_state);
}
