use crate::Token;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SecureData {
    pub token: Option<Token>,
    pub password: Option<String>,
    pub note: Option<String>,
}

impl Display for SecureData {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}-{}-{}",
            self.token
                .as_ref()
                .map(|t| format!("{}", t))
                .unwrap_or("".to_string()),
            self.password.as_ref().unwrap_or(&"".to_string()),
            self.note.as_ref().unwrap_or(&"".to_string())
        )
    }
}
impl SecureData {}
