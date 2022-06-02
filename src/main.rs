extern crate core;

mod errors;
mod generator;
mod storage;

use crate::errors::TotpError;
use crate::generator::Generator;
use crate::storage::{Storage, Token};
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use rpassword::read_password;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

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
    /// Generate an OTP
    Generate {
        /// Account name
        #[clap(short, long)]
        account: Option<String>,
        /// Time of token
        #[clap(short, long)]
        time: Option<NaiveDateTime>,
        /// Loop every n seconds
        #[clap(short, long)]
        repeat_delay: Option<u64>,
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
    let mut storage = Storage::new(password, Some(cli.filename));
    match &cli.command {
        Commands::Add { account, secret } => {
            storage.add_account(account.to_owned(), secret.to_owned());
        }
        Commands::Generate {
            account,
            time,
            repeat_delay,
        } => loop {
            for (account_name, token) in storage.accounts.iter().filter(|(acc, _token)| {
                if let Some(account) = account {
                    acc.to_lowercase().contains(&account.to_lowercase())
                } else {
                    true
                }
            }) {
                let generator = Generator::new(token)?;
                let totp = generator
                    .generate(time.map(|t| t.timestamp() as u64))
                    .unwrap();
                println!("{: <10} : {: <10}", account_name, totp);
            }
            if let Some(repeat_delay) = repeat_delay {
                sleep(Duration::from_secs(*repeat_delay));
            } else {
                break;
            }
        },
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
}
