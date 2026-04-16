use std::process::Command;

use crate::error::ToolError;

/// Open an application by name or bundle ID.
///
/// Uses macOS `open` command which handles both display names and bundle IDs.
pub fn open_application(app: &str) -> Result<(), ToolError> {
    // Try bundle ID first (-b flag)
    let status = Command::new("open")
        .args(["-b", app])
        .status()
        .map_err(|e| ToolError::AppFailed(format!("open command: {e}")))?;

    if status.success() {
        return Ok(());
    }

    // Fall back to app name (-a flag)
    let status = Command::new("open")
        .args(["-a", app])
        .status()
        .map_err(|e| ToolError::AppFailed(format!("open command: {e}")))?;

    if !status.success() {
        return Err(ToolError::AppFailed(format!(
            "could not open application '{app}'"
        )));
    }

    Ok(())
}

/// List installed applications by scanning /Applications.
pub fn list_installed_apps() -> Result<Vec<String>, ToolError> {
    let output = Command::new("ls")
        .arg("/Applications")
        .output()
        .map_err(|e| ToolError::AppFailed(format!("ls /Applications: {e}")))?;

    let text = String::from_utf8_lossy(&output.stdout);
    let apps: Vec<String> = text
        .lines()
        .filter(|name| name.ends_with(".app"))
        .map(|name| name.trim_end_matches(".app").to_string())
        .collect();

    Ok(apps)
}
