use crate::db::encryption::Encryption;
use crate::db::models::secure_record::SecureRecord;
use crate::{Token, TotpError};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

pub type AccountName = String;

// TODO: Remove Serialize and Deserialize
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: u32,
    pub account: Option<AccountName>,
    pub user: Option<String>,
    pub token: Option<Token>,
    pub password: Option<String>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Display for Record {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.token
                .as_ref()
                .map(|t| t.to_string())
                .unwrap_or_else(|| "N/A".to_string())
        )
    }
}

impl Default for Record {
    fn default() -> Self {
        Self {
            id: 0,
            account: None,
            user: None,
            token: None,
            password: None,
            note: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Record {
    pub fn from_secure_record(
        secure_record: &SecureRecord,
        encryption: &Encryption,
        password: &str,
    ) -> Result<Record, TotpError> {
        Ok(Record {
            id: secure_record.id,
            account: decrypt_record_field(secure_record.account.as_ref(), password, encryption),
            user: decrypt_record_field(secure_record.user.as_ref(), password, encryption),
            token: decrypt_record_field(secure_record.token.as_ref(), password, encryption)
                .map(|t| serde_json::from_str(&t))
                .and_then(|t| t.ok()),
            password: decrypt_record_field(secure_record.password.as_ref(), password, encryption),
            note: decrypt_record_field(secure_record.note.as_ref(), password, encryption),
            created_at: secure_record.created_at,
            updated_at: secure_record.updated_at,
        })
    }

    pub fn to_secure_record(
        &self,
        encryption: &Encryption,
        password: &str,
    ) -> Result<SecureRecord, TotpError> {
        Ok(SecureRecord {
            id: self.id,
            account: encrypt_record_field(self.account.as_ref(), password, encryption),
            user: encrypt_record_field(self.user.as_ref(), password, encryption),
            token: encrypt_record_field(
                self.token
                    .as_ref()
                    .map(serde_json::to_string)
                    .and_then(|r| r.ok())
                    .as_ref(),
                password,
                encryption,
            ),
            password: encrypt_record_field(self.password.as_ref(), password, encryption),
            note: encrypt_record_field(self.note.as_ref(), password, encryption),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}
fn decrypt_record_field<T: Display>(
    field: Option<&T>,
    password: &str,
    encryption: &Encryption,
) -> Option<String> {
    field
        .map(|value| {
            let mut parts = value
                .to_string()
                .splitn(2, ':')
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .into_iter();
            let content = parts.next().unwrap_or_default();
            let iv = parts.next().unwrap_or_default();
            (content, iv)
        })
        .map(|(content, iv)| encryption.decrypt(&content, password, &iv))
        .and_then(|res| res.ok())
}

fn encrypt_record_field<T: Display>(
    field: Option<&T>,
    password: &str,
    encryption: &Encryption,
) -> Option<String> {
    field
        .map(|value| encryption.encrypt(&value.to_string(), password))
        .and_then(|res| res.ok())
        .map(|(encrypted_content, iv)| format!("{}:{}", encrypted_content, iv))
}
