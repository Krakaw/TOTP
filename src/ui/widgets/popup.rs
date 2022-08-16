use crate::ui::widgets::clear::Clear;
use crate::App;
use chrono::NaiveDateTime;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use tui::Frame;

pub struct Popup<W: Widget + ?Sized> {
    pub title: String,
    pub message: Option<String>,
    pub widget: Box<W>,
    pub show_until: Option<NaiveDateTime>,
}

impl<W: Widget> Popup<W> {
    pub fn new(
        title: String,
        message: Option<String>,
        widget: Option<W>,
        show_until: Option<NaiveDateTime>,
    ) -> Popup<W> {
        Popup {
            title,
            show_until,
            message,
            widget,
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

    pub fn render<B: Backend>(&self, app: &mut App<W>, frame: &mut Frame<'_, B>, rect: Rect) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        let paragraph = if let Some(message) = self.message.as_ref() {
            Paragraph::new(message.as_str())
                .block(block)
                .alignment(Alignment::Center)
                .wrap(Wrap { trim: false })
        } else if let Some(widget) = self.widget.as_ref() {
            Paragraph::new(app.state.filter_input.as_ref())
                .style(Style::default().fg(Color::Yellow))
                .block(block)
        } else {
            Paragraph::new("").block(block)
        };

        let area = self.centered_rect(60, 20, rect);
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }
}
