use crate::storage::accounts::AccountName;
use crate::Token;
use chrono::NaiveDateTime;
use r2d2_sqlite::rusqlite::{params, Row};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

type EncryptedString = String;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecureRecord {
    pub id: u32,
    pub account: Option<AccountName>,
    pub user: Option<EncryptedString>,
    pub token: Option<EncryptedString>,
    pub password: Option<EncryptedString>,
    pub note: Option<EncryptedString>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl<'stmt> From<&Row<'stmt>> for SecureRecord {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get(0).unwrap(),
            account: row.get(1).unwrap_or(None),
            user: row.get(2).unwrap_or(None),
            token: row.get(3).unwrap_or(None),
            password: row.get(4).unwrap_or(None),
            note: row.get(5).unwrap_or(None),
            created_at: NaiveDateTime::from_timestamp(row.get(6).unwrap(), 0),
            updated_at: NaiveDateTime::from_timestamp(row.get(7).unwrap(), 0),
        }
    }
}
