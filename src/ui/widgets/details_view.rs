use crate::ui::app::App;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::Text;
use tui::widgets::{Block, Borders, List, ListItem};
use tui::Frame;

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, rect: Rect) {
    let mut block = Block::default().borders(Borders::ALL);
    let mut list_items = vec![];
    if let Some(selected) = app.table_state.selected() {
        if let Some((_, _, _, record_id)) = app.state.display_otps.get(selected) {
            if let Some(record) = app.state.records.iter().find(|r| &r.id == record_id) {
                block = block.title(record.account.clone().unwrap_or_default());

                let mut password =
                    Text::styled("Password:\n", Style::default().add_modifier(Modifier::DIM));
                password.extend(Text::styled(
                    record.clone().password.unwrap_or_else(|| " ".to_string()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                password.extend(Text::raw("_".repeat(frame.size().width as usize)));
                list_items.push(ListItem::new(password));

                let mut username =
                    Text::styled("Username:\n", Style::default().add_modifier(Modifier::DIM));
                username.extend(Text::styled(
                    record.clone().user.unwrap_or_else(|| " ".to_string()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                username.extend(Text::raw("_".repeat(frame.size().width as usize)));
                list_items.push(ListItem::new(username));

                let mut note =
                    Text::styled("Note:\n", Style::default().add_modifier(Modifier::DIM));
                note.extend(Text::styled(
                    record.clone().note.unwrap_or_else(|| " ".to_string()),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                ));
                note.extend(Text::raw("_".repeat(frame.size().width as usize)));
                list_items.push(ListItem::new(note));
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
