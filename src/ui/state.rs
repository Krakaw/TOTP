use crate::db::models::record::AccountName;
use crate::ui::widgets::popup::Popup;
use crate::{Generator, Record, StorageTrait, TotpError};
use tui::widgets::Widget;

pub type TotpAccountName = String;
pub type TotpCode = String;
type ExpirySeconds = u64;
type RecordId = u32;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    FilterList,
    EditDetail,
}

impl Default for InputMode {
    fn default() -> Self {
        InputMode::Normal
    }
}

#[derive(PartialEq)]
pub enum ActivePane {
    OtpTable,
    DetailView,
}

impl Default for ActivePane {
    fn default() -> Self {
        ActivePane::OtpTable
    }
}
pub struct State<W: Widget + ?Sized> {
    pub input_mode: InputMode,
    pub active_pane: ActivePane,
    pub detail_input: String,
    pub filter_input: String,
    pub items: Vec<(AccountName, Option<Generator>, RecordId)>,
    pub records: Vec<Record>,
    pub display_otps: Vec<(TotpAccountName, TotpCode, ExpirySeconds, RecordId)>,
    pub running: bool,
    pub show_popup: Option<Popup<W>>,
}

impl<W: Widget> Default for State<W> {
    fn default() -> Self {
        Self {
            input_mode: InputMode::default(),
            active_pane: ActivePane::default(),
            detail_input: String::new(),
            filter_input: String::new(),
            items: vec![],
            records: vec![],
            display_otps: vec![],
            running: true,
            show_popup: None,
        }
    }
}

impl<W: Widget> State<W> {
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
