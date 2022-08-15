use crate::ui::widgets::clear::Clear;
use chrono::NaiveDateTime;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

pub struct Popup {
    pub title: String,
    pub content: String,
    pub show_until: NaiveDateTime,
}

impl Popup {
    pub fn new(title: String, content: String, show_until: NaiveDateTime) -> Popup {
        Popup {
            title,
            content,
            show_until,
        }
    }

    pub fn centered_rect(&self, percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
                .as_ref(),
            )
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - percent_x) / 2),
                    Constraint::Percentage(percent_x),
                    Constraint::Percentage((100 - percent_x) / 2),
                ]
                .as_ref(),
            )
            .split(popup_layout[1])[1]
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<'_, B>, rect: Rect) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        let paragraph = Paragraph::new(self.content.as_str())
            .block(block)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });
        let area = self.centered_rect(60, 20, rect);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }
}
