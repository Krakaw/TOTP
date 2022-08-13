use crate::db::encryption::Encryption;
use crate::db::models::secure_record::SecureRecord;
use crate::db::storage::StorageTrait;
use crate::db::Connection;
use crate::storage::accounts::AccountName;
use crate::{Db, Record, TotpError};
use r2d2_sqlite::rusqlite::params;
use std::collections::HashMap;

pub struct SqliteStorage {
    pub db: Db,
    secure_records: Vec<SecureRecord>,
}

impl SqliteStorage {
    pub fn new(db: Db) -> Self {
        Self {
            db,
            secure_records: vec![],
        }
    }

    pub fn connection(&self) -> Result<Connection, TotpError> {
        Connection::try_from(&self.db)
    }
}

impl StorageTrait for SqliteStorage {
    fn search_account(&self, account_search: String) -> Result<(AccountName, Record), TotpError> {
        todo!()
    }

    fn search_accounts(
        &self,
        account_search: String,
    ) -> Result<Vec<(AccountName, Record)>, TotpError> {
        todo!()
    }

    fn add_account(&self, record: Record) -> Result<(), TotpError> {
        let secure_record = record.to_secure_record(&Encryption::default(), self.db.password())?;
        const SQL: &str = r#"
        INSERT INTO secure_records
            (account, user, token, password, note, created_at, updated_at)
            VALUES
            (
             ?1,
             ?2,
             ?3,
             ?4,
             ?5,
             strftime('%s','now'),
             strftime('%s','now')
            );
        "#;
        let conn = self.connection()?;
        let mut stmt = conn.prepare(SQL)?;
        stmt.execute(params![
            secure_record.account,
            secure_record.user,
            secure_record.token,
            secure_record.password,
            secure_record.note,
        ])?;
        Ok(())
    }

    fn remove_account(&self, account_or_id: String) -> Result<(), TotpError> {
        if let Ok(id) = account_or_id.parse::<u32>() {
            return self.remove_account_by_id(id);
        }
        self.remove_account_by_name(account_or_id)
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
            let record = Record::from_secure_record(rec, &encryption, self.db.password())?;
            let account_name = record.account.clone().unwrap_or_default();

            accounts.insert(account_name, record);
        }
        Ok(accounts)
    }

    fn password(&self) -> &str {
        self.db.password()
    }
    fn load(&mut self) -> Result<(), TotpError> {
        let conn = Connection::try_from(&self.db)?;
        self.secure_records = SecureRecord::all(&conn)?;
        Ok(())
    }
}
