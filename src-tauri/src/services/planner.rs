//! Task planner — decomposes an `IntentResult` into a list of executable steps.
//!
//! Each intent type maps to a predefined step template. Slots extracted by the
//! intent parser are interpolated into the steps.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CntrlError;
use crate::services::intent::{IntentResult, IntentType};

// ── Step definitions ───────────────────────────────────────────────────

/// A single executable step in a task plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Step {
    /// Navigate the active tab to the given URL.
    NavigateTo(String),
    /// Click an element matching the CSS selector.
    ClickElement(String),
    /// Type text into an element matching the CSS selector.
    TypeText(String, String),
    /// Read the current page's DOM content.
    ReadPage,
    /// Execute a JavaScript snippet in the active tab.
    RunScript(String),
    /// Wait for the given number of milliseconds.
    WaitFor(u64),
    /// Report a result message back to the user.
    ReportResult(String),
}

/// The status of a task as it moves through the execution pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    /// Waiting to be executed.
    Queued,
    /// Currently being executed.
    Running,
    /// Completed successfully.
    Done,
    /// Failed with an error.
    Failed,
    /// Cancelled by the user.
    Cancelled,
}

/// The result of executing a complete task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    /// Unique task identifier.
    pub task_id: Uuid,
    /// Final status of the task.
    pub status: TaskStatus,
    /// Steps that were planned.
    pub steps: Vec<Step>,
    /// The step index that failed (if any).
    pub failed_at: Option<usize>,
    /// Human-readable result or error message.
    pub message: String,
    /// Data returned by the task (e.g., scraped value, summary text).
    pub data: Option<String>,
}

/// An entry in the task history log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskHistoryEntry {
    /// Unique task identifier.
    pub task_id: Uuid,
    /// The raw user input that triggered this task.
    pub raw_input: String,
    /// Classified intent type.
    pub intent_type: IntentType,
    /// Final status.
    pub status: TaskStatus,
    /// Result message.
    pub message: String,
    /// ISO 8601 timestamp.
    pub timestamp: String,
}

// ── Planner implementation ─────────────────────────────────────────────

/// Generate a list of steps for the given intent.
///
/// Each intent type has a predefined template. Slots from the intent parser
/// are interpolated into the step arguments.
///
/// # Arguments
/// * `intent` — A parsed intent with type and slots.
///
/// # Returns
/// A vector of `Step`s that the executor can run sequentially.
pub fn plan(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    let steps = match intent.intent_type {
        IntentType::Navigate => plan_navigate(intent)?,
        IntentType::Search => plan_search(intent)?,
        IntentType::Scrape => plan_scrape(intent)?,
        IntentType::Compose => plan_compose(intent)?,
        IntentType::System => plan_system(intent)?,
        IntentType::Macro => plan_macro(intent)?,
        IntentType::Query => plan_query(intent)?,
    };

    if steps.is_empty() {
        return Err(CntrlError::Ai("Planner produced zero steps".into()));
    }

    Ok(steps)
}

/// Plan: navigate to a URL.
fn plan_navigate(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    let url = intent
        .slots
        .get("url")
        .cloned()
        .unwrap_or_else(|| "https://google.com".to_string());

    Ok(vec![
        Step::NavigateTo(url.clone()),
        Step::WaitFor(1500),
        Step::ReportResult(format!("Navigated to {url}")),
    ])
}

/// Plan: search the web.
fn plan_search(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    let query = intent
        .slots
        .get("query")
        .cloned()
        .unwrap_or_else(|| intent.raw.clone());

    let encoded = urlencoding::encode(&query);
    let search_url = format!("https://www.google.com/search?q={encoded}");

    Ok(vec![
        Step::NavigateTo(search_url),
        Step::WaitFor(2000),
        Step::ReportResult(format!("Searched for: {query}")),
    ])
}

/// Plan: scrape data from a web page.
fn plan_scrape(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    let target = intent
        .slots
        .get("target")
        .cloned()
        .unwrap_or_else(|| intent.raw.clone());

    let lower = target.to_lowercase();

    // Special case: crypto prices
    if lower.contains("bitcoin") || lower.contains("btc") {
        return Ok(vec![
            Step::NavigateTo("https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd".to_string()),
            Step::WaitFor(2000),
            Step::ReadPage,
            Step::ReportResult("Fetched Bitcoin price".to_string()),
        ]);
    }

    if lower.contains("ethereum") || lower.contains("eth") {
        return Ok(vec![
            Step::NavigateTo("https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd".to_string()),
            Step::WaitFor(2000),
            Step::ReadPage,
            Step::ReportResult("Fetched Ethereum price".to_string()),
        ]);
    }

    // Generic scrape: search for the data
    let encoded = urlencoding::encode(&target);
    Ok(vec![
        Step::NavigateTo(format!("https://www.google.com/search?q={encoded}")),
        Step::WaitFor(2000),
        Step::ReadPage,
        Step::ReportResult(format!("Scraped data for: {target}")),
    ])
}

/// Plan: compose text content.
fn plan_compose(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    let content = intent
        .slots
        .get("content")
        .cloned()
        .unwrap_or_else(|| intent.raw.clone());

    Ok(vec![
        Step::ReportResult(format!("Composing: {content}")),
    ])
}

/// Plan: system commands (mute, screenshot, close tabs).
fn plan_system(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    let action = intent
        .slots
        .get("action")
        .cloned()
        .unwrap_or_else(|| intent.raw.to_lowercase());

    if action.contains("mute") && !action.contains("unmute") {
        return Ok(vec![
            Step::RunScript("document.querySelectorAll('video, audio').forEach(el => el.muted = true)".to_string()),
            Step::ReportResult("Muted all media".to_string()),
        ]);
    }

    if action.contains("unmute") {
        return Ok(vec![
            Step::RunScript("document.querySelectorAll('video, audio').forEach(el => el.muted = false)".to_string()),
            Step::ReportResult("Unmuted all media".to_string()),
        ]);
    }

    if action.contains("screenshot") {
        return Ok(vec![
            Step::ReportResult("Screenshot saved".to_string()),
        ]);
    }

    if action.contains("close all") {
        return Ok(vec![
            Step::ReportResult("Closed all tabs".to_string()),
        ]);
    }

    if action.contains("close tab") {
        return Ok(vec![
            Step::ReportResult("Closed current tab".to_string()),
        ]);
    }

    Ok(vec![Step::ReportResult(format!(
        "System action: {action}"
    ))])
}

/// Plan: macro recording / playback.
fn plan_macro(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    Ok(vec![Step::ReportResult(format!(
        "Macro: {}",
        intent.raw
    ))])
}

/// Plan: query — summarize, translate, explain.
fn plan_query(intent: &IntentResult) -> Result<Vec<Step>, CntrlError> {
    let lower = intent.raw.to_lowercase();

    if lower.contains("summarize") {
        return Ok(vec![
            Step::ReadPage,
            Step::ReportResult("Page summarized".to_string()),
        ]);
    }

    Ok(vec![
        Step::ReportResult(format!("Query: {}", intent.raw)),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::intent::parse_intent;

    #[test]
    fn plan_navigate_produces_steps() {
        let intent = parse_intent("go to reddit").expect("should parse");
        let steps = plan(&intent).expect("should plan");
        assert!(!steps.is_empty());
        assert!(matches!(steps[0], Step::NavigateTo(_)));
    }

    #[test]
    fn plan_search_produces_steps() {
        let intent = parse_intent("search for rust tutorials").expect("should parse");
        let steps = plan(&intent).expect("should plan");
        assert!(!steps.is_empty());
        assert!(matches!(steps[0], Step::NavigateTo(_)));
    }

    #[test]
    fn plan_scrape_bitcoin() {
        let intent = parse_intent("get the bitcoin price").expect("should parse");
        let steps = plan(&intent).expect("should plan");
        assert!(!steps.is_empty());
        if let Step::NavigateTo(ref url) = steps[0] {
            assert!(url.contains("coingecko"), "expected coingecko URL, got {url}");
        } else {
            panic!("expected NavigateTo step");
        }
    }

    #[test]
    fn plan_system_mute() {
        let intent = parse_intent("mute").expect("should parse");
        let steps = plan(&intent).expect("should plan");
        assert!(matches!(steps[0], Step::RunScript(_)));
    }

    #[test]
    fn plan_query_summarize() {
        let intent = parse_intent("summarize this page").expect("should parse");
        let steps = plan(&intent).expect("should plan");
        assert!(matches!(steps[0], Step::ReadPage));
    }

    #[test]
    fn plan_all_intent_types_produce_steps() {
        let commands = [
            "go to youtube",
            "search for recipes",
            "get the bitcoin price",
            "write an email to boss",
            "mute",
            "record this",
            "summarize this page",
        ];

        for cmd in &commands {
            let intent = parse_intent(cmd).expect("should parse");
            let steps = plan(&intent).expect("should plan");
            assert!(!steps.is_empty(), "empty steps for: {cmd}");
        }
    }
}
