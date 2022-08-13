use data_encoding::DecodeError;
use openssl::error::ErrorStack;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::RecvError;
use totp_rs::TotpUrlError;

#[derive(Debug)]
pub enum TotpError {
    AccountNotFound(String),
    Base32Decode(String),
    Clap(String),
    TotpUrl(String),
    Format(String),
    Encryption(String),
    Decryption(String),
    FileIO(String),
    Utf8(String),
    StdIO(String),
    InvalidOtpForRange,
    Ui(String),
    UiEvent(String),
    Json(String),
    HttpServer(String),
    R2d2(String),
    Migration(String),
}

impl Error for TotpError {}

impl Display for TotpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOTP Error: {:?}", self)
    }
}

impl From<r2d2::Error> for TotpError {
    fn from(e: r2d2::Error) -> Self {
        TotpError::R2d2(e.to_string())
    }
}
impl From<r2d2_sqlite::rusqlite::Error> for TotpError {
    fn from(e: r2d2_sqlite::rusqlite::Error) -> Self {
        TotpError::R2d2(e.to_string())
    }
}

impl From<rusqlite_migration::Error> for TotpError {
    fn from(e: rusqlite_migration::Error) -> Self {
        TotpError::Migration(e.to_string())
    }
}

impl From<DecodeError> for TotpError {
    fn from(e: DecodeError) -> Self {
        TotpError::Base32Decode(e.to_string())
    }
}

impl From<TotpUrlError> for TotpError {
    fn from(e: TotpUrlError) -> Self {
        TotpError::TotpUrl(format!("{:?}", e))
    }
}

impl From<clap::Error> for TotpError {
    fn from(e: clap::Error) -> Self {
        TotpError::Clap(e.to_string())
    }
}

impl From<std::fmt::Error> for TotpError {
    fn from(e: std::fmt::Error) -> Self {
        TotpError::Format(e.to_string())
    }
}
impl From<ErrorStack> for TotpError {
    fn from(e: ErrorStack) -> Self {
        TotpError::Encryption(e.to_string())
    }
}

impl From<RecvError> for TotpError {
    fn from(e: RecvError) -> Self {
        TotpError::UiEvent(e.to_string())
    }
}

impl From<std::io::Error> for TotpError {
    fn from(e: std::io::Error) -> Self {
        TotpError::Ui(e.to_string())
    }
}

impl From<serde_json::Error> for TotpError {
    fn from(e: serde_json::Error) -> Self {
        TotpError::Json(e.to_string())
    }
}
