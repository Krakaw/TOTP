use crate::{Generator, Storage, TotpError};
use chrono::NaiveDateTime;
use std::io::stdout;
use std::io::Write;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

#[derive(Debug)]
pub enum OutputFormat {
    Short,
    Long,
}

impl FromStr for OutputFormat {
    type Err = TotpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "s" | "short" => Ok(OutputFormat::Short),
            "l" | "long" => Ok(OutputFormat::Long),
            _ => Err(TotpError::Clap("Invalid Output Format".to_string())),
        }
    }
}

pub struct Display {
    pub storage: Storage,
}

impl Display {
    pub fn render(
        &self,
        account: &Option<String>,
        time: &Option<NaiveDateTime>,
        format: &OutputFormat,
        repeat: &bool,
    ) -> Result<(), TotpError> {
        loop {
            let mut lines = Vec::new();
            for (account_name, token) in self.storage.accounts.iter().filter(|(acc, _token)| {
                if let Some(account) = account {
                    acc.to_lowercase().contains(&account.to_lowercase())
                } else {
                    true
                }
            }) {
                let generator = Generator::new(token)?;
                let (totp, expiry) = generator.generate(time.map(|t| t.timestamp() as u64))?;
                let colour = if expiry > 15 {
                    "\x1b[92m" // Green
                } else if expiry > 5 {
                    "\x1b[93m" // Yellow
                } else {
                    "\x1b[91m" //Red
                };
                let output = match format {
                    OutputFormat::Long => format!(
                        "{: <10} {: <10} {}{:0>2}\x1b[0m",
                        account_name, totp, colour, expiry
                    ),
                    OutputFormat::Short => format!("{}", totp),
                };
                lines.push(output.len());
                print!("{}\n", output);
            }
            if *repeat {
                sleep(Duration::from_secs(1));
                for char_count in lines {
                    for _i in 0..char_count {
                        print!("{}", (8u8 as char));
                    }
                    print!("{}\r", (8u8 as char));
                    stdout()
                        .flush()
                        .map_err(|e| TotpError::StdIO(e.to_string()))?;
                }
            } else {
                break;
            }
        }
        Ok(())
    }
}
