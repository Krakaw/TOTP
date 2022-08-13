use crate::ui::state::{InputMode, State};
use crate::{Storage, StorageTrait, TotpError};
#[cfg(feature = "cli-clipboard")]
use cli_clipboard::set_contents;
use tui::widgets::{ListState, TableState};

pub struct App {
    /// Application State
    pub state: State,
    /// Table State
    pub table_state: TableState,
    /// Detail View State
    pub detail_state: ListState,
}
impl App {
    pub fn new<T: StorageTrait>(storage: T) -> Result<Self, TotpError> {
        Ok(Self {
            state: State::new(storage)?,
            table_state: TableState::default(),
            detail_state: ListState::default(),
        })
    }

    pub fn tick(&mut self) {}

    pub fn move_down(&mut self) {
        match self.state.input_mode {
            InputMode::Normal | InputMode::Input => {
                self.move_down_table();
            }
            InputMode::Details => {}
            _ => {}
        }
    }
    fn move_down_table(&mut self) {
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
        match self.state.input_mode {
            InputMode::Normal | InputMode::Input => {
                self.move_up_table();
            }
            InputMode::Details => {}
            _ => {}
        }
    }

    fn move_up_table(&mut self) {
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

    pub fn toggle_list_detail_mode(&mut self) {
        if self.state.input_mode == InputMode::Normal {
            self.state.input_mode = InputMode::Details;
        } else if self.state.input_mode == InputMode::Details {
            self.state.input_mode = InputMode::Normal;
        }
    }

    pub fn set_clipboard(&mut self) {
        #[cfg(feature = "cli-clipboard")]
        if let Some(i) = self.table_state.selected() {
            set_contents(self.state.display_otps[i].1.clone())
                .expect("Failed to copy to clipboard");
        }
    }
}
