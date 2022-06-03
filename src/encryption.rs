use crate::TotpError;
use data_encoding::BASE64;
use openssl::rand::rand_bytes;
use openssl::symm::{decrypt, encrypt, Cipher};

pub struct Encryption {
    pub key: String,
    pub value: String,
}
impl Default for Encryption {
    fn default() -> Self {
        Self {
            key: "TOTP_KEY".to_string(),
            value: "TOTP_VALUE".to_string(),
        }
    }
}

impl Encryption {
    pub fn encrypt(&self, content: &str, password: &str) -> Result<(String, String), TotpError> {
        let cipher = Cipher::aes_256_cbc();
        let mut password = password.as_bytes().to_vec();
        while password.len() < cipher.key_len() {
            password.push(b'0');
        }
        let content = content.to_owned() + format!("\n{}:{}", self.key, self.value).as_str();
        let data = content.as_bytes();
        let key = password.as_slice();

        let iv = {
            let mut buf = vec![0; cipher.iv_len().unwrap_or(0)];
            rand_bytes(buf.as_mut_slice())?;
            buf
        };
        let encrypted_content = encrypt(cipher, key, Some(iv.as_slice()), data)?;
        Ok((
            BASE64.encode(encrypted_content.as_slice()),
            BASE64.encode(iv.as_slice()),
        ))
    }

    pub fn decrypt(&self, content: &str, password: &str, iv: &str) -> Result<String, TotpError> {
        let base64_decoded_content = BASE64.decode(content.as_bytes())?;
        let iv_decoded = BASE64.decode(iv.as_bytes())?;
        let cipher = Cipher::aes_256_cbc();
        let mut password = password.as_bytes().to_vec();
        while password.len() < cipher.key_len() {
            password.push(b'0');
        }
        let data = base64_decoded_content.as_slice();
        let key = password.as_slice();
        let decrypted_content = decrypt(cipher, key, Some(iv_decoded.as_slice()), data)
            .map_err(|_e| TotpError::Decryption("Invalid password".to_string()))?;
        let decrypted_content =
            String::from_utf8(decrypted_content).map_err(|e| TotpError::Utf8(e.to_string()))?;
        if !decrypted_content.contains(format!("\n{}:{}", self.key, self.value).as_str()) {
            return Err(TotpError::Decryption("Invalid password".to_string()));
        }
        let decrypted_content =
            decrypted_content.replace(&format!("\n{}:{}", self.key, self.value), "");
        Ok(decrypted_content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt() {
        let encrypt = Encryption::default();
        let (content, _iv) = encrypt.encrypt("TestContent", "password").unwrap();
        assert!(!content.contains("TestContent"));
    }

    #[test]
    fn decrypt() {
        let encrypt = Encryption::default();
        let content = encrypt
            .decrypt(
                "3wm4AUCJG+/Cr+NiZ/6M1tRaJp8ivdJCIFbzI8CcsTE=",
                "password",
                "ow1G5PUj8YY3Avnq2QpOPQ==",
            )
            .unwrap();
        assert_eq!(content, "TestContent");
    }

    #[test]
    fn decrypt_invalid_password() {
        let encrypt = Encryption::default();
        let content = encrypt.decrypt(
            "3wm4AUCJG+/Cr+NiZ/6M1tRaJp8ivdJCIFbzI8CcsTE=",
            "wrong",
            "ow1G5PUj8YY3Avnq2QpOPQ==",
        );
        assert!(content.is_err());
    }

    #[test]
    fn encrypt_decrypt() {
        let encrypt = Encryption::default();
        let (content, iv) = encrypt.encrypt("TestContent", "password").unwrap();
        assert!(!content.contains("TestContent"));
        let content = encrypt.decrypt(&content, "password", &iv).unwrap();
        assert_eq!(content, "TestContent");
    }
}
