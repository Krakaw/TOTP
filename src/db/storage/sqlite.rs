use crate::db::encryption::Encryption;
use crate::db::models::record::AccountName;
use crate::db::models::secure_record::SecureRecord;
use crate::db::storage::StorageTrait;
use crate::db::Connection;
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
    fn search_accounts(&self, account_search: &str) -> Result<Vec<Record>, TotpError> {
        let accounts = self.accounts()?;
        let mut accounts = accounts
            .iter()
            .filter(|(account_name, _secure_data)| {
                account_name
                    .to_lowercase()
                    .contains(&account_search.to_lowercase())
            })
            .clone()
            .collect::<Vec<(&AccountName, &Record)>>();
        accounts.sort_by(|a, b| a.0.cmp(b.0));

        Ok(accounts
            .into_iter()
            .map(|(_a, t)| t.clone())
            .collect::<Vec<_>>())
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

    fn remove_account(&mut self, account_or_id: String) -> Result<(), TotpError> {
        if let Ok(id) = account_or_id.parse::<u32>() {
            return self.remove_account_by_id(id);
        }
        self.remove_account_by_name(account_or_id)
    }
    fn remove_account_by_name(&mut self, account: AccountName) -> Result<(), TotpError> {
        let account = self.search_account(&account)?;
        self.remove_account_by_id(account.id)
    }

    fn remove_account_by_id(&mut self, id: u32) -> Result<(), TotpError> {
        const SQL: &str = "DELETE FROM secure_records WHERE id = ?1;";
        let conn = Connection::try_from(&self.db)?;
        let mut stmt = conn.prepare(SQL)?;
        stmt.execute(params![&id])?;
        self.load()
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

    fn load(&mut self) -> Result<(), TotpError> {
        let conn = Connection::try_from(&self.db)?;
        self.secure_records = SecureRecord::all(&conn)?;
        Ok(())
    }

    fn password(&self) -> &str {
        self.db.password()
    }
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::str::FromStr;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn rand() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }

    fn get_filename() -> String {
        let mut dir = temp_dir();
        dir.push(format!(".test.storage.{}.txt", rand()));
        dir.to_string_lossy().to_string()
    }

    #[test]
    fn tokens_ignore_case() {
        let token_string = "jbswy3dpehpk3pxp";
        let token: Token = token_string.parse().unwrap();
        assert_eq!(token, Token::from_str("JBSWY3DPEHPK3PXP").unwrap());
    }

    #[test]
    fn add_account() {
        let filename = get_filename();
        let mut storage = Storage::new("password".to_string(), Some(filename.clone())).unwrap();
        assert_eq!(storage.to_iter().len(), 0);
        storage
            .add_account(
                "Account1".to_string(),
                Token::from_str("JBSWY3DPEHPK3PXP").unwrap(),
            )
            .unwrap();
        assert_eq!(storage.to_iter().len(), 1);
        let storage = Storage::new("password".to_string(), Some(filename.clone())).unwrap();
        assert_eq!(storage.to_iter().len(), 1);
        let _ = std::fs::remove_file(filename);
    }

    #[test]
    fn delete_account() {
        let filename = get_filename();
        let mut storage = Storage::new("password".to_string(), Some(filename.clone())).unwrap();
        assert_eq!(storage.to_iter().len(), 0);
        storage
            .add_account(
                "Account1".to_string(),
                Token::from_str("JBSWY3DPEHPK3PXP").unwrap(),
            )
            .unwrap();
        assert_eq!(storage.to_iter().len(), 1);
        storage
            .add_account(
                "Account2".to_string(),
                Token::from_str("JBSWY3DPEHPK3PXP").unwrap(),
            )
            .unwrap();
        assert_eq!(storage.to_iter().len(), 2);
        storage.remove_account("Account1".into()).unwrap();
        assert_eq!(storage.to_iter().len(), 1);
        let _ = std::fs::remove_file(filename);
    }

    #[test]
    fn get_account_token() {
        let filename = get_filename();
        let mut storage = Storage::new("password".to_string(), Some(filename.clone())).unwrap();
        storage
            .add_account(
                "Account1".to_string(),
                Token::from_str("JBSWY3DPEHPK3PXP").unwrap(),
            )
            .unwrap();
        storage
            .add_account("Account2".to_string(), Token::from_str("KRSXG5A=").unwrap())
            .unwrap();
        let (_, token) = storage.search_account("Account2".into()).unwrap();
        assert_eq!(token.to_string(), "KRSXG5A=".to_string());
        let token = storage.search_account("Account3".into());
        assert!(token.is_err());

        let _ = std::fs::remove_file(filename);
    }
}
