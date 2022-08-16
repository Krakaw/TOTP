use crate::ui::app::App;
use crate::ui::state::InputMode;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, rect: Rect) {
    let border_type = if app.state.input_mode == InputMode::FilterList {
        BorderType::Thick
    } else {
        BorderType::Plain
    };
    let input = Paragraph::new(app.state.filter_input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Filter")
                .border_type(border_type),
        );
    match app.state.input_mode {
        InputMode::FilterList => {
            frame.set_cursor(
                // Put cursor past the end of the input text
                rect.x + app.state.filter_input.len() as u16 + 1,
                // Move one line down, from the border to the input line
                rect.y + 1,
            );
        }
        _ => {}
    }
    frame.render_widget(input, rect);
}
