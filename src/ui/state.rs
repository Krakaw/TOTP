use crate::storage::accounts::AccountName;
use crate::{Generator, Record, Storage, StorageTrait, TotpError};

pub type TotpAccountName = String;
pub type TotpCode = String;
type ExpirySeconds = u64;
type RecordId = u32;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    Input,
    Details,
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
    pub items: Vec<(AccountName, Option<Generator>, RecordId)>,
    pub records: Vec<Record>,
    pub display_otps: Vec<(TotpAccountName, TotpCode, ExpirySeconds, RecordId)>,
    pub running: bool,
}

impl Default for State {
    fn default() -> Self {
        Self {
            input_mode: InputMode::default(),
            filter: String::new(),
            items: vec![],
            records: vec![],
            display_otps: vec![],
            running: true,
        }
    }
}

impl State {
    pub fn new<T: StorageTrait>(storage: T) -> Result<Self, TotpError> {
        let mut items = vec![];
        let mut records = vec![];
        for (account_name, record) in storage.accounts()?.iter() {
            records.push(record.clone());
            let generator = record
                .token
                .as_ref()
                .map(|t| Generator::new(t.to_owned()))
                .and_then(|g| g.ok());
            items.push((account_name.clone(), generator, record.id));
        }
        items.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(Self {
            items,
            records,
            ..State::default()
        })
    }
}
