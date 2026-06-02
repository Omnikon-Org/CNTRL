use std::fs::OpenOptions;
use std::io::Write;
use tauri::Manager;

use crate::error::CntrlError;

pub async fn fetch_fallback_html<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    url: &str,
) -> Result<String, CntrlError> {
    let fallback_path = app
        .path()
        .resource_dir()
        .map(|p| p.join("fallback.mjs"))
        .unwrap_or_else(|_| std::path::PathBuf::from("fallback.mjs"));

    let _ = log_fallback(url);

    let output = std::process::Command::new("node")
        .arg(fallback_path)
        .arg(url)
        .output()
        .map_err(|e| CntrlError::Browser(format!("Failed to spawn fallback: {}", e)))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(CntrlError::Browser(format!(
            "Fallback script error: {}",
            err
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn log_fallback(url: &str) -> Result<(), CntrlError> {
    let log_path = std::env::temp_dir().join("cntrl-fallback.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    writeln!(file, "[{}] Fallback activated for URL: {}", timestamp, url)?;

    Ok(())
}
