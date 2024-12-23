extern crate core;

use std::io::Write;
use std::net::SocketAddr;

use crate::db::encryption::Encryption;
use crate::db::models::record::Record;
use crate::db::Db;
use crate::errors::TotpError;
use crate::ui::app::App;
use crate::ui::clip::set_clipboard;
use crate::ui::event_handler::{Event, EventHandler};
use crate::ui::tui::Tui;
use chrono::{DateTime, FixedOffset};
use clap::{Parser, Subcommand, ValueEnum};
use db::storage::StorageTrait;
use env_logger::Env;
use otp::generator::Generator;
use otp::token::Token;
use rpassword::read_password;

mod api;
mod db;
mod errors;
mod otp;
mod ui;

/// A CLI and TUI TOTP manager
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The encryption password
    #[clap(short, long)]
    password: Option<String>,
    /// The sqlite filename
    #[clap(short, long, default_value = ".totp.sqlite3")]
    sqlite_path: String,
    /// Automatically set the table lock key
    #[clap(short, long)]
    auto_lock_key: bool,
    /// Commands
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum OutputFormat {
    /// Json
    Json,
    /// List
    List,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new account
    Add {
        /// Account name
        #[clap(short, long)]
        account: String,

        /// User ID
        #[clap(short, long)]
        user: Option<String>,

        /// Note
        #[clap(short, long)]
        note: Option<String>,

        /// Password
        #[clap(short, long)]
        password: Option<String>,

        /// TOTP Secret
        #[clap(short, long)]
        secret: Option<Token>,

        /// Digits
        #[clap(short, long, default_value = "6")]
        digits: usize,

        /// Skew
        #[clap(short = 'k', long, default_value = "1")]
        skew: u8,

        /// Step
        #[clap(short = 't', long, default_value = "30")]
        step: u64,
    },
    /// Edit an existing account
    Edit {
        /// Id
        #[clap(short, long)]
        id: u32,

        /// Account name
        #[clap(short, long)]
        account: Option<String>,

        /// User ID
        #[clap(short, long)]
        user: Option<String>,

        /// Note
        #[clap(short, long)]
        note: Option<String>,

        /// Password
        #[clap(short, long)]
        password: Option<String>,

        /// TOTP Secret
        #[clap(short, long)]
        secret: Option<Token>,
    },
    /// Delete an account
    Delete {
        /// Id of account to delete
        #[clap(short, long)]
        id: u32,
    },
    /// Run in interactive mode [default]
    Interactive,
    /// Check an OTP
    Check {
        /// Secret token for key
        #[clap(short, long)]
        token: Token,
        /// The generated OTP
        #[clap(short, long)]
        otp: String,
        /// The start time
        #[clap(short, long)]
        start: DateTime<FixedOffset>,
        /// Range in minutes (applies before and after the start time)
        #[clap(short, long, default_value = "1")]
        range: u64,
    },
    /// Dump the config file
    Dump {
        /// The format to dump in
        #[clap(short, long, default_value = "list")]
        #[arg(value_enum)]
        format: OutputFormat,
    },
    /// Extract the TOTP Secret from a record
    Secret {
        /// Id
        #[clap(short, long)]
        id: u32,
    },
    /// Start an HTTP Server
    Serve {
        /// Listening address
        #[clap(short, long, default_value = "0.0.0.0:8080")]
        listen: SocketAddr,
    },
}

fn main() -> Result<(), TotpError> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trotp=info")).init();
    let cli = Cli::parse();

    let password = match cli.password {
        Some(password) => password,
        None => {
            print!("Password: ");
            std::io::stdout().flush().unwrap();
            read_password().unwrap()
        }
    };

    let db = Db::new(password, Some(cli.sqlite_path))?;
    db.init()?;
    let mut storage = db::storage::sqlite::SqliteStorage::new(db, Encryption::default());
    storage.load()?;
    match storage.verify_lock_encryption() {
        Ok(_) => {}
        Err(TotpError::MissingLockKey) => {
            if cli.auto_lock_key {
                storage.set_lock_encryption()?;
            } else {
                println!("You have not set the table lock key, pass -a to automatically set it");
                return Err(TotpError::MissingLockKey);
            }
        }
        Err(e) => {
            return Err(e);
        }
    }
    let command = match &cli.command {
        Some(command) => command,
        None => &Commands::Interactive {},
    };
    match command {
        Commands::Add {
            account,
            user,
            note,
            password,
            secret,
            digits,
            skew,
            step,
        } => {
            let token = secret.as_ref().map(|secret| Token {
                secret: secret.secret.clone(),
                digits: *digits,
                skew: *skew,
                step: *step,
            });

            let record = Record {
                account: Some(account.to_string()),
                token,
                password: password.clone(),
                note: note.clone(),
                user: user.clone(),
                ..Record::default()
            };
            storage.add_account(record)?;
        }
        Commands::Edit {
            id,
            account,
            user,
            note,
            password,
            secret,
        } => {
            let mut record = storage.get_account(*id)?;
            record.account = account.clone().or(record.account);
            record.user = user.clone().or(record.user);
            record.note = note.clone().or(record.note);
            record.password = password.clone().or(record.password);
            let token = match (record.token, secret) {
                (Some(mut token), Some(secret)) => {
                    token.secret = secret.secret.clone();
                    Some(token)
                }
                (Some(token), _) => Some(token),
                (_, Some(secret)) => Some(secret.clone()),
                _ => None,
            };

            record.token = token;
            eprintln!("record = {:?}", record);
            storage.edit_account(record)?;
        }
        Commands::Secret { id } => {
            let record = storage.get_account(*id)?;
            if let Some(token) = record.token {
                println!("{}", token);
            } else {
                println!("No token found for record");
            }
        }
        Commands::Delete { id } => {
            storage.remove_account_by_id(id.to_owned())?;
        }
        Commands::Check {
            token,
            otp,
            start,
            range,
        } => {
            let generator = Generator::new(token.to_owned())?;
            let output = generator.check_range(otp, start.timestamp() as u64, *range)?;
            let local_date = DateTime::<FixedOffset>::from_utc(output, *start.offset());
            println!(
                "OTP {}\nValid At:\n{} UTC\n{} {}",
                otp,
                output,
                local_date.naive_local(),
                start.offset()
            );
        }
        Commands::Dump { format } => {
            if format == &OutputFormat::Json {
                println!("{}", serde_json::to_string(&storage.accounts()?)?);
            } else if format == &OutputFormat::List {
                let mut id_length = 2;
                let mut account_length = 30;
                let mut token_length = 20;
                let mut user_length = 10;
                let mut password_length = 10;
                let mut note_length = 10;
                for record in storage.accounts()? {
                    id_length = id_length.max(format!("{}", record.id).len());
                    account_length =
                        account_length.max(record.account.unwrap_or_else(|| "".to_string()).len());
                    token_length = token_length.max(
                        record
                            .token
                            .map(|t| format!("{}", t))
                            .unwrap_or("".to_string())
                            .len(),
                    );
                    user_length =
                        user_length.max(record.user.unwrap_or_else(|| "".to_string()).len());
                    password_length = password_length
                        .max(record.password.unwrap_or_else(|| "".to_string()).len());
                    note_length =
                        note_length.max(record.note.unwrap_or_else(|| "".to_string()).len());
                }

                // Use the above lengths to set the format lengths
                println!(
                    "{: <id_length$} | {: <account_length$} | {: <token_length$} | {: <user_length$} | {: <password_length$} | {: <10}",
                    "ID",
                    "Account",
                    "Token",
                    "User",
                    "Password",
                    "Note",
                    id_length = id_length,
                    account_length = account_length,
                    token_length = token_length,
                    user_length = user_length,
                    password_length = password_length,
                );

                for record in storage.accounts()? {
                    println!(
                        "{: <id_length$} | {: <account_length$} | {: <token_length$} | {: <user_length$} | {: <password_length$} | {: <10}",

                        record.id,
                        record.account.unwrap_or_else(|| "".to_string()),
                        record
                            .token
                            .map(|t| format!("{}", t))
                            .unwrap_or("".to_string()),
                        record.user.unwrap_or_else(|| "".to_string()),
                        record.password.unwrap_or_else(|| "".to_string()),
                        record.note.unwrap_or_else(|| "".to_string()),
                        id_length = id_length,
                        account_length = account_length,
                        token_length = token_length,
                        user_length = user_length,
                        password_length = password_length,
                    );
                }
            }
        }
        Commands::Interactive => {
            ui::init(storage)?;
        }
        Commands::Serve { listen } => {
            api::server::Server::new(*listen, storage)?.start()?;
        }
    }
    Ok(())
}
