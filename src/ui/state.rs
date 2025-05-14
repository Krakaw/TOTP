#![allow(dead_code)]
use crate::db::models::record::AccountName;
use crate::ui::widgets::popup::Popup;
use crate::{Generator, Record, StorageTrait, TotpError};

pub type TotpAccountName = String;
pub type TotpCode = String;
type ExpirySeconds = u64;
type RecordId = u32;

#[derive(PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    FilterList,
    EditDetail,
    DeleteConfirmation,
}

#[derive(PartialEq, Eq, Default)]
pub enum ActivePane {
    #[default]
    OtpTable,
    DetailView,
}

#[derive(PartialEq, Eq)]
pub enum DetailInputType {
    Password,
    Username,
    Note,
}

pub struct State {
    pub input_mode: InputMode,
    pub active_pane: ActivePane,
    pub detail_input_type: DetailInputType,
    pub detail_input: String,
    pub filter_input: String,
    pub items: Vec<(AccountName, Option<Generator>, RecordId)>,
    pub records: Vec<Record>,
    pub display_otps: Vec<(TotpAccountName, TotpCode, ExpirySeconds, RecordId)>,
    pub running: bool,
    pub show_popup: Option<Popup>,
    pub storage: Option<Box<dyn StorageTrait + 'static>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            input_mode: InputMode::default(),
            active_pane: ActivePane::default(),
            detail_input_type: DetailInputType::Password,
            detail_input: String::new(),
            filter_input: String::new(),
            items: vec![],
            records: vec![],
            display_otps: vec![],
            running: true,
            show_popup: None,
            storage: None,
        }
    }
}

impl State {
    pub fn new<T: StorageTrait + 'static>(storage: T) -> Result<Self, TotpError> {
        let mut state = Self {
            items: vec![],
            records: vec![],
            storage: Some(Box::new(storage)),
            ..State::default()
        };
        state.build_records()?;
        Ok(state)
    }

    pub fn build_records(&mut self) -> Result<(), TotpError> {
        let mut items = vec![];
        let mut records = vec![];
        for record in self
            .storage
            .as_ref()
            .ok_or(TotpError::Storage("Storage not found".to_string()))?
            .accounts()?
            .iter()
        {
            records.push(record.clone());
            let generator = record
                .token
                .as_ref()
                .map(|t| Generator::new(t.to_owned()))
                .and_then(|g| g.ok());
            items.push((
                record.account.clone().unwrap_or_default(),
                generator,
                record.id,
            ));
        }
        items.sort_by(|a, b| a.0.cmp(&b.0));
        self.items = items;
        self.records = records;
        Ok(())
    }
}
