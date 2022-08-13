use crate::db::encryption::Encryption;

use crate::errors::TotpError;
use crate::{Record, Token};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;

pub type AccountName = String;

#[derive(Serialize, Deserialize)]
pub struct Contents {
    pub accounts: HashMap<AccountName, Record>,
}

#[derive(Clone)]
pub struct Storage {
    pub accounts: HashMap<AccountName, Record>,
    password: String,
    filename: String,
}

impl Default for Storage {
    fn default() -> Self {
        Self {
            accounts: HashMap::new(),
            password: String::new(),
            filename: ".storage.txt".to_string(),
        }
    }
}

impl Storage {
    pub fn new(password: String, filename: Option<String>) -> Result<Self, TotpError> {
        let mut storage = Self {
            accounts: HashMap::new(),
            password,
            filename: filename.unwrap_or_else(|| ".storage.txt".to_string()),
        };
        storage.load()?;
        Ok(storage)
    }

    pub fn search_account(&self, account_search: &str) -> Result<(AccountName, Record), TotpError> {
        self.search_accounts(account_search)?
            .first()
            .cloned()
            .ok_or(TotpError::AccountNotFound(account_search.to_string()))
    }

    pub fn search_accounts(
        &self,
        account_search: &str,
    ) -> Result<Vec<(AccountName, Record)>, TotpError> {
        let mut accounts = self
            .accounts
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
            .map(|(a, t)| (a.clone(), t.clone()))
            .collect::<Vec<_>>())
    }

    pub fn add_account(
        &mut self,
        account: AccountName,
        secure_data: Record,
    ) -> Result<(), TotpError> {
        let account_name: AccountName = account.trim().to_string();
        let _ = self.accounts.insert(account_name, secure_data);
        self.save_file()
    }

    pub fn remove_account(&mut self, account: AccountName) -> Result<(), TotpError> {
        if !self.accounts.contains_key(&account) {
            return Err(TotpError::AccountNotFound(account));
        }

        let _ = self.accounts.remove(&account);
        self.save_file()
    }

    pub fn save_file(&self) -> Result<(), TotpError> {
        let mut contents = String::new();
        for (account, secure_data) in &self.accounts {
            writeln!(contents, "{}|{}", account, secure_data)?;
        }
        let contents = Contents {
            accounts: self.accounts.clone(),
        };
        let contents = json!(contents).to_string();
        let encryption = Encryption::default();
        let (encrypted_content, iv) = encryption.encrypt(&contents, &self.password)?;
        fs::write(&self.filename, format!("{}:{}\n", encrypted_content, iv))
            .map_err(|e| TotpError::FileIO(e.to_string()))?;
        Ok(())
    }

    pub fn load(&mut self) -> Result<(), TotpError> {
        if let Ok(file_contents) = fs::read_to_string(&self.filename) {
            let file_contents: Vec<&str> = file_contents.split(':').collect();
            let content = file_contents[0];
            let iv = file_contents[1].trim();
            let encryption = Encryption::default();
            let file_contents = encryption.decrypt(content, &self.password, iv)?;
            let accounts: Contents = serde_json::from_str(&file_contents)?;
            self.accounts = accounts.accounts;
        };

        Ok(())
    }

    pub fn to_iter(&self) -> Iter<AccountName, Record> {
        self.accounts.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::accounts::{Storage, Token};
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
