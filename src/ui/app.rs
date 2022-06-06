use crate::ui::state::State;
use crate::{Storage, TotpError};
use cli_clipboard::set_contents;
use tui::widgets::TableState;

pub struct App {
    /// Application State
    pub state: State,
    /// Table State
    pub table_state: TableState,
}
impl App {
    pub fn new(storage: Storage) -> Result<Self, TotpError> {
        Ok(Self {
            state: State::new(storage)?,
            table_state: TableState::default(),
        })
    }

    pub fn tick(&mut self) {}

    pub fn move_down(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if !self.state.display_otps.is_empty() && i >= self.state.display_otps.len() - 1 {
                    0
                } else {
                    usize::min(i + 1, self.state.display_otps.len())
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn move_up(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if !self.state.display_otps.is_empty() && i == 0 {
                    self.state.display_otps.len() - 1
                } else {
                    usize::max(i, 1) - 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    pub fn set_clipboard(&mut self) {
        if let Some(i) = self.table_state.selected() {
            set_contents(self.state.display_otps[i].1.clone())
                .expect("Failed to copy to clipboard");
        }
    }
}
