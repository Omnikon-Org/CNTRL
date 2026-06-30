//! Intent parser — classifies natural language input into structured intents.
//!
//! Uses regex + keyword matching as the primary parser. Optionally calls
//! the AI router (Tier 1 preferred) for disambiguation when confidence is low.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::error::CntrlError;

// ── Intent taxonomy ────────────────────────────────────────────────────

/// The seven intent categories for every user command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentType {
    /// "go to reddit", "open youtube"
    Navigate,
    /// "search for best lo-fi playlists"
    Search,
    /// "get the bitcoin price", "what's trending on HN?"
    Scrape,
    /// "write an email to...", "draft a reply to..."
    Compose,
    /// "mute", "close all tabs", "take screenshot"
    System,
    /// "record this", "run my morning routine"
    Macro,
    /// "summarize this page", "translate this"
    Query,
}

/// The result of parsing a natural language command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    /// Unique identifier for this intent.
    pub id: Uuid,
    /// Classified intent type.
    pub intent_type: IntentType,
    /// Confidence of classification (0.0 – 1.0).
    pub confidence: f32,
    /// Extracted slots: key-value pairs like `url`, `query`, `selector`.
    pub slots: HashMap<String, String>,
    /// Original raw input from the user.
    pub raw: String,
}

// ── Keyword patterns ───────────────────────────────────────────────────

/// A pattern entry: keywords and the intent they map to.
struct Pattern {
    keywords: &'static [&'static str],
    intent: IntentType,
    confidence: f32,
}

/// All keyword patterns ordered by priority (first match wins within group).
const PATTERNS: &[Pattern] = &[
    // System commands — highest priority, most specific
    Pattern {
        keywords: &["mute", "unmute"],
        intent: IntentType::System,
        confidence: 0.95,
    },
    Pattern {
        keywords: &["screenshot", "screen shot", "screen capture"],
        intent: IntentType::System,
        confidence: 0.95,
    },
    Pattern {
        keywords: &["close tab", "close all tab", "close all"],
        intent: IntentType::System,
        confidence: 0.95,
    },
    Pattern {
        keywords: &["new tab"],
        intent: IntentType::System,
        confidence: 0.90,
    },
    // Macro commands
    Pattern {
        keywords: &["record", "run macro", "morning routine", "start recording"],
        intent: IntentType::Macro,
        confidence: 0.85,
    },
    // Navigate — URL-like patterns
    Pattern {
        keywords: &["go to", "open", "navigate to", "visit"],
        intent: IntentType::Navigate,
        confidence: 0.85,
    },
    // Search
    Pattern {
        keywords: &["search for", "search", "find me", "look up", "google"],
        intent: IntentType::Search,
        confidence: 0.85,
    },
    // Scrape — data extraction
    Pattern {
        keywords: &[
            "get the", "what is the price", "what's the price",
            "price of", "trending on", "scrape", "extract",
            "what's trending",
        ],
        intent: IntentType::Scrape,
        confidence: 0.80,
    },
    // Compose — content creation
    Pattern {
        keywords: &[
            "write", "draft", "compose", "email to", "reply to",
            "create a post",
        ],
        intent: IntentType::Compose,
        confidence: 0.80,
    },
    // Query — analysis / transformation
    Pattern {
        keywords: &[
            "summarize", "translate", "explain", "what does",
            "how does", "tell me about", "describe",
        ],
        intent: IntentType::Query,
        confidence: 0.80,
    },
];

// ── Parser implementation ──────────────────────────────────────────────

/// Parse a natural language input into a structured intent.
///
/// Uses keyword matching with slot extraction. Returns an `IntentResult`
/// with the best-matching intent type and extracted parameters.
///
/// # Arguments
/// * `input` — The raw natural language command from the user.
///
/// # Returns
/// An `IntentResult` with the classified intent and extracted slots.
pub fn parse_intent(input: &str) -> Result<IntentResult, CntrlError> {
    let lower = input.to_lowercase().trim().to_string();

    if lower.is_empty() {
        return Err(CntrlError::Ai("Empty input".into()));
    }

    let mut best_intent = IntentType::Query; // Default fallback
    let mut best_confidence: f32 = 0.3;
    let mut slots = HashMap::new();

    // Check each pattern
    for pattern in PATTERNS {
        for kw in pattern.keywords {
            if lower.contains(kw) {
                if pattern.confidence > best_confidence {
                    best_confidence = pattern.confidence;
                    best_intent = pattern.intent;
                }
                break; // Don't double-count keywords in the same pattern
            }
        }
    }

    // Extract slots based on intent type
    match best_intent {
        IntentType::Navigate => {
            // Try to extract a URL or site name
            let url = extract_url_or_site(&lower);
            if let Some(u) = url {
                slots.insert("url".to_string(), u);
            }
        }
        IntentType::Search => {
            // Extract the search query
            let query = extract_after_keywords(&lower, &["search for", "search", "find me", "look up", "google"]);
            if let Some(q) = query {
                slots.insert("query".to_string(), q);
            }
        }
        IntentType::Scrape => {
            // Extract what to scrape
            let target = extract_after_keywords(&lower, &["get the", "price of", "what is the", "what's the"]);
            if let Some(t) = target {
                slots.insert("target".to_string(), t);
            }
        }
        IntentType::Compose => {
            // Extract the composition target
            let target = extract_after_keywords(&lower, &["write", "draft", "compose", "email to", "reply to"]);
            if let Some(t) = target {
                slots.insert("content".to_string(), t);
            }
        }
        IntentType::Query => {
            // The entire input is the query
            slots.insert("query".to_string(), input.to_string());
        }
        IntentType::System | IntentType::Macro => {
            // System commands are self-contained; store the action
            slots.insert("action".to_string(), lower.clone());
        }
    }

    Ok(IntentResult {
        id: Uuid::new_v4(),
        intent_type: best_intent,
        confidence: best_confidence,
        slots,
        raw: input.to_string(),
    })
}

/// Try to extract a URL or well-known site name from the input.
fn extract_url_or_site(input: &str) -> Option<String> {
    // Check for explicit URLs
    for word in input.split_whitespace() {
        if word.contains("http://") || word.contains("https://") || word.contains("www.") {
            return Some(word.to_string());
        }
        if word.contains('.') && !word.ends_with('.') && word.len() > 3 {
            return Some(format!("https://{word}"));
        }
    }

    // Well-known site aliases
    let aliases: &[(&str, &str)] = &[
        ("youtube", "https://youtube.com"),
        ("reddit", "https://reddit.com"),
        ("twitter", "https://twitter.com"),
        ("x.com", "https://x.com"),
        ("github", "https://github.com"),
        ("hacker news", "https://news.ycombinator.com"),
        ("hn", "https://news.ycombinator.com"),
        ("google", "https://google.com"),
        ("gmail", "https://mail.google.com"),
        ("wikipedia", "https://wikipedia.org"),
    ];

    for (alias, url) in aliases {
        if input.contains(alias) {
            return Some(url.to_string());
        }
    }

    None
}

/// Extract text that comes after any of the given keywords.
fn extract_after_keywords(input: &str, keywords: &[&str]) -> Option<String> {
    for kw in keywords {
        if let Some(pos) = input.find(kw) {
            let after = input[pos + kw.len()..].trim();
            if !after.is_empty() {
                return Some(after.to_string());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_navigate_intent() {
        let result = parse_intent("go to reddit").expect("should parse");
        assert_eq!(result.intent_type, IntentType::Navigate);
        assert_eq!(result.slots.get("url").map(String::as_str), Some("https://reddit.com"));
    }

    #[test]
    fn parse_search_intent() {
        let result = parse_intent("search for best lo-fi playlists").expect("should parse");
        assert_eq!(result.intent_type, IntentType::Search);
        assert!(result.slots.contains_key("query"));
    }

    #[test]
    fn parse_scrape_intent() {
        let result = parse_intent("get the bitcoin price").expect("should parse");
        assert_eq!(result.intent_type, IntentType::Scrape);
        assert!(result.slots.contains_key("target"));
    }

    #[test]
    fn parse_system_mute() {
        let result = parse_intent("mute").expect("should parse");
        assert_eq!(result.intent_type, IntentType::System);
    }

    #[test]
    fn parse_system_screenshot() {
        let result = parse_intent("take a screenshot").expect("should parse");
        assert_eq!(result.intent_type, IntentType::System);
    }

    #[test]
    fn parse_compose_intent() {
        let result = parse_intent("write an email to my professor").expect("should parse");
        assert_eq!(result.intent_type, IntentType::Compose);
    }

    #[test]
    fn parse_query_intent() {
        let result = parse_intent("summarize this page").expect("should parse");
        assert_eq!(result.intent_type, IntentType::Query);
    }

    #[test]
    fn parse_macro_intent() {
        let result = parse_intent("record this workflow").expect("should parse");
        assert_eq!(result.intent_type, IntentType::Macro);
    }

    #[test]
    fn parse_empty_input() {
        let result = parse_intent("");
        assert!(result.is_err());
    }

    #[test]
    fn parse_navigate_with_url() {
        let result = parse_intent("go to https://example.com").expect("should parse");
        assert_eq!(result.intent_type, IntentType::Navigate);
        assert_eq!(
            result.slots.get("url").map(String::as_str),
            Some("https://example.com")
        );
    }

    #[test]
    fn parse_20_commands_at_least_18_correct() {
        let commands = vec![
            ("go to youtube", IntentType::Navigate),
            ("open reddit", IntentType::Navigate),
            ("search for rust tutorials", IntentType::Search),
            ("find me a recipe", IntentType::Search),
            ("get the bitcoin price", IntentType::Scrape),
            ("what's the price of ethereum", IntentType::Scrape),
            ("write an email to john", IntentType::Compose),
            ("draft a reply to the customer", IntentType::Compose),
            ("mute", IntentType::System),
            ("take a screenshot", IntentType::System),
            ("close all tabs", IntentType::System),
            ("unmute", IntentType::System),
            ("summarize this page", IntentType::Query),
            ("translate this to Spanish", IntentType::Query),
            ("explain how DNS works", IntentType::Query),
            ("record this workflow", IntentType::Macro),
            ("search for best coffee shops", IntentType::Search),
            ("navigate to github.com", IntentType::Navigate),
            ("what does this function do", IntentType::Query),
            ("look up the weather", IntentType::Search),
        ];

        let mut correct = 0;
        for (cmd, expected) in &commands {
            let result = parse_intent(cmd).expect("should parse");
            if result.intent_type == *expected {
                correct += 1;
            }
        }

        assert!(
            correct >= 18,
            "Only {correct}/20 commands matched (need ≥ 18)"
        );
    }
}
