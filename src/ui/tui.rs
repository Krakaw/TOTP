use crate::ui::event_handler::EventHandler;
use crate::ui::state::State;
use crate::{Storage, TotpError};
use crossterm::event::EnableMouseCapture;
use crossterm::terminal;
use crossterm::terminal::EnterAlternateScreen;
use tui::backend::Backend;
use tui::Terminal;

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
        Ok(())
    }

    pub fn draw(&mut self, app: &mut App) -> Result<(), TotpError> {
        self.terminal
            .draw(|frame| renderer::render(app, frame))
            .context("failed to draw TUI")?;
        Ok(())
    }
}
