#![allow(dead_code)]
#![allow(clippy::large_enum_variant)]
use std::sync::Arc;

use crate::TotpError;
use r2d2::{Pool, PooledConnection};
use r2d2_sqlite::rusqlite::{Statement, Transaction};
use r2d2_sqlite::SqliteConnectionManager;

pub mod encryption;
mod migrations;
pub mod models;
pub mod storage;

#[derive(Debug, Clone)]
pub struct Db {
    pub pool: Arc<Pool<SqliteConnectionManager>>,
    password: String,
}

impl Db {
    pub fn new(password: String, connection_string: Option<String>) -> Result<Self, TotpError> {
        let sqlite_connection_manager = if let Some(connection_string) = connection_string {
            SqliteConnectionManager::file(connection_string)
        } else {
            SqliteConnectionManager::memory()
        };

        let sqlite_pool = Pool::new(sqlite_connection_manager)?;
        let pool = Arc::new(sqlite_pool);
        Ok(Db { pool, password })
    }

    pub fn init(&self) -> Result<(), TotpError> {
        let mut connection = self.pool.get()?;
        let migrations = migrations::migrations();
        migrations.to_latest(&mut connection)?;
        Ok(())
    }

    pub fn password(&self) -> &str {
        self.password.as_str()
    }
}

pub enum Connection<'a> {
    Pooled(PooledConnection<SqliteConnectionManager>),
    Transaction(Transaction<'a>),
}

impl<'a> TryFrom<&Db> for Connection<'a> {
    type Error = TotpError;

    fn try_from(db: &Db) -> Result<Self, Self::Error> {
        Ok(Self::Pooled(db.pool.get()?))
    }
}

impl<'a> From<Transaction<'a>> for Connection<'a> {
    fn from(transaction: Transaction<'a>) -> Self {
        Self::Transaction(transaction)
    }
}

impl<'a> Connection<'a> {
    #[inline]
    pub fn prepare(&self, query: &str) -> Result<Statement, TotpError> {
        match self {
            Connection::Pooled(client) => Ok(client.prepare(query)?),
            Connection::Transaction(transaction) => Ok(transaction.prepare(query)?),
        }
    }

    #[inline]
    pub fn last_insert_rowid(&self) -> i64 {
        match self {
            Connection::Pooled(client) => client.last_insert_rowid(),
            Connection::Transaction(transaction) => transaction.last_insert_rowid(),
        }
    }

    pub fn transaction(self) -> Result<Transaction<'a>, TotpError> {
        match self {
            Connection::Pooled(_) => Err(TotpError::R2d2("Not a transaction".to_string())),
            Connection::Transaction(t) => Ok(t),
        }
    }
}
