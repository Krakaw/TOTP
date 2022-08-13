use crate::db::models::record::Record;
use crate::{Generator, Storage, StorageTrait, Token, TotpError};
use serde_json::json;
use std::net::SocketAddr;
use std::str::FromStr;
use tiny_http::{Response, Server as TinyServer};

pub struct Server<T: StorageTrait> {
    listen: SocketAddr,
    server: TinyServer,
    storage: T,
}

impl<T> Server<T>
where
    T: StorageTrait,
{
    pub fn new(listen: SocketAddr, storage: T) -> Result<Self, TotpError> {
        let server = TinyServer::http(listen).map_err(|e| TotpError::HttpServer(e.to_string()))?;
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
            let account_or_secret = request.url().replace('/', "");
            let decoded = urlencoding::decode(&account_or_secret)
                .map_err(|e| TotpError::Utf8(e.to_string()))?;
            let account_token_result =
                self.storage
                    .search_account(decoded.to_string())
                    .or_else(|_e| {
                        Token::from_str(&account_or_secret)
                            .map(|token| ("Secret".to_string(), token))
                            .and_then(|(account_name, token)| {
                                Ok((
                                    account_name,
                                    Record {
                                        token: Some(token),
                                        ..Record::default()
                                    },
                                ))
                            })
                    });

            let result = if let Ok((account_name, secure_data)) = account_token_result {
                if let Some(token) = secure_data.token {
                    if let Ok(generator) = Generator::new(token) {
                        let (code, expiry) = generator.generate(None)?;
                        json!({"account_name": account_name, "code": code, "expiry": expiry})
                    } else {
                        json!({"error": "Failed to create generator"})
                    }
                } else {
                    json!({"error": "Invalid token or account provided"})
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
