use totp_rs::{Algorithm, TOTP};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Wasm {
    secret: String,
}

#[wasm_bindgen]
impl Wasm {
    #[wasm_bindgen(constructor)]
    pub fn new(secret: String) -> Self {
        Self { secret }
    }

    #[wasm_bindgen]
    pub fn generate(&self, timestamp: JsValue) -> Result<String, JsValue> {
        let totp = TOTP::new(
            Algorithm::SHA1,
            6,
            1,
            30,
            self.secret.clone(),
            None,
            "".to_string(),
        )
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
        Ok(totp.generate(timestamp.as_f64().unwrap_or(0f64) as u64))
    }
}
