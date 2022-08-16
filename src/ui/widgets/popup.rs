use crate::ui::widgets::clear::Clear;
use crate::App;
use chrono::NaiveDateTime;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use tui::Frame;

pub struct Popup {
    pub title: String,
    pub message: Option<String>,
    pub show_until: Option<NaiveDateTime>,
}

impl Popup {
    pub fn new(title: String, message: Option<String>, show_until: Option<NaiveDateTime>) -> Popup {
        Popup {
            title,
            show_until,
            message,
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

    pub fn render<B: Backend>(&self, app: &App, frame: &mut Frame<'_, B>, rect: Rect) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        let paragraph = if let Some(message) = self.message.as_ref() {
            Paragraph::new(message.as_str())
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
        } else {
            Paragraph::new("").block(block)
        };

        let area = self.centered_rect(60, 20, rect);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }

    pub fn render_paragraph<B: Backend>(
        &self,
        frame: &mut Frame<'_, B>,
        rect: Rect,
        paragraph: Paragraph,
        percent_x: u16,
        percent_y: u16,
    ) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        let paragraph = paragraph.block(block);

        let area = self.centered_rect(percent_x, percent_y, rect);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }
}
