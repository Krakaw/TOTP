extern crate core;

mod encryption;
mod errors;
mod generator;
mod storage;

use crate::errors::TotpError;
use crate::generator::Generator;
use crate::storage::{Storage, Token};
use chrono::NaiveDateTime;
use clap::{Parser, Subcommand};
use rpassword::read_password;
use std::io::{stdout, Write};
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
        } => loop {
            let mut lines = Vec::new();
            for (account_name, token) in storage.accounts.iter().filter(|(acc, _token)| {
                if let Some(account) = account {
                    acc.to_lowercase().contains(&account.to_lowercase())
                } else {
                    true
                }
            }) {
                let generator = Generator::new(token)?;
                let (totp, expiry) = generator.generate(time.map(|t| t.timestamp() as u64))?;
                let colour = if expiry > 15 {
                    "\x1b[92m"
                } else if expiry > 5 {
                    "\x1b[93m"
                } else {
                    "\x1b[91m"
                };
                let output = format!(
                    "{: <10} {: <10} {}{:0>2}\x1b[0m",
                    account_name, totp, colour, expiry
                );
                lines.push(output.len());
                print!("{}\n", output);
            }
            if *repeat {
                sleep(Duration::from_secs(1));
                for char_count in lines {
                    for i in 0..char_count {
                        print!("{}", (8u8 as char));
                    }
                    print!("{}\r", (8u8 as char));
                    stdout().flush();
                }
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
