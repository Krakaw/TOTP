use crate::db::encryption::Encryption;
use crate::db::models::record::{decrypt_record_field, encrypt_record_field, AccountName};
use crate::db::models::secure_record::SecureRecord;
use crate::db::storage::StorageTrait;
use crate::db::Connection;
use crate::{Db, Record, TotpError};
use r2d2_sqlite::rusqlite::params;

pub struct SqliteStorage {
    pub db: Db,
    secure_records: Vec<SecureRecord>,
    encryption: Encryption,
}

impl SqliteStorage {
    pub fn new(db: Db, encryption: Encryption) -> Self {
        Self {
            db,
            secure_records: vec![],
            encryption,
        }
    }
}

impl StorageTrait for SqliteStorage {
    fn get_account(&self, id: u32) -> Result<Record, TotpError> {
        if let Some(record) = self.accounts()?.iter().find(|r| r.id == id) {
            return Ok(record.clone());
        }
        Err(TotpError::AccountNotFound(format!("id {} not found", id)))
    }

    fn search_accounts(&self, account_search: &str) -> Result<Vec<Record>, TotpError> {
        let accounts = self.accounts()?;
        let mut accounts = accounts
            .iter()
            .filter(|record| {
                record
                    .account
                    .clone()
                    .unwrap_or_default()
                    .to_lowercase()
                    .contains(&account_search.to_lowercase())
            })
            .clone()
            .collect::<Vec<&Record>>();
        accounts.sort_by(|a, b| a.account.cmp(&b.account));

        Ok(accounts.into_iter().cloned().collect::<Vec<_>>())
    }

    fn add_account(&mut self, record: Record) -> Result<(), TotpError> {
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
        let conn = Connection::try_from(&self.db)?;
        let mut stmt = conn.prepare(SQL)?;
        stmt.execute(params![
            secure_record.account,
            secure_record.user,
            secure_record.token,
            secure_record.password,
            secure_record.note,
        ])?;
        self.load()?;
        Ok(())
    }

    fn edit_account(&mut self, record: Record) -> Result<(), TotpError> {
        let secure_record = record.to_secure_record(&Encryption::default(), self.db.password())?;
        const SQL: &str = r#"
        UPDATE secure_records SET account = ?1, user = ?2, password = ?3, note = ?4, token = ?5, updated_at = strftime('%s','now')
            WHERE id = ?6;
        "#;
        let conn = Connection::try_from(&self.db)?;
        let mut stmt = conn.prepare(SQL)?;
        stmt.execute(params![
            secure_record.account,
            secure_record.user,
            secure_record.password,
            secure_record.note,
            secure_record.token,
            record.id
        ])?;
        self.load()?;
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

    fn accounts(&self) -> Result<Vec<Record>, TotpError> {
        let mut accounts = vec![];
        let encryption = Encryption::default();
        for rec in self.secure_records.iter() {
            let record = Record::from_secure_record(rec, &encryption, self.db.password())?;
            accounts.push(record);
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

    fn get_encryption(&self) -> &Encryption {
        &self.encryption
    }

    fn set_lock_encryption(&self) -> Result<(), TotpError> {
        const DELETE_SQL: &str = "DELETE FROM table_lock WHERE 1=1;";
        let conn = Connection::try_from(&self.db)?;
        let mut stmt = conn.prepare(DELETE_SQL)?;
        stmt.execute(params![])?;
        const INSERT_SQL: &str = "INSERT INTO table_lock (key) VALUES (?1);";
        let mut stmt = conn.prepare(INSERT_SQL)?;
        let encryption = self.get_encryption();
        stmt.execute(params![encrypt_record_field(
            Some(&encryption.key),
            self.password(),
            encryption
        )])?;
        Ok(())
    }

    fn verify_lock_encryption(&self) -> Result<(), TotpError> {
        const SQL: &str = "SELECT key FROM table_lock";
        let conn = Connection::try_from(&self.db)?;
        let mut stmt = conn.prepare(SQL)?;
        let mut rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        if let Some(result) = rows.next() {
            let encryption = self.get_encryption();
            let value = result
                .map_err(|_e| TotpError::Decryption("Missing lock key, aborting.".to_string()))?;
            let key = decrypt_record_field(Some(&value), self.password(), encryption)?;
            if key == Some(encryption.key.clone()) {
                return Ok(());
            }
        } else {
            // Key is missing
            return Err(TotpError::MissingLockKey);
        }

        Err(TotpError::Decryption(
            "Lock key is invalid, aborting.".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Token;
    use std::str::FromStr;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn rand() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }

    fn get_storage(encryption: Option<Encryption>) -> SqliteStorage {
        let db = Db::new(
            "password".to_string(),
            Some(format!("file:memdb{}?mode=memory&cache=shared", rand())),
        )
        .unwrap();
        db.init().unwrap();
        SqliteStorage::new(db, encryption.unwrap_or_default())
    }

    #[test]
    fn add_account() {
        let mut storage = get_storage(None);
        assert_eq!(storage.accounts().unwrap().iter().len(), 0);
        storage
            .add_account(Record {
                account: Some("Account1".to_string()),
                token: Some(Token::from_str("JBSWY3DPEHPK3PXP").unwrap()),
                ..Record::default()
            })
            .unwrap();
        assert_eq!(storage.accounts().unwrap().iter().len(), 1);
    }

    #[test]
    fn delete_account() {
        let mut storage = get_storage(None);
        assert_eq!(storage.accounts().unwrap().iter().len(), 0);
        storage
            .add_account(Record {
                account: Some("Account1".to_string()),
                token: Some(Token::from_str("JBSWY3DPEHPK3PXP").unwrap()),
                ..Record::default()
            })
            .unwrap();
        assert_eq!(storage.accounts().unwrap().iter().len(), 1);
        storage
            .add_account(Record {
                account: Some("Account2".to_string()),
                token: Some(Token::from_str("JBSWY3DPEHPK3PXP").unwrap()),
                ..Record::default()
            })
            .unwrap();
        assert_eq!(storage.accounts().unwrap().iter().len(), 2);
        storage.remove_account("Account2".to_string()).unwrap();
        assert_eq!(storage.accounts().unwrap().iter().len(), 1);
        storage.remove_account("1".to_string()).unwrap();
        assert_eq!(storage.accounts().unwrap().iter().len(), 0);
    }

    #[test]
    fn get_account_token() {
        let mut storage = get_storage(None);
        storage
            .add_account(Record {
                account: Some("Account1".to_string()),
                token: Some(Token::from_str("JBSWY3DPEHPK3PXP").unwrap()),
                ..Record::default()
            })
            .unwrap();
        storage
            .add_account(Record {
                account: Some("Account2".to_string()),
                token: Some(Token::from_str("KRSXG5A=").unwrap()),
                ..Record::default()
            })
            .unwrap();

        let record = storage.search_account("Account2").unwrap();
        assert_eq!(record.token.unwrap().to_string(), "KRSXG5A=".to_string());
        let token = storage.search_account("Account3");
        assert!(token.is_err());
    }

    #[test]
    fn table_lock_keys() {
        let db_path = format!("file:memdb{}?mode=memory&cache=shared", rand());
        let db = Db::new("password".to_string(), Some(db_path.clone())).unwrap();
        db.init().unwrap();
        let storage = SqliteStorage::new(
            db,
            Encryption {
                key: "SomeOtherKey".to_string(),
                value: "SomeOtherValue".to_string(),
            },
        );

        // Cannot verify lock encryption without first setting it.
        let missing_error = storage.verify_lock_encryption();
        assert!(matches!(missing_error, Err(TotpError::MissingLockKey)));

        // Set lock encryption
        assert!(storage.set_lock_encryption().is_ok());
        // Now verify lock encryption is value
        assert!(storage.verify_lock_encryption().is_ok());

        let db = Db::new("password".to_string(), Some(db_path.clone())).unwrap();
        let mut storage = SqliteStorage::new(
            db,
            Encryption {
                key: "SomeOtherKey".to_string(),
                value: "SomeOtherValue".to_string(),
            },
        );
        // Reconnecting will still verify correctly.
        assert!(storage.verify_lock_encryption().is_ok());

        // Now check the wrong password fails
        let db = Db::new("wrong_password".to_string(), Some(db_path.clone())).unwrap();
        let mut storage = SqliteStorage::new(
            db,
            Encryption {
                key: "SomeOtherKey".to_string(),
                value: "SomeOtherValue".to_string(),
            },
        );
        // Using the wrong password will fail to validate
        let error_result = storage.verify_lock_encryption();
        assert!(error_result.is_err());

        // Now check the wrong key fails
        let db = Db::new("password".to_string(), Some(db_path)).unwrap();
        let mut storage = SqliteStorage::new(
            db,
            Encryption {
                key: "WrongKey".to_string(),
                value: "SomeOtherValue".to_string(),
            },
        );
        // Using the wrong password will fail to validate
        let error_result = storage.verify_lock_encryption();
        assert!(error_result.is_err());
    }
    // #[test]
    // fn get_account_token() {
    //     let filename = get_filename();
    //     let mut storage = Storage::new("password".to_string(), Some(filename.clone())).unwrap();
    //     storage
    //         .add_account(
    //             "Account1".to_string(),
    //             Token::from_str("JBSWY3DPEHPK3PXP").unwrap(),
    //         )
    //         .unwrap();
    //     storage
    //         .add_account("Account2".to_string(), Token::from_str("KRSXG5A=").unwrap())
    //         .unwrap();
    //     let (_, token) = storage.search_account("Account2".into()).unwrap();
    //     assert_eq!(token.to_string(), "KRSXG5A=".to_string());
    //     let token = storage.search_account("Account3".into());
    //     assert!(token.is_err());
    //
    //     let _ = std::fs::remove_file(filename);
    // }
}
