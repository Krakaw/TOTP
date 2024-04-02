use crate::errors::TotpError;
use std::process::Command;

pub fn set_clipboard(content: String) -> Result<(), TotpError> {
    if cfg!(feature = "clip") {
        Command::new("sh")
            .args(["-c", &format!("echo '{}' | clip.exe", content)])
            .output()
            .expect("failed to execute process");
    } else if cfg!(feature = "arboard") {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        clipboard
            .set_text(content)
            .map_err(|e| TotpError::ClipboardError(e.to_string()))?;
    }
    Ok(())
}
