extern crate core;
mod api;
mod display;
mod errors;
mod otp;
mod storage;
mod ui;

use crate::display::{Display, OutputFormat};
use crate::errors::TotpError;
use crate::storage::accounts::Storage;
use crate::ui::app::App;
use crate::ui::event_handler::{Event, EventHandler};
use crate::ui::tui::Tui;
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use clap::{Parser, Subcommand};
use otp::generator::Generator;
use otp::token::Token;
use rpassword::read_password;
use std::io::Write;
use std::net::SocketAddr;

/// A CLI and TUI TOTP manager
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The encryption password
    #[clap(short, long)]
    password: Option<String>,
    /// The storage filename
    #[clap(short, long, default_value = ".storage.txt")]
    filename: String,
    #[clap(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new account
    Add {
        /// Account name
        #[clap(short, long)]
        account: String,

        /// TOTP Secret
        #[clap(short, long)]
        secret: Token,

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
    /// Delete an account
    Delete {
        /// Account name
        #[clap(short, long)]
        account: String,
    },
    /// Run in interactive mode [default]
    Interactive,
    /// Generate an OTP
    Generate {
        /// Account name
        #[clap(short, long)]
        account: Option<String>,
        /// Time of token
        #[clap(short, long)]
        time: Option<NaiveDateTime>,
        /// Run on a loop
        #[clap(short, long)]
        repeat: bool,
        /// Output format
        #[clap(short, long, default_value = "long")]
        format: OutputFormat,
    },
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
    Dump,
    /// Start an HTTP Server
    Server {
        /// Listening address
        #[clap(short, long, default_value = "0.0.0.0:8080")]
        listen: SocketAddr,
    },
}

fn main() -> Result<(), TotpError> {
    let cli = Cli::parse();

    let password = match cli.password {
        Some(password) => password,
        None => {
            print!("Password: ");
            std::io::stdout().flush().unwrap();
            read_password().unwrap()
        }
    };
    let mut storage = Storage::new(password, Some(cli.filename))?;
    let command = match &cli.command {
        Some(command) => command,
        None => &Commands::Interactive {},
    };
    match command {
        Commands::Add {
            account,
            secret,
            digits,
            skew,
            step,
        } => {
            let token = Token {
                secret: secret.secret.clone(),
                digits: digits.clone(),
                skew: skew.clone(),
                step: step.clone(),
            };
            storage.add_account(account.to_owned(), token)?;
        }
        Commands::Delete { account } => {
            storage.remove_account(account.to_owned())?;
        }
        Commands::Generate {
            account,
            time,
            repeat,
            format,
        } => {
            let display = Display { storage };
            display.render(account, time, format, repeat)?;
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
        Commands::Dump => {
            for (a, t) in storage.to_iter() {
                println!("{}\t{}", a, t);
            }
        }
        Commands::Interactive => {
            ui::init(storage)?;
        }
        Commands::Server { listen } => {
            api::server::Server::new(listen.clone(), storage)?.start()?;
        }
    }
    Ok(())
}
