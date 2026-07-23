use std::fs::OpenOptions;
use std::io::Write;
use std::time::Duration;

use tauri_plugin_shell::ShellExt;

use crate::error::CntrlError;

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

const PLAYWRIGHT_TIMEOUT: Duration = Duration::from_secs(45);

pub async fn fetch_fallback_html<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    url: &str,
) -> Result<String, CntrlError> {
    log_fallback(url)?;

    let shell = app.shell();

    let output = shell
        .command("node")
        .args(["--eval", PLAYWRIGHT_SCRIPT, "--", url])
        .output()
        .await
        .map_err(|e| CntrlError::Browser(format!("Failed to spawn Playwright subprocess: {e}")))?;

    let _ = PLAYWRIGHT_TIMEOUT;

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

    #[test]
    fn log_fallback_writes_entry() {
        let test_url = "https://test.example.com/fallback-log-test";
        let log_path = std::env::temp_dir().join("cntrl-fallback-test.log");
        let _ = std::fs::remove_file(&log_path);

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

        let _ = std::fs::remove_file(&log_path);
    }

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
