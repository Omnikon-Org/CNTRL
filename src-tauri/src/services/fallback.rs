use std::fs::OpenOptions;
use std::io::Write;

use crate::error::CntrlError;

const FALLBACK_USER_AGENT: &str =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36";

pub async fn fetch_fallback_html<R: tauri::Runtime>(
    _app: &tauri::AppHandle<R>,
    url: &str,
) -> Result<String, CntrlError> {
    let _ = log_fallback(url);

    let client = reqwest::Client::builder()
        .user_agent(FALLBACK_USER_AGENT)
        .build()
        .map_err(|e| CntrlError::Browser(format!("Failed to build HTTP client: {}", e)))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| CntrlError::Browser(format!("Failed to fetch URL: {}", e)))?;

    if !response.status().is_success() {
        return Err(CntrlError::Browser(format!(
            "HTTP {} for {}",
            response.status(),
            url
        )));
    }

    let html = response
        .text()
        .await
        .map_err(|e| CntrlError::Browser(format!("Failed to read response body: {}", e)))?;

    Ok(html)
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
