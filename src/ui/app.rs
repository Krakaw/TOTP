use crate::ui::state::{ActivePane, State};
use crate::ui::widgets::popup::Popup;
use crate::{StorageTrait, TotpError};
use chrono::Utc;
#[cfg(feature = "cli-clipboard")]
use cli_clipboard::set_contents;
use std::ops::Add;
use tui::widgets::{ListState, TableState, Widget};

const POPUP_DELAY: i64 = 750;
pub struct App<W: Widget + ?Sized> {
    /// Application State
    pub state: State<W>,
    /// Table State
    pub table_state: TableState,
    /// Detail View State
    pub detail_state: ListState,
}
impl<W: Widget> App<W> {
    pub fn new<T: StorageTrait>(storage: T) -> Result<Self, TotpError> {
        Ok(Self {
            state: State::new(storage)?,
            table_state: TableState::default(),
            detail_state: ListState::default(),
        })
    }

    pub fn tick(&mut self) {
        if let Some(popup) = self.state.show_popup.as_ref() {
            if let Some(show_until) = popup.show_until {
                if show_until < Utc::now().naive_utc() {
                    self.state.show_popup = None;
                }
            }
        }
    }

    pub fn move_down(&mut self) {
        match self.state.active_pane {
            ActivePane::OtpTable => {
                let selected_index =
                    self.move_down_list(self.table_state.selected(), &self.state.display_otps);
                self.table_state.select(selected_index);
            }
            ActivePane::DetailView => {
                let selected_index =
                    self.move_down_list(self.detail_state.selected(), &vec![0, 0, 0]);
                self.detail_state.select(selected_index);
            }
        }
    }

    pub fn move_up(&mut self) {
        match self.state.active_pane {
            ActivePane::OtpTable => {
                let selected_index =
                    self.move_up_list(self.table_state.selected(), &self.state.display_otps);
                self.table_state.select(selected_index);
            }
            ActivePane::DetailView => {
                let selected_index =
                    self.move_up_list(self.detail_state.selected(), &vec![0, 0, 0]);
                self.detail_state.select(selected_index);
            }
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
        if self.state.active_pane == ActivePane::OtpTable {
            self.state.active_pane = ActivePane::DetailView;
            self.detail_state.select(Some(0));
        } else {
            self.state.active_pane = ActivePane::OtpTable;
            self.detail_state.select(None);
        }
    }

    pub fn set_clipboard(&mut self) {
        #[cfg(feature = "cli-clipboard")]
        match self.state.active_pane {
            ActivePane::OtpTable => {
                if let Some(i) = self.table_state.selected() {
                    self.state.show_popup = Some(Popup::new(
                        "OTP Copied".to_string(),
                        Some("Successfully copied OTP".to_string()),
                        None,
                        Some(
                            Utc::now()
                                .add(chrono::Duration::milliseconds(POPUP_DELAY))
                                .naive_utc(),
                        ),
                    ));
                    set_contents(self.state.display_otps[i].1.clone())
                        .expect("Failed to copy to clipboard");
                }
            }
            ActivePane::DetailView => {
                if let Some(i) = self.table_state.selected() {
                    let record_id = self.state.display_otps[i].3;
                    if let Some(record) = self.state.records.iter().find(|r| r.id == record_id) {
                        let detail_selected_index =
                            self.detail_state.selected().unwrap_or_default();
                        let (value, content) = match detail_selected_index {
                            1 => (
                                record.user.clone().unwrap_or_default(),
                                "Successfully copied username",
                            ),
                            2 => (
                                record.note.clone().unwrap_or_default(),
                                "Successfully copied note",
                            ),
                            _ => (
                                record.password.clone().unwrap_or_default(),
                                "Successfully copied password",
                            ),
                        };
                        self.state.show_popup = Some(Popup::new(
                            "Detail Copied".to_string(),
                            Some(content.to_string()),
                            None,
                            Some(
                                Utc::now()
                                    .add(chrono::Duration::milliseconds(POPUP_DELAY))
                                    .naive_utc(),
                            ),
                        ));
                        set_contents(value).expect("Failed to copy to clipboard");
                    }
                }
            }
        }
    }
}
