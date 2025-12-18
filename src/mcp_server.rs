use std::io::{Read, Write};

use chrono::Utc;
use serde_json::{json, Value};

use crate::db::encryption::Encryption;
use crate::db::models::record::Record;
use crate::db::storage::sqlite::SqliteStorage;
use crate::db::storage::StorageTrait;
use crate::db::Db;
use crate::errors::TotpError;
use crate::otp::generator::Generator;
use crate::otp::token::Token;

mod db;
mod errors;
mod otp;

#[derive(Debug)]
struct RpcMessage {
    json: Value,
}

fn write_rpc(out: &mut dyn Write, msg: &Value) -> std::io::Result<()> {
    let body = serde_json::to_vec(msg).unwrap_or_else(|_| b"{\"jsonrpc\":\"2.0\"}".to_vec());
    write!(out, "Content-Length: {}\r\n\r\n", body.len())?;
    out.write_all(&body)?;
    out.flush()?;
    Ok(())
}

fn read_one_rpc(stdin: &mut dyn Read, buf: &mut Vec<u8>) -> std::io::Result<Option<RpcMessage>> {
    // MCP uses LSP-style framing: headers ending with \r\n\r\n then JSON body of Content-Length bytes.
    loop {
        // Try parse headers if we have them.
        if let Some(headers_end) = buf.windows(4).position(|w| w == b"\r\n\r\n").map(|i| i + 4) {
            let headers = &buf[..headers_end];
            let headers_str = String::from_utf8_lossy(headers);
            let mut content_length: Option<usize> = None;
            for line in headers_str.split("\r\n") {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                if let Some(rest) = line.strip_prefix("Content-Length:") {
                    if let Ok(v) = rest.trim().parse::<usize>() {
                        content_length = Some(v);
                    }
                }
            }
            let Some(len) = content_length else {
                // Malformed frame; drop up to headers end and keep going.
                buf.drain(..headers_end);
                continue;
            };
            if buf.len() < headers_end + len {
                // Need more data.
            } else {
                let body = buf[headers_end..headers_end + len].to_vec();
                buf.drain(..headers_end + len);
                if body.is_empty() {
                    continue;
                }
                let json: Value = match serde_json::from_slice(&body) {
                    Ok(v) => v,
                    Err(_) => continue,
                };
                return Ok(Some(RpcMessage { json }));
            }
        }

        let mut chunk = [0u8; 8192];
        let n = stdin.read(&mut chunk)?;
        if n == 0 {
            return Ok(None);
        }
        buf.extend_from_slice(&chunk[..n]);
    }
}

fn jsonrpc_error(id: Value, code: i64, message: &str, data: Option<Value>) -> Value {
    let mut err = json!({
        "code": code,
        "message": message,
    });
    if let Some(data) = data {
        err["data"] = data;
    }
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": err
    })
}

fn jsonrpc_result(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn tools_list() -> Value {
    json!({
        "tools": [
            {
                "name": "trotp_add",
                "description": "Add a new TOTP account into the encrypted sqlite store.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "db_password": { "type": "string", "description": "Encryption password for the local TOTP database." },
                        "sqlite_path": { "type": "string", "description": "Path to sqlite file. Defaults to .totp.sqlite3." },
                        "auto_lock_key": { "type": "boolean", "description": "Automatically create the table lock key if missing. Defaults to true." },

                        "account": { "type": "string", "description": "Account name (display name)." },
                        "user": { "type": ["string", "null"], "description": "Optional username / user id." },
                        "note": { "type": ["string", "null"], "description": "Optional note." },
                        "record_password": { "type": ["string", "null"], "description": "Optional password stored with the record (not the DB password)." },

                        "secret": { "type": ["string", "null"], "description": "Base32 TOTP secret. If omitted, record will be created without a token." },
                        "digits": { "type": "integer", "minimum": 4, "maximum": 10, "description": "OTP digits. Default 6." },
                        "skew": { "type": "integer", "minimum": 0, "maximum": 10, "description": "Allowed time-step skew. Default 1." },
                        "step": { "type": "integer", "minimum": 5, "maximum": 300, "description": "Time step in seconds. Default 30." }
                    },
                    "required": ["db_password", "account"]
                }
            },
            {
                "name": "trotp_edit",
                "description": "Edit an existing TOTP account (by id).",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "db_password": { "type": "string" },
                        "sqlite_path": { "type": "string" },
                        "auto_lock_key": { "type": "boolean" },

                        "id": { "type": "integer", "minimum": 1 },
                        "account": { "type": ["string", "null"] },
                        "user": { "type": ["string", "null"] },
                        "note": { "type": ["string", "null"] },
                        "record_password": { "type": ["string", "null"] },

                        "secret": { "type": ["string", "null"], "description": "New Base32 secret (replaces token secret)." },
                        "digits": { "type": ["integer", "null"], "minimum": 4, "maximum": 10 },
                        "skew": { "type": ["integer", "null"], "minimum": 0, "maximum": 10 },
                        "step": { "type": ["integer", "null"], "minimum": 5, "maximum": 300 }
                    },
                    "required": ["db_password", "id"]
                }
            },
            {
                "name": "trotp_delete",
                "description": "Delete an account (by id).",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "db_password": { "type": "string" },
                        "sqlite_path": { "type": "string" },
                        "auto_lock_key": { "type": "boolean" },
                        "id": { "type": "integer", "minimum": 1 }
                    },
                    "required": ["db_password", "id"]
                }
            },
            {
                "name": "trotp_generate",
                "description": "Generate a current TOTP code from a stored account (by id/account) or from a provided secret.",
                "inputSchema": {
                    "type": "object",
                    "additionalProperties": false,
                    "properties": {
                        "db_password": { "type": ["string", "null"], "description": "DB password is required when generating from a stored account. Not needed when generating from a one-off secret." },
                        "sqlite_path": { "type": "string" },
                        "auto_lock_key": { "type": "boolean" },

                        "id": { "type": ["integer", "null"], "minimum": 1 },
                        "account": { "type": ["string", "null"], "description": "Account name search (substring match)." },

                        "secret": { "type": ["string", "null"], "description": "One-off Base32 secret to generate from (bypasses DB)." },
                        "digits": { "type": ["integer", "null"], "minimum": 4, "maximum": 10 },
                        "skew": { "type": ["integer", "null"], "minimum": 0, "maximum": 10 },
                        "step": { "type": ["integer", "null"], "minimum": 5, "maximum": 300 },
                        "timestamp": { "type": ["integer", "null"], "minimum": 0, "description": "Unix timestamp (seconds). Defaults to now." }
                    }
                }
            }
        ]
    })
}

fn arg_str(args: &Value, key: &str) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}
fn arg_bool(args: &Value, key: &str) -> Option<bool> {
    args.get(key).and_then(|v| v.as_bool())
}
fn arg_u32(args: &Value, key: &str) -> Option<u32> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .and_then(|v| u32::try_from(v).ok())
}
fn arg_u8(args: &Value, key: &str) -> Option<u8> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .and_then(|v| u8::try_from(v).ok())
}
fn arg_usize(args: &Value, key: &str) -> Option<usize> {
    args.get(key)
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
}
fn arg_u64(args: &Value, key: &str) -> Option<u64> {
    args.get(key).and_then(|v| v.as_u64())
}

fn open_storage(
    db_password: String,
    sqlite_path: Option<String>,
    auto_lock_key: bool,
) -> Result<SqliteStorage, TotpError> {
    let db = Db::new(
        db_password,
        Some(sqlite_path.unwrap_or_else(|| ".totp.sqlite3".to_string())),
    )?;
    db.init()?;
    let mut storage = SqliteStorage::new(db, Encryption::default());
    storage.load()?;
    match storage.verify_lock_encryption() {
        Ok(_) => {}
        Err(TotpError::MissingLockKey) => {
            if auto_lock_key {
                storage.set_lock_encryption()?;
            } else {
                return Err(TotpError::MissingLockKey);
            }
        }
        Err(e) => return Err(e),
    }
    Ok(storage)
}

fn tool_add(args: &Value) -> Result<Value, TotpError> {
    let db_password = arg_str(args, "db_password")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| TotpError::ValidationError("db_password is required".to_string()))?;
    let sqlite_path = arg_str(args, "sqlite_path");
    let auto_lock_key = arg_bool(args, "auto_lock_key").unwrap_or(true);
    let account = arg_str(args, "account")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| TotpError::ValidationError("account is required".to_string()))?;

    let user = args
        .get("user")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let note = args
        .get("note")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let record_password = args
        .get("record_password")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let secret = args
        .get("secret")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let digits = arg_usize(args, "digits").unwrap_or(6);
    let skew = arg_u8(args, "skew").unwrap_or(1);
    let step = arg_u64(args, "step").unwrap_or(30);

    let mut storage = open_storage(db_password, sqlite_path, auto_lock_key)?;
    let token = match secret {
        Some(s) if !s.trim().is_empty() => {
            let mut t: Token = s.parse()?;
            t.digits = digits;
            t.skew = skew;
            t.step = step;
            Some(t)
        }
        _ => None,
    };

    let record = Record {
        account: Some(account.clone()),
        user,
        note,
        password: record_password,
        token,
        ..Record::default()
    };
    storage.add_account(record)?;

    // Try to return the newly created record (best-effort).
    let created = storage.accounts()?.into_iter().find(|r| {
        r.account
            .as_deref()
            .map(|a| a.eq_ignore_ascii_case(&account))
            .unwrap_or(false)
    });

    Ok(json!({ "ok": true, "record": created }))
}

fn tool_edit(args: &Value) -> Result<Value, TotpError> {
    let db_password = arg_str(args, "db_password")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| TotpError::ValidationError("db_password is required".to_string()))?;
    let sqlite_path = arg_str(args, "sqlite_path");
    let auto_lock_key = arg_bool(args, "auto_lock_key").unwrap_or(true);
    let id = arg_u32(args, "id").unwrap_or(0);

    let mut storage = open_storage(db_password, sqlite_path, auto_lock_key)?;
    let mut record = storage.get_account(id)?;

    if let Some(v) = args.get("account") {
        record.account = v.as_str().map(|s| s.to_string());
    }
    if let Some(v) = args.get("user") {
        record.user = v.as_str().map(|s| s.to_string());
    }
    if let Some(v) = args.get("note") {
        record.note = v.as_str().map(|s| s.to_string());
    }
    if let Some(v) = args.get("record_password") {
        record.password = v.as_str().map(|s| s.to_string());
    }

    let secret = args
        .get("secret")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let digits = args
        .get("digits")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok());
    let skew = args
        .get("skew")
        .and_then(|v| v.as_u64())
        .and_then(|v| u8::try_from(v).ok());
    let step = args.get("step").and_then(|v| v.as_u64());

    let mut token = record.token.clone();
    if let Some(secret) = secret {
        token = Some(secret.parse()?);
    }
    if let Some(ref mut t) = token {
        if let Some(d) = digits {
            t.digits = d;
        }
        if let Some(s) = skew {
            t.skew = s;
        }
        if let Some(s) = step {
            t.step = s;
        }
    } else if digits.is_some() || skew.is_some() || step.is_some() {
        // Cannot adjust token params without having a token.
        return Err(TotpError::Storage(
            "Cannot edit token parameters without a token/secret".to_string(),
        ));
    }
    record.token = token;

    storage.edit_account(record)?;
    let updated = storage.get_account(id)?;
    Ok(json!({ "ok": true, "record": updated }))
}

fn tool_delete(args: &Value) -> Result<Value, TotpError> {
    let db_password = arg_str(args, "db_password")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| TotpError::ValidationError("db_password is required".to_string()))?;
    let sqlite_path = arg_str(args, "sqlite_path");
    let auto_lock_key = arg_bool(args, "auto_lock_key").unwrap_or(true);
    let id = arg_u32(args, "id").unwrap_or(0);

    let mut storage = open_storage(db_password, sqlite_path, auto_lock_key)?;
    storage.remove_account_by_id(id)?;
    Ok(json!({ "ok": true }))
}

fn build_token_from_args(args: &Value) -> Result<Token, TotpError> {
    let secret = arg_str(args, "secret")
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| TotpError::ValidationError("secret is required".to_string()))?;
    let mut token: Token = secret.parse()?;
    if let Some(d) = args
        .get("digits")
        .and_then(|v| v.as_u64())
        .and_then(|v| usize::try_from(v).ok())
    {
        token.digits = d;
    }
    if let Some(s) = args
        .get("skew")
        .and_then(|v| v.as_u64())
        .and_then(|v| u8::try_from(v).ok())
    {
        token.skew = s;
    }
    if let Some(s) = args.get("step").and_then(|v| v.as_u64()) {
        token.step = s;
    }
    Ok(token)
}

fn tool_generate(args: &Value) -> Result<Value, TotpError> {
    let timestamp = args
        .get("timestamp")
        .and_then(|v| v.as_u64())
        .unwrap_or_else(|| Utc::now().timestamp() as u64);

    if args
        .get("secret")
        .and_then(|v| v.as_str())
        .map(|s| !s.trim().is_empty())
        .unwrap_or(false)
    {
        let token = build_token_from_args(args)?;
        let generator = Generator::new(token)?;
        let (code, expiry) = generator.generate(Some(timestamp))?;
        return Ok(json!({
            "source": "secret",
            "code": code,
            "expiry": expiry
        }));
    }

    let db_password = args
        .get("db_password")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            TotpError::Storage(
                "db_password is required when generating from stored accounts".to_string(),
            )
        })?;
    let sqlite_path = arg_str(args, "sqlite_path");
    let auto_lock_key = arg_bool(args, "auto_lock_key").unwrap_or(true);

    let storage = open_storage(db_password, sqlite_path, auto_lock_key)?;

    let record = if let Some(id) = args.get("id").and_then(|v| v.as_u64()) {
        storage.get_account(
            u32::try_from(id).map_err(|_| TotpError::Storage("invalid id".to_string()))?,
        )?
    } else if let Some(account) = args.get("account").and_then(|v| v.as_str()) {
        storage.search_account(account)?
    } else {
        return Err(TotpError::Storage(
            "Provide either secret, id, or account".to_string(),
        ));
    };

    let Some(token) = record.token.clone() else {
        return Err(TotpError::Storage(
            "No token/secret stored for this record".to_string(),
        ));
    };
    let generator = Generator::new(token)?;
    let (code, expiry) = generator.generate(Some(timestamp))?;
    Ok(json!({
        "source": "account",
        "id": record.id,
        "account": record.account,
        "code": code,
        "expiry": expiry
    }))
}

fn tool_call(name: &str, arguments: &Value) -> Result<Value, TotpError> {
    match name {
        "trotp_add" => tool_add(arguments),
        "trotp_edit" => tool_edit(arguments),
        "trotp_delete" => tool_delete(arguments),
        "trotp_generate" => tool_generate(arguments),
        _ => Err(TotpError::Storage(format!("Unknown tool: {}", name))),
    }
}

fn tool_result_text(value: Value) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": value.to_string()
            }
        ]
    })
}

fn main() -> Result<(), TotpError> {
    let mut stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    let mut buf: Vec<u8> = Vec::new();


    while let Some(msg) = read_one_rpc(&mut stdin, &mut buf)? {
        let id = msg.json.get("id").cloned().unwrap_or(Value::Null);
        let method = msg
            .json
            .get("method")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        let params = msg.json.get("params").cloned().unwrap_or(Value::Null);


        // Notifications don't require replies (id is null/missing). We still handle exit gracefully.
        let needs_reply = !id.is_null();

        let reply = match method {
            "initialize" => {
                let version = env!("CARGO_PKG_VERSION");
                // Use client's protocol version if provided, otherwise default
                let client_protocol = params
                    .get("protocolVersion")
                    .and_then(|v| v.as_str())
                    .unwrap_or("2024-11-05");
                jsonrpc_result(
                    id,
                    json!({
                        "protocolVersion": client_protocol,
                        "serverInfo": { "name": "trotp-mcp", "version": version },
                        "capabilities": { "tools": {} }
                    }),
                )
            }
            "tools/list" => jsonrpc_result(id, tools_list()),
            "tools/call" => {
                let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                match tool_call(tool_name, &args) {
                    Ok(v) => jsonrpc_result(id, tool_result_text(v)),
                    Err(e) => jsonrpc_result(
                        id,
                        json!({
                            "content": [{ "type": "text", "text": format!("{:?}", e) }],
                            "isError": true
                        }),
                    ),
                }
            }
            "notifications/initialized" => {
                // Notification - no reply needed, just continue
                json!({})
            }
            "ping" => jsonrpc_result(id, json!({})),
            "shutdown" => jsonrpc_result(id, json!({})),
            "exit" => {
                // exit is typically a notification.
                if needs_reply {
                    jsonrpc_result(id, json!({}))
                } else {
                    // No reply required
                    break;
                }
            }
            _ => jsonrpc_error(
                id,
                -32601,
                "Method not found",
                Some(json!({ "method": method })),
            ),
        };

        if needs_reply {
            // If writing fails, exit the loop.
            if write_rpc(&mut stdout, &reply).is_err() {
                break;
            }
        } else if method == "exit" {
            break;
        }
    }

    Ok(())
}
