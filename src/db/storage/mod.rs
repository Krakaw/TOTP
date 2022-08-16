use crate::db::models::record::AccountName;
use crate::{Record, TotpError};

pub mod sqlite;

pub trait StorageTrait {
    fn get_account(&self, id: u32) -> Result<Record, TotpError>;
    fn search_account(&self, account_search: &str) -> Result<Record, TotpError> {
        self.search_accounts(account_search)?
            .first()
            .cloned()
            .ok_or_else(|| TotpError::AccountNotFound(account_search.to_string()))
    }
    fn search_accounts(&self, account_search: &str) -> Result<Vec<Record>, TotpError>;
    fn add_account(&mut self, record: Record) -> Result<(), TotpError>;
    fn edit_account(&mut self, record: Record) -> Result<(), TotpError>;
    fn remove_account(&mut self, account_or_id: String) -> Result<(), TotpError>;
    fn remove_account_by_name(&mut self, account: AccountName) -> Result<(), TotpError>;
    fn remove_account_by_id(&mut self, id: u32) -> Result<(), TotpError>;
    fn accounts(&self) -> Result<Vec<Record>, TotpError>;
    fn load(&mut self) -> Result<(), TotpError>;
    fn password(&self) -> &str;
}
