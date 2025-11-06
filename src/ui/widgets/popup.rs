#![allow(dead_code)]
use crate::ui::widgets::clear::Clear;
use chrono::NaiveDateTime;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::Style;
use tui::text::Text;
use tui::widgets::{Block, BorderType, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tui::Frame;

#[derive(Clone)]
pub enum Position {
    Top,
    Center,
    Bottom,
}

impl Default for Position {
    fn default() -> Self {
        Self::Center
    }
}

#[derive(Clone)]
pub struct Size {
    pub x: u16,
    pub y: u16,
}

impl Default for Size {
    fn default() -> Self {
        Self { x: 60, y: 20 }
    }
}
pub struct Popup {
    pub title: String,
    pub message: Option<String>,
    pub style: Option<Style>,
    pub show_background: Option<bool>,
    pub show_until: Option<NaiveDateTime>,
    pub size: Option<Size>,
    pub position: Option<Position>,
}

impl Popup {
    pub fn new(
        title: String,
        message: Option<String>,
        show_until: Option<NaiveDateTime>,
        show_background: Option<bool>,
        style: Option<Style>,
        size: Option<Size>,
        position: Option<Position>,
    ) -> Popup {
        Popup {
            title,
            show_until,
            message,
            show_background,
            style,
            size,
            position,
        }
    }

    fn vertical_constraints(&self, percent_y: u16) -> Vec<Constraint> {
        match self.position.clone().unwrap_or_default() {
            Position::Top => {
                vec![
                    Constraint::Percentage(0),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage(100 - percent_y),
                ]
            }
            Position::Center => {
                vec![
                    Constraint::Percentage((100 - percent_y) / 2),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage((100 - percent_y) / 2),
                ]
            }
            Position::Bottom => {
                vec![
                    Constraint::Percentage(100 - percent_y),
                    Constraint::Percentage(percent_y),
                    Constraint::Percentage(0),
                ]
            }
        }
    }
    pub fn centered_rect(&self, size: Size, r: Rect) -> Rect {
        let Size { x, y } = size;
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(self.vertical_constraints(y))
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage((100 - x) / 2),
                    Constraint::Percentage(x),
                    Constraint::Percentage((100 - x) / 2),
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

        let area = self.centered_rect(self.size.clone().unwrap_or_default(), rect);
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

        let area = self.centered_rect(self.size.clone().unwrap_or_default(), rect);
        if self.show_background.is_none() || self.show_background == Some(false) {
            frame.render_widget(Clear, rect);
        }
        frame.render_widget(Clear, area); //this clears out the background
        frame.render_widget(paragraph, area);
    }

    pub fn render_text_input<B: Backend>(
        &self,
        frame: &mut Frame<'_, B>,
        rect: Rect,
        input_text: &str,
        show_cursor: bool,
    ) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        let instruction = if let Some(message) = self.message.as_ref() {
            format!("{}\n\n{}", input_text, message)
        } else {
            format!("{}\n\nEnter to save, Esc to cancel", input_text)
        };

        let paragraph = Paragraph::new(instruction.as_str())
            .block(block)
            .style(self.style.unwrap_or_default())
            .wrap(Wrap { trim: false });

        let area = self.centered_rect(self.size.clone().unwrap_or_default(), rect);
        if self.show_background.is_none() || self.show_background == Some(false) {
            frame.render_widget(Clear, rect);
        }
        frame.render_widget(Clear, area);
        frame.render_widget(paragraph, area);

        if show_cursor {
            // Set cursor position at the end of the input text
            let input_line_y = area.y + 1;
            let cursor_x = area.x + input_text.len() as u16 + 1;
            frame.set_cursor(cursor_x, input_line_y);
        }
    }

    pub fn render_help<B: Backend>(
        &self,
        frame: &mut Frame<'_, B>,
        rect: Rect,
        help_state: &mut ListState,
    ) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL)
            .border_type(BorderType::Thick);

        let help_items = if let Some(message) = self.message.as_ref() {
            message
                .lines()
                .map(|line| {
                    if line.is_empty() {
                        ListItem::new("")
                    } else if line.starts_with("  ") {
                        // Regular help line - format with proper spacing
                        ListItem::new(Text::raw(line))
                    } else {
                        // Section header - make it bold
                        ListItem::new(Text::styled(
                            line,
                            Style::default().add_modifier(tui::style::Modifier::BOLD),
                        ))
                    }
                })
                .collect::<Vec<_>>()
        } else {
            vec![]
        };

        let list = List::new(help_items)
            .block(block)
            .style(self.style.unwrap_or_default());

        let area = self.centered_rect(self.size.clone().unwrap_or_default(), rect);
        if self.show_background.is_none() || self.show_background == Some(false) {
            frame.render_widget(Clear, rect);
        }
        frame.render_widget(Clear, area);
        frame.render_stateful_widget(list, area, help_state);
    }
}
