use crate::storage::accounts::AccountName;
use crate::{Generator, Storage, TotpError};

pub type TotpAccountName = String;
pub type TotpCode = String;
type ExpirySeconds = u64;
pub enum InputMode {
    Normal,
    Input,
    AddOtp,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

pub struct State {
    pub input_mode: InputMode,
    pub filter: String,
    pub items: Vec<(AccountName, Option<Generator>)>,
    pub display_otps: Vec<(TotpAccountName, TotpCode, ExpirySeconds)>,
    pub running: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            input_mode: InputMode::default(),
            filter: String::new(),
            items: vec![],
            display_otps: vec![],
            running: true,
        }
    }
}

impl State {
    pub fn new(storage: Storage) -> Result<Self, TotpError> {
        let mut items = vec![];
        for (account_name, secure_data) in storage.accounts.iter() {
            let generator = secure_data
                .token
                .as_ref()
                .map(|t| Generator::new(t.to_owned()))
                .and_then(|g| g.ok());
            items.push((account_name.clone(), generator));
        }
        items.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(Self {
            items,
            ..State::default()
        })
    }
}
