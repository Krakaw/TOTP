use totp_rs::{Algorithm, TOTP};
use crate::errors::TotpError;
use crate::Token;

pub struct Generator<'a> {
    totp: TOTP<&'a Token>,
}

impl<'a> Generator<'a> {
    pub fn new(token: &'a Token) -> Result<Self, TotpError> {
        let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, token, None, "".to_string())?;
        Ok(Self {
            totp
        })
    }

    pub fn generate(&self, time: Option<u64>) -> Result<String, TotpError> {
        let time = time.unwrap_or(chrono::Local::now().timestamp() as u64);
        Ok(self.totp.generate(time))
    }

    pub fn check(&self, code: String, time: Option<u64>) -> bool {
        let time = time.unwrap_or(chrono::Utc::now().timestamp() as u64);
        self.totp.check(&code, time)
    }
}
