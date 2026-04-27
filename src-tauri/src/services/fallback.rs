use std::fs::OpenOptions;
use std::io::Write;
use tauri_plugin_shell::ShellExt;
use tauri_plugin_shell::process::CommandEvent;

use crate::error::VibError;

pub async fn fetch_fallback_html(app: &tauri::AppHandle, url: &str) -> Result<String, VibError> {
    log_fallback(url)?;

    let (mut rx, _child) = app
        .shell()
        .command("node")
        .args(["fallback.mjs", url])
        .spawn()
        .map_err(|e| VibError::Browser(format!("Failed to spawn fallback: {}", e)))?;

    let mut output = String::new();
    while let Some(event) = rx.recv().await {
        if let CommandEvent::Stdout(line) = event {
            output.push_str(&String::from_utf8_lossy(&line));
        } else if let CommandEvent::Stderr(line) = event {
            eprintln!("Fallback error: {}", String::from_utf8_lossy(&line));
        }
    }

    Ok(output)
}

fn log_fallback(url: &str) -> Result<(), VibError> {
    let log_path = std::env::temp_dir().join("vibe-fallback.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    writeln!(file, "[{}] Fallback activated for URL: {}", timestamp, url)?;

    Ok(())
}
