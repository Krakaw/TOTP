use crate::ui::state::State;
use crate::Storage;
use cli_clipboard::{ClipboardContext, ClipboardProvider};

pub struct App<'a> {
    /// Application State
    pub state: State<'a>,
}
impl<'a> App<'a> {
    pub fn new(storage: Storage) -> Self {
        Self {
            state: State::new(storage),
        }
    }
}
