use std::process::Command;
use crate::errors::TotpError;

pub fn set_clipboard(content: String) -> Result<(), TotpError> {
    if cfg!(feature = "clip") {
        Command::new("sh").args(&["-c", &format!("echo '{}' | clip.exe", content)]).output().expect("failed to execute process");
    } else if cfg!(feature = "cli-clipboard") {
        cli_clipboard::set_contents(content).map_err(|e| TotpError::ClipboardError(e.to_string()))?;
    }
    Ok(())

}