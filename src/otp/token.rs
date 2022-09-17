use crate::TotpError;
use data_encoding::BASE32;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Token {
    pub secret: Vec<u8>,
    pub digits: usize,
    pub skew: u8,
    pub step: u64,
}

impl Default for Token {
    fn default() -> Self {
        Self {
            secret: vec![],
            digits: 6,
            skew: 1,
            step: 30,
        }
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let encoded = BASE32.encode(self.secret.as_slice());
        write!(f, "{}", encoded)
    }
}

impl AsRef<[u8]> for Token {
    fn as_ref(&self) -> &[u8] {
        self.secret.as_ref()
    }
}

impl TryFrom<String> for Token {
    type Error = TotpError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let secret = BASE32.decode(value.trim_end().to_uppercase().as_bytes())?;
        Ok(Token {
            secret,
            ..Token::default()
        })
    }
}

impl FromStr for Token {
    type Err = TotpError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let secret = BASE32.decode(value.trim_end().to_uppercase().as_bytes())?;
        Ok(Token {
            secret,
            ..Token::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tokens_ignore_case() {
        let token_string = "jbswy3dpehpk3pxp";
        let token: Token = token_string.parse().unwrap();
        assert_eq!(token, Token::from_str("JBSWY3DPEHPK3PXP").unwrap());
    }
}
