use crate::ui::app::App;
use crossterm::style::style;
use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, Cell, List, ListItem, Row, Table};
use tui::Frame;

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, rect: Rect) {
    let selected_style = Style::default().add_modifier(Modifier::REVERSED);
    let normal_style = Style::default().bg(Color::Gray);
    let mut block = Block::default().borders(Borders::ALL);
    let mut list_items = vec![];
    if let Some(selected) = app.table_state.selected() {
        if let Some((_, _, _, record_id)) = app.state.display_otps.get(selected) {
            if let Some(record) = app.state.records.iter().find(|r| &r.id == record_id) {
                block = block.title(record.account.clone().unwrap_or_default());
                list_items.push(ListItem::new(format!(
                    "Username\n{}",
                    record.clone().user.unwrap_or_default()
                )));
                list_items.push(ListItem::new(format!(
                    "Password\n{}",
                    record.clone().password.unwrap_or_default()
                )));
                list_items.push(ListItem::new(format!(
                    "Note\n{}",
                    record.clone().note.unwrap_or_default()
                )));
            }
        }
    }
    let list = List::new(list_items)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::Green),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list.block(block), rect, &mut app.detail_state);
}
