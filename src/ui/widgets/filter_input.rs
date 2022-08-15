use crate::ui::app::App;
use crate::ui::state::InputMode;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;

pub fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>, rect: Rect) {
    let input = Paragraph::new(app.state.filter.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Filter"));
    match app.state.input_mode {
        InputMode::Input => {
            frame.set_cursor(
                // Put cursor past the end of the input text
                rect.x + app.state.filter.len() as u16 + 1,
                // Move one line down, from the border to the input line
                rect.y + 1,
            );
        }
        InputMode::Normal => {}
        InputMode::AddOtp => {}
        InputMode::Details => {}
    }
    frame.render_widget(input, rect);
}
