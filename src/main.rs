extern crate core;

mod display;
mod encryption;
mod errors;
mod generator;
mod storage;

use crate::display::{Display, OutputFormat};
use crate::errors::TotpError;
use crate::generator::Generator;
use crate::storage::{Storage, Token};
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use rpassword::read_password;
use std::io::Write;

/// Generate TOTP codes
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
    command: Commands,
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
    },
    /// Delete an account
    Delete {
        /// Account name
        #[clap(short, long)]
        account: String,
    },
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
    match &cli.command {
        Commands::Add { account, secret } => {
            storage.add_account(account.to_owned(), secret.to_owned())?;
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
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
