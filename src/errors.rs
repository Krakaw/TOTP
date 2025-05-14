use data_encoding::DecodeError;
use openssl::error::ErrorStack;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::mpsc::RecvError;
use totp_rs::TotpUrlError;

#[derive(Debug)]
pub enum TotpError {
    #[allow(dead_code)]
    AccountNotFound(String),
    #[allow(dead_code)]
    Base32Decode(String),
    #[allow(dead_code)]
    Clap(String),
    #[allow(dead_code)]
    TotpUrl(String),
    #[allow(dead_code)]
    Format(String),
    #[allow(dead_code)]
    Encryption(String),
    #[allow(dead_code)]
    Decryption(String),
    MissingLockKey,
    #[allow(dead_code)]
    Utf8(String),
    InvalidOtpForRange,
    #[allow(dead_code)]
    Ui(String),
    #[allow(dead_code)]
    UiEvent(String),
    #[allow(dead_code)]
    Json(String),
    #[allow(dead_code)]
    HttpServer(String),
    #[allow(dead_code)]
    R2d2(String),
    #[allow(dead_code)]
    Migration(String),
    #[allow(dead_code)]
    Storage(String),
    #[allow(dead_code)]
    SecretParseError(String),
    #[allow(dead_code)]
    ClipboardError(String),
}

impl Error for TotpError {}

impl Display for TotpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOTP Error: {:?}", self)
    }
}
impl From<totp_rs::SecretParseError> for TotpError {
    fn from(e: totp_rs::SecretParseError) -> Self {
        TotpError::SecretParseError(format!("{:?}", e))
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
