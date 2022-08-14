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
                let selected_index =
                    self.move_down_list(self.table_state.selected(), &self.state.display_otps);
                self.table_state.select(selected_index);
            }
            InputMode::Details => {
                let selected_index =
                    self.move_down_list(self.detail_state.selected(), &vec![0, 0, 0]);
                self.detail_state.select(selected_index);
            }
            _ => {}
        }
    }

    pub fn move_up(&mut self) {
        match self.state.input_mode {
            InputMode::Normal | InputMode::Input => {
                let selected_index =
                    self.move_up_list(self.table_state.selected(), &self.state.display_otps);
                self.table_state.select(selected_index);
            }
            InputMode::Details => {
                let selected_index =
                    self.move_up_list(self.detail_state.selected(), &vec![0, 0, 0]);
                self.detail_state.select(selected_index);
            }
            _ => {}
        }
    }

    fn move_down_list<T>(&self, selected: Option<usize>, items: &Vec<T>) -> Option<usize> {
        let i = match selected {
            Some(i) => {
                if !items.is_empty() && i >= items.len() - 1 {
                    0
                } else {
                    usize::min(i + 1, items.len())
                }
            }
            None => 0,
        };
        Some(i)
    }

    fn move_up_list<T>(&self, selected: Option<usize>, items: &Vec<T>) -> Option<usize> {
        let selected_value = match selected {
            Some(i) => {
                if !items.is_empty() && i == 0 {
                    items.len() - 1
                } else {
                    usize::max(i, 1) - 1
                }
            }
            None => 0,
        };
        Some(selected_value)
    }

    pub fn toggle_list_detail_mode(&mut self) {
        if self.state.input_mode == InputMode::Normal {
            self.state.input_mode = InputMode::Details;
            self.detail_state.select(Some(0));
        } else if self.state.input_mode == InputMode::Details {
            self.state.input_mode = InputMode::Normal;
            self.detail_state.select(None);
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
