use data_encoding::DecodeError;
use openssl::error::ErrorStack;
use std::error::Error;
use std::fmt::{Display, Formatter};
use totp_rs::TotpUrlError;

#[derive(Debug)]
pub enum TotpError {
    Base32Decode(String),
    Clap(String),
    TotpUrl(String),
    Encryption(String),
    Decryption(String),
    FileIO(String),
    Utf8(String),
}

impl Error for TotpError {}

impl Display for TotpError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TOTP Error: {:?}", self)
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

impl From<ErrorStack> for TotpError {
    fn from(e: ErrorStack) -> Self {
        TotpError::Encryption(e.to_string())
    }
}
