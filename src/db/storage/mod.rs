use crate::storage::accounts::AccountName;
use crate::{Record, TotpError};
use std::collections::HashMap;

pub mod sqlite;

pub trait StorageTrait {
    fn search_account(&self, account_search: String) -> Result<(AccountName, Record), TotpError>;
    fn search_accounts(
        &self,
        account_search: String,
    ) -> Result<Vec<(AccountName, Record)>, TotpError>;
    fn add_account(&self, record: Record) -> Result<(), TotpError>;
    fn remove_account(&self, account_or_id: String) -> Result<(), TotpError>;
    fn remove_account_by_name(&self, account: AccountName) -> Result<(), TotpError>;
    fn remove_account_by_id(&self, id: u32) -> Result<(), TotpError>;
    fn accounts(&self) -> Result<HashMap<AccountName, Record>, TotpError>;
    fn load(&mut self) -> Result<(), TotpError>;
    fn password(&self) -> &str;
}