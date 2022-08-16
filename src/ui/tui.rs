use crate::ui::app::App;
use crate::ui::event_handler::EventHandler;
use crate::ui::widgets::{details_view, filter_input, otp_table};
use crate::TotpError;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::terminal;
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::io;
use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::Widget;
use tui::{Frame, Terminal};

pub struct Tui<B: Backend> {
    terminal: Terminal<B>,
    pub events: EventHandler,
}

impl<B: Backend> Tui<B> {
    pub fn new(terminal: Terminal<B>, events: EventHandler) -> Self {
        Self { terminal, events }
    }

    pub fn init(&mut self) -> Result<(), TotpError> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;
        let original_hook = std::panic::take_hook();

        std::panic::set_hook(Box::new(move |panic| {
            disable_raw_mode().unwrap();
            crossterm::execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture).unwrap();
            original_hook(panic);
        }));
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<(), TotpError> {
        self.terminal
            .draw(|frame| render(app, frame))
            .map_err(|e| TotpError::Ui(e.to_string()))?;
        Ok(())
    }

    pub fn exit(&mut self) -> Result<(), TotpError> {
        terminal::disable_raw_mode()?;
        crossterm::execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

fn render<B: Backend>(app: &mut App, frame: &mut Frame<'_, B>) {
    if let Some(popup) = app.state.show_popup.as_ref() {
        let rect = frame.size();
        popup.render(app, frame, rect);
    } else {
        let rects = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
            .margin(1)
            .split(frame.size());
        filter_input::render(app, frame, rects[0]);
        let body_rects = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .margin(0)
            .split(rects[1]);
        otp_table::render(app, frame, body_rects[0]);
        details_view::render(app, frame, body_rects[1]);
    }
}
