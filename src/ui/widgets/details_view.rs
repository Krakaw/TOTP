use crate::ui::app::App;
use crate::ui::state::{ActivePane, InputMode};
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Modifier, Style};
use tui::text::Text;
use tui::widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Widget};
use tui::Frame;

pub fn render<B: Backend, W: Widget>(app: &mut App<W>, frame: &mut Frame<'_, B>, rect: Rect) {
    let border_type = if app.state.active_pane == ActivePane::DetailView {
        BorderType::Thick
    } else {
        BorderType::Plain
    };
    let mut block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type);
    let mut list_items = vec![];
    if let Some(selected) = app.table_state.selected() {
        if let Some((_, _, _, record_id)) = app.state.display_otps.get(selected) {
            if let Some(record) = app.state.records.iter().find(|r| &r.id == record_id) {
                block = block.title(record.account.clone().unwrap_or_default());

                let hidden = app.state.active_pane != ActivePane::DetailView;
                let frame_size = frame.size().width as usize;
                list_items.push(list_item(
                    "Password",
                    record.password.clone(),
                    frame_size,
                    hidden,
                ));
                list_items.push(list_item(
                    "Username",
                    record.user.clone(),
                    frame_size,
                    hidden,
                ));
                list_items.push(list_item("Note", record.note.clone(), frame_size, hidden));
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

fn list_item(title: &str, value: Option<String>, frame_size: usize, hidden: bool) -> ListItem {
    let mut title_text = Text::styled(
        format!("{}:\n", title),
        Style::default().add_modifier(Modifier::DIM),
    );
    let value_text = if hidden {
        "*".repeat(frame_size)
    } else {
        value.unwrap_or_else(|| " ".to_string())
    };
    title_text.extend(Text::styled(
        value_text,
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    title_text.extend(Text::raw("_".repeat(frame_size)));

    ListItem::new(title_text)
}
