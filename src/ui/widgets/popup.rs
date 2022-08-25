use crate::ui::widgets::clear::Clear;
use chrono::NaiveDateTime;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::Style;
use tui::widgets::{Block, Borders, Paragraph, Wrap};
use tui::Frame;

pub struct Popup {
    pub title: String,
    pub message: Option<String>,
    pub style: Option<Style>,
    pub show_background: Option<bool>,
    pub show_until: Option<NaiveDateTime>,
    pub percent_x: Option<u16>,
    pub percent_y: Option<u16>,
}

impl Popup {
    pub fn new(
        title: String,
        message: Option<String>,
        show_until: Option<NaiveDateTime>,
        show_background: Option<bool>,
        style: Option<Style>,
        percent_x: Option<u16>,
        percent_y: Option<u16>,
    ) -> Popup {
        Popup {
            title,
            show_until,
            message,
            show_background,
            style,
            percent_x,
            percent_y,
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
        let paragraph = if let Some(message) = self.message.as_ref() {
            Paragraph::new(message.as_str())
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
                .style(self.style.unwrap_or_default())
        } else {
            Paragraph::new("").block(block)
        };

        let area = self.centered_rect(
            self.percent_x.unwrap_or(60),
            self.percent_y.unwrap_or(20),
            rect,
        );
        if self.show_background.is_none() || self.show_background == Some(false) {
            frame.render_widget(Clear, rect);
        }
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }

    pub fn render_paragraph<B: Backend>(
        &self,
        frame: &mut Frame<'_, B>,
        rect: Rect,
        paragraph: Paragraph,
    ) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        let paragraph = paragraph.block(block);

        let area = self.centered_rect(
            self.percent_x.unwrap_or(60),
            self.percent_y.unwrap_or(20),
            rect,
        );
        if self.show_background.is_none() || self.show_background == Some(false) {
            frame.render_widget(Clear, rect);
        }
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }
}
