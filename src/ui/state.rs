use crate::storage::AccountName;
use crate::{Generator, Storage};

pub enum InputMode {
    Normal,
    Input,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

pub struct State<'a> {
    storage: Storage,
    pub input_mode: InputMode,
    pub filter: String,
    pub items: Vec<(AccountName, Generator<'a>)>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self {
            input_mode: InputMode::default(),
            filter: String::new(),
            items: vec![],
            storage: Storage::default(),
        }
    }
}
impl<'a> State<'a> {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage,
            ..State::default()
        }
    }
}
