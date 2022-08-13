use crate::db::models::secure_record::SecureRecord;
use crate::db::Connection;
use crate::storage::accounts::AccountName;
use crate::storage::encryption::Encryption;
use crate::storage::StorageTrait;
use crate::{Db, Record, TotpError};
use std::collections::HashMap;
use TryInto;

pub struct Storage {
    pub db: Db,
    secure_records: Vec<SecureRecord>,
    password: String,
}

impl StorageTrait for Storage {
    fn search_account(&self, account_search: String) -> Result<(AccountName, Record), TotpError> {
        todo!()
    }

    fn search_accounts(
        &self,
        account_search: String,
    ) -> Result<Vec<(AccountName, Record)>, TotpError> {
        todo!()
    }

    fn add_account(&self, account: AccountName, record: Record) -> Result<(), TotpError> {
        todo!()
    }

    fn remove_account_by_name(&self, account: AccountName) -> Result<(), TotpError> {
        todo!()
    }

    fn remove_account_by_id(&self, id: u32) -> Result<(), TotpError> {
        todo!()
    }

    fn accounts(&self) -> Result<HashMap<AccountName, Record>, TotpError> {
        let mut accounts = HashMap::new();
        let encryption = Encryption::default();
        for rec in self.secure_records.iter() {
            let account_name = rec.account.clone().unwrap_or_default();
            let record = Record::from_secure_record(rec, &encryption)?;
            accounts.insert(account_name, record);
        }
        Ok(accounts)
    }

    fn load(&mut self) -> Result<(), TotpError> {
        let conn = Connection::try_from(&self.db)?;
        self.secure_records = SecureRecord::all(&conn)?;
        Ok(())
    }
}
