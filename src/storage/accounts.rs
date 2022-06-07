use crate::storage::encryption::Encryption;
use data_encoding::BASE32;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::hash_map::Iter;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs;
use std::str::FromStr;

use crate::errors::TotpError;

pub type AccountName = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Token(Vec<u8>);

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let encoded = BASE32.encode(self.0.as_slice());
        write!(f, "{}", encoded)
    }
}

impl AsRef<[u8]> for Token {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl TryFrom<String> for Token {
    type Error = TotpError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let bytes = BASE32.decode(value.trim_end().to_uppercase().as_bytes())?;
        Ok(Token(bytes))
    }
}

impl FromStr for Token {
    type Err = TotpError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Token(
            BASE32.decode(value.trim_end().to_uppercase().as_bytes())?,
        ))
    }
}

#[derive(Serialize, Deserialize)]
pub struct Contents {
    pub accounts: HashMap<AccountName, Token>,
}

#[derive(Clone)]
pub struct Storage {
    pub accounts: HashMap<AccountName, Token>,
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
        storage.load_file()?;
        Ok(storage)
    }

    #[allow(dead_code)]
    pub fn get_account_token(&self, account: AccountName) -> Result<Token, TotpError> {
        if !self.accounts.contains_key(&account) {
            return Err(TotpError::AccountNotFound(account));
        }
        self.accounts
            .get(&account)
            .cloned()
            .ok_or(TotpError::AccountNotFound(account))
    }

    pub fn add_account(&mut self, account: AccountName, token: Token) -> Result<(), TotpError> {
        let _ = self.accounts.insert(account.trim().into(), token);
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
        for (account, token) in &self.accounts {
            contents.push_str(&format!("{}|{}\n", account, token));
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

    pub fn load_file(&mut self) -> Result<(), TotpError> {
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

    pub fn to_iter(&self) -> Iter<AccountName, Token> {
        self.accounts.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::accounts::{Storage, Token};
    use std::str::FromStr;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn rand() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .subsec_nanos()
    }

    fn get_filename() -> String {
        format!(".test.storage.{}.txt", rand())
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
        std::fs::remove_file(filename).unwrap();
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
        std::fs::remove_file(filename).unwrap();
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
        let token = storage.get_account_token("Account2".into()).unwrap();
        assert_eq!(token.to_string(), "KRSXG5A=".to_string());
        let token = storage.get_account_token("Account3".into());
        assert!(token.is_err());

        std::fs::remove_file(filename).unwrap();
    }
}
