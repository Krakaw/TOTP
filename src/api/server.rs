use crate::{Generator, Storage, Token, TotpError};
use serde_json::json;
use std::net::SocketAddr;
use std::str::FromStr;
use tiny_http::{Response, Server as TinyServer};
pub struct Server {
    listen: SocketAddr,
    server: TinyServer,
    storage: Storage,
}

impl Server {
    pub fn new(listen: SocketAddr, storage: Storage) -> Result<Self, TotpError> {
        let server =
            TinyServer::http(listen.clone()).map_err(|e| TotpError::HttpServer(e.to_string()))?;
        Ok(Self {
            listen,
            server,
            storage,
        })
    }

    pub fn start(&self) -> Result<(), TotpError> {
        println!("Listening on {:?}", self.listen);
        ctrlc::set_handler(move || {
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");

        for request in self.server.incoming_requests() {
            let account_or_secret = request.url().replace("/", "");
            let decoded = urlencoding::decode(&account_or_secret)
                .map_err(|e| TotpError::Utf8(e.to_string()))?;
            let account_token_result =
                self.storage
                    .search_accounts(decoded.to_string())
                    .or_else(|_e| {
                        Token::from_str(&account_or_secret)
                            .map(|token| ("Secret".to_string(), token))
                    });

            let result = if let Ok((account_name, token)) = account_token_result {
                if let Ok(generator) = Generator::new(token) {
                    let (code, expiry) = generator.generate(None)?;
                    json!({"account_name": account_name, "code": code, "expiry": expiry})
                } else {
                    json!({"error": "Failed to create generator"})
                }
            } else {
                json!({"error": "Invalid token or account provided"})
            };

            let response = Response::from_string(result.to_string());
            request.respond(response)?;
        }
        Ok(())
    }
}
