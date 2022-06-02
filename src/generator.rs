use crate::errors::TotpError;
use crate::Token;
use totp_rs::{Algorithm, TOTP};

pub struct Generator<'a> {
    totp: TOTP<&'a Token>,
}

impl<'a> Generator<'a> {
    pub fn new(token: &'a Token) -> Result<Self, TotpError> {
        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, token, None, "".to_string())?;
        Ok(Self { totp })
    }

    pub fn generate(&self, time: Option<u64>) -> Result<(String, u64), TotpError> {
        let time = time.unwrap_or(chrono::Local::now().timestamp() as u64);
        let offset = time % 30;
        let rounded_up = (time - offset + 30) - time;
        Ok((self.totp.generate(time), rounded_up))
    }

    pub fn check(&self, code: String, time: Option<u64>) -> bool {
        let time = time.unwrap_or(chrono::Utc::now().timestamp() as u64);
        self.totp.check(&code, time)
    }
}
