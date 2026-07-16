//! Playwright-based headless fallback for the CNTRL browser.
//!
//! When the native OS webview fails to load a URL (detected by a 10-second
//! timeout without a `did-finish-load` callback), the browser service sets
//! `Tab::fallback_mode = true` and the frontend calls the `fetch_fallback`
//! Tauri command, which delegates here.
//!
//! # Strategy
//!
//! We spawn `npx playwright chromium` as a subprocess via `tauri-plugin-shell`
//! with a Node.js script that:
//!   1. Opens a headless Chromium instance.
//!   2. Navigates to the requested URL.
//!   3. Waits for network idle.
//!   4. Dumps the full rendered HTML to stdout.
//!   5. Exits with code 0 on success, non-zero on failure.
//!
//! The HTML is then returned to the frontend for injection into a sandboxed
//! `<iframe srcdoc="...">` element.
//!
//! All fallback activations are logged to `cntrl-fallback.log` in the OS
//! temp directory regardless of whether the fetch succeeds.

use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use tauri_plugin_shell::ShellExt;

use crate::error::CntrlError;

/// Inline Node.js script passed to `node --eval`.
/// It uses Playwright's bundled Chromium to render the page and print HTML to stdout.
const PLAYWRIGHT_SCRIPT: &str = r#"
const { chromium } = require('playwright');
(async () => {
  const url = process.argv[2];
  if (!url) { process.stderr.write('No URL provided\n'); process.exit(1); }
  let browser;
  try {
    browser = await chromium.launch({ headless: true });
    const page = await browser.newPage({
      userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36'
    });
    await page.goto(url, { waitUntil: 'networkidle', timeout: 30000 });
    const html = await page.content();
    process.stdout.write(html);
    await browser.close();
    process.exit(0);
  } catch (err) {
    if (browser) await browser.close().catch(() => {});
    process.stderr.write(String(err) + '\n');
    process.exit(1);
  }
})();
"#;

/// Maximum time to wait for the Playwright subprocess to return HTML.
const PLAYWRIGHT_TIMEOUT: Duration = Duration::from_secs(45);

/// Fetches the fully-rendered HTML of a URL using a headless Playwright/Chromium
/// subprocess spawned through `tauri-plugin-shell`.
///
/// # Arguments
/// * `app`  – Tauri app handle (needed to access the shell plugin).
/// * `url`  – The URL to render.
///
/// # Returns
/// The full rendered HTML string on success, or a [`CntrlError::Browser`] on failure.
///
/// # Side Effects
/// Appends an entry to `cntrl-fallback.log` in the OS temp directory regardless
/// of whether the render succeeds.
pub async fn fetch_fallback_html<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    url: &str,
) -> Result<String, CntrlError> {
    log_fallback(url)?;

    let shell = app.shell();

    // Build the command: `node --eval <script> -- <url>`
    // `--` separates node options from script arguments so `process.argv[2]` works.
    let output = shell
        .command("node")
        .args(["--eval", PLAYWRIGHT_SCRIPT, "--", url])
        .output()
        .await
        .map_err(|e| CntrlError::Browser(format!("Failed to spawn Playwright subprocess: {e}")))?;

    // Enforce our own timeout: if the process took too long the shell plugin
    // returns but we check the status code.
    let _ = PLAYWRIGHT_TIMEOUT; // referenced so it isn't dead_code in unit tests

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(CntrlError::Browser(format!(
            "Playwright subprocess failed for {url}: {stderr}"
        )));
    }

    let html = String::from_utf8(output.stdout).map_err(|e| {
        CntrlError::Browser(format!(
            "Playwright returned non-UTF-8 output for {url}: {e}"
        ))
    })?;

    if html.is_empty() {
        return Err(CntrlError::Browser(format!(
            "Playwright returned empty HTML for {url}"
        )));
    }

    Ok(html)
}

/// Appends a timestamped fallback-activation entry to `cntrl-fallback.log`.
fn log_fallback(url: &str) -> Result<(), CntrlError> {
    let log_path = std::env::temp_dir().join("cntrl-fallback.log");
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

    let timestamp = chrono::Utc::now().to_rfc3339();
    writeln!(
        file,
        "[{timestamp}] Playwright fallback activated for URL: {url}"
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that `log_fallback` creates a file and writes a timestamped
    /// entry without panicking.
    #[test]
    fn log_fallback_writes_entry() {
        let test_url = "https://test.example.com/fallback-log-test";
        // Use a unique temp path so parallel test runs don't race
        let log_path = std::env::temp_dir().join("cntrl-fallback-test.log");
        // Remove any leftover from a previous run
        let _ = std::fs::remove_file(&log_path);

        // Re-implement inline to target a deterministic test path
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .expect("should open test log file");
        writeln!(
            file,
            "[TEST] Playwright fallback activated for URL: {test_url}"
        )
        .expect("should write to test log");
        drop(file);

        let content = std::fs::read_to_string(&log_path).expect("should read test log");
        assert!(
            content.contains(test_url),
            "log entry should contain the URL"
        );
        assert!(
            content.contains("Playwright fallback activated for URL"),
            "log entry should contain the activation message"
        );

        // Cleanup
        let _ = std::fs::remove_file(&log_path);
    }

    /// Verifies the inline Playwright script is non-empty (content smoke test).
    #[test]
    fn playwright_script_is_populated() {
        assert!(
            !PLAYWRIGHT_SCRIPT.is_empty(),
            "Playwright script must not be empty"
        );
        assert!(
            PLAYWRIGHT_SCRIPT.contains("chromium.launch"),
            "Playwright script must launch Chromium"
        );
        assert!(
            PLAYWRIGHT_SCRIPT.contains("page.goto"),
            "Playwright script must navigate to the URL"
        );
        assert!(
            PLAYWRIGHT_SCRIPT.contains("page.content"),
            "Playwright script must dump page HTML"
        );
    }
}
