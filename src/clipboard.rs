use std::process::Command;

use crate::error::ToolError;

/// Read clipboard contents via `pbpaste`.
pub fn read_clipboard() -> Result<String, ToolError> {
    let output = Command::new("pbpaste")
        .output()
        .map_err(|e| ToolError::ClipboardFailed(format!("pbpaste: {e}")))?;

    if !output.status.success() {
        return Err(ToolError::ClipboardFailed(format!(
            "pbpaste exited with code {}",
            output.status.code().unwrap_or(-1)
        )));
    }

    String::from_utf8(output.stdout)
        .map_err(|e| ToolError::ClipboardFailed(format!("invalid UTF-8: {e}")))
}

/// Write text to clipboard via `pbcopy`, with read-back verification.
pub fn write_clipboard(text: &str) -> Result<(), ToolError> {
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| ToolError::ClipboardFailed(format!("pbcopy spawn: {e}")))?;

    use std::io::Write;
    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| ToolError::ClipboardFailed(format!("pbcopy write: {e}")))?;
    }

    let status = child
        .wait()
        .map_err(|e| ToolError::ClipboardFailed(format!("pbcopy wait: {e}")))?;

    if !status.success() {
        return Err(ToolError::ClipboardFailed(format!(
            "pbcopy exited with code {}",
            status.code().unwrap_or(-1)
        )));
    }

    // Read-back verification (like the reference implementation)
    let readback = read_clipboard()?;
    if readback != text {
        return Err(ToolError::ClipboardFailed(
            "clipboard write verification failed — read-back mismatch".into(),
        ));
    }

    Ok(())
}
