use crate::db::models::secure_record::SecureRecord;
use crate::storage::accounts::AccountName;
use crate::storage::encryption::Encryption;
use crate::{Token, TotpError};
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

// TODO: Remove Serialize and Deserialize
#[derive(Clone, Serialize, Deserialize)]
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
                .unwrap_or("N/A".to_string())
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
    ) -> Result<Record, TotpError> {
        Ok(Record::default())
    }
}
