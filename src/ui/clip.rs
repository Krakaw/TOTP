use crate::errors::TotpError;
use std::process::Command;

pub fn set_clipboard(content: String) -> Result<(), TotpError> {
    if cfg!(feature = "clip") {
        let executable = if cfg!(target_os = "windows") {
            "clip.exe"
        } else {
            "xsel --clipboard --input"
        };
        Command::new("sh")
            .args(["-c", &format!("echo -n '{}' | {}", content, executable)])
            .output()
            .map_err(|e| TotpError::ClipboardError(e.to_string()))?;
    } else if cfg!(feature = "arboard") {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        clipboard
            .set_text(content)
            .map_err(|e| TotpError::ClipboardError(e.to_string()))?;
    }
    Ok(())
}
