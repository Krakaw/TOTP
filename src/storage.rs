use crate::encryption::Encryption;
use data_encoding::BASE32;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs;
use std::str::FromStr;

use crate::errors::TotpError;

pub type AccountName = String;

#[derive(Debug, Clone)]
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
        let bytes = BASE32.decode(value.as_bytes())?;
        Ok(Token(bytes))
    }
}

impl FromStr for Token {
    type Err = TotpError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Token(BASE32.decode(value.as_bytes())?))
    }
}

pub struct Storage {
    pub accounts: HashMap<AccountName, Token>,
    password: String,
    filename: String,
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

    pub fn add_account(&mut self, account: AccountName, token: Token) -> Result<(), TotpError> {
        let _ = self.accounts.insert(account, token);
        self.save_file()
    }

    pub fn save_file(&self) -> Result<(), TotpError> {
        let mut contents = String::new();
        for (account, token) in &self.accounts {
            contents.push_str(&format!("{}:{}\n", account, token));
        }
        let encryption = Encryption::default();
        let (encrypted_content, iv) = encryption.encrypt(&contents, &self.password)?;
        fs::write(&self.filename, format!("{}:{}\n", encrypted_content, iv))
            .map_err(|e| TotpError::FileIO(e.to_string()))?;
        Ok(())
    }

    pub fn load_file(&mut self) -> Result<(), TotpError> {
        let file_contents = fs::read_to_string(&self.filename).expect("Failed to read file");
        let file_contents: Vec<&str> = file_contents.split(':').collect();
        let content = file_contents[0];
        let iv = file_contents[1].trim();
        let encryption = Encryption::default();
        let file_contents = encryption.decrypt(content, &self.password, iv)?;
        for (account, token) in file_contents
            .split('\n')
            .filter(|l| l.trim() != "")
            .map(|line| {
                let parts = line.split(':').collect::<Vec<&str>>();
                (parts[0].to_string(), parts[1].to_string().parse::<Token>())
            })
        {
            self.accounts.insert(account, token?);
        }
        Ok(())
    }
}
