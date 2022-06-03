use crate::errors::TotpError;
use crate::Token;
use chrono::NaiveDateTime;
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

    pub fn check_range(
        &self,
        code: &str,
        time: u64,
        range_in_minutes: u64,
    ) -> Result<NaiveDateTime, TotpError> {
        let range = range_in_minutes * 60;
        let start = time - range;
        let end = time + range;
        let mut i = start;
        while i <= end {
            if self.check(code, Some(i)) {
                return Ok(NaiveDateTime::from_timestamp(i as i64, 0));
            }
            i += 30;
        }
        Err(TotpError::InvalidOtpForRange)
    }
    pub fn check(&self, code: &str, time: Option<u64>) -> bool {
        let time = time.unwrap_or(chrono::Utc::now().timestamp() as u64);
        self.totp.check(code, time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;
    use std::str::FromStr;

    #[test]
    fn generate() {
        let secret = Token::from_str("JBSWY3DPEHPK3PXP").unwrap();
        let generator = Generator::new(&secret).unwrap();
        let (token, _) = generator.generate(Some(1654258053)).unwrap();
        assert_eq!(token, "975361");
    }

    #[test]
    fn check_range() {
        let secret = Token::from_str("JBSWY3DPEHPK3PXP").unwrap();
        let generator = Generator::new(&secret).unwrap();
        let generated_time = NaiveDateTime::from_str("2022-06-03T08:00:00").unwrap();
        let (token, _) = generator
            .generate(Some(generated_time.timestamp() as u64))
            .unwrap();
        let time = generated_time.with_hour(7).unwrap();
        let invalid_range = generator.check_range(&token, time.timestamp() as u64, 10);
        assert!(invalid_range.is_err());
        let valid_range = generator
            .check_range(&token, time.timestamp() as u64, 61)
            .unwrap();
        assert_eq!(valid_range.timestamp(), generated_time.timestamp() - 30);
    }
}
