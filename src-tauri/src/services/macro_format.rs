// services/macro_format.rs
//
// ─────────────────────────────────────────────────────────────────────────────
// .vibe  Macro File Format — Schema & Versioning
// ─────────────────────────────────────────────────────────────────────────────
//
// A `.vibe` file is a UTF-8 JSON document with the following top-level shape:
//
// {
//   "vibe_version":  "1",          // format version, bumped on breaking changes
//   "id":            "<uuid-v4>",  // stable identifier; never changes after save
//   "name":          "string",     // human-readable title
//   "description":   "string",     // optional longer description
//   "author":        "string",     // free-form author field
//   "created_at":    "<rfc3339>",  // ISO-8601 creation timestamp
//   "updated_at":    "<rfc3339>",  // ISO-8601 last-updated timestamp
//   "steps": [                     // ordered list of recorded intent strings
//     {
//       "intent":  "go to github.com",   // raw intent text as typed by user
//       "delay_ms": 0                    // optional pause before this step (ms)
//     }
//   ],
//   "triggers": [                  // zero or more automatic triggers
//     {
//       "kind":   "cron",          // "cron" | "manual"
//       "cron":   "0 9 * * 1-5"   // standard 5-field cron (only if kind=cron)
//     }
//   ]
// }
//
// Version history:
//   v1 (2026-07-16): initial format; steps as intent strings with delay_ms
//
// Extension policy: non-breaking additions (new optional fields) keep v1.
// Breaking changes (removing/renaming required fields) MUST bump vibe_version.
// ─────────────────────────────────────────────────────────────────────────────

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::CntrlError;

/// Current format version string embedded in every .vibe file.
pub const VIBE_FORMAT_VERSION: &str = "1";

/// A single recorded step inside a macro.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroStep {
    /// Raw intent string exactly as the user typed it (e.g. "go to github.com").
    pub intent: String,
    /// Optional pause before executing this step, in milliseconds.
    #[serde(default)]
    pub delay_ms: u64,
}

/// How a macro can be triggered automatically.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MacroTrigger {
    /// Manually triggered only — no automatic schedule.
    Manual,
    /// Triggered by a standard 5-field cron expression.
    Cron {
        /// e.g. `"0 9 * * 1-5"` (09:00 Mon–Fri)
        cron: String,
    },
}

/// The complete in-memory representation of a `.vibe` macro.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vibemacro {
    /// Format version — always `"1"` in the current release.
    pub vibe_version: String,
    /// Stable UUID identifier for this macro.
    pub id: String,
    /// Human-readable title shown in the MacroLibrary UI.
    pub name: String,
    /// Optional longer description / purpose.
    #[serde(default)]
    pub description: String,
    /// Free-form author string.
    #[serde(default)]
    pub author: String,
    /// RFC-3339 creation timestamp.
    pub created_at: DateTime<Utc>,
    /// RFC-3339 last-modified timestamp.
    pub updated_at: DateTime<Utc>,
    /// Ordered sequence of steps to execute.
    pub steps: Vec<MacroStep>,
    /// Zero or more automatic trigger definitions.
    #[serde(default)]
    pub triggers: Vec<MacroTrigger>,
}

impl Vibemacro {
    /// Create a new, unsaved macro with the given name and steps.
    pub fn new(name: impl Into<String>, steps: Vec<MacroStep>) -> Self {
        let now = Utc::now();
        Self {
            vibe_version: VIBE_FORMAT_VERSION.to_string(),
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            author: String::new(),
            created_at: now,
            updated_at: now,
            steps,
            triggers: Vec::new(),
        }
    }

    /// Serialize this macro to a pretty-printed JSON string.
    pub fn to_json(&self) -> Result<String, CntrlError> {
        serde_json::to_string_pretty(self)
            .map_err(|e| CntrlError::Macro(format!("Failed to serialize macro: {e}")))
    }

    /// Deserialize a `.vibe` JSON string into a [`Vibemacro`].
    pub fn from_json(json: &str) -> Result<Self, CntrlError> {
        let m: Vibemacro = serde_json::from_str(json)
            .map_err(|e| CntrlError::Macro(format!("Failed to parse .vibe file: {e}")))?;

        if m.vibe_version != VIBE_FORMAT_VERSION {
            return Err(CntrlError::Macro(format!(
                "Unsupported .vibe format version '{}' (expected '{}')",
                m.vibe_version, VIBE_FORMAT_VERSION
            )));
        }
        Ok(m)
    }

    /// Safe filename for this macro (alphanumeric + dashes, no spaces).
    pub fn safe_filename(&self) -> String {
        let slug: String = self
            .name
            .chars()
            .map(|c| {
                if c.is_ascii_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        format!("{}-{}.vibe", slug, &self.id[..8])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vibemacro {
        Vibemacro::new(
            "Morning Routine",
            vec![
                MacroStep {
                    intent: "go to gmail.com".to_string(),
                    delay_ms: 0,
                },
                MacroStep {
                    intent: "go to calendar.google.com".to_string(),
                    delay_ms: 500,
                },
            ],
        )
    }

    #[test]
    fn round_trip_json() {
        let m = sample();
        let json = m.to_json().expect("serialise should succeed");
        let parsed = Vibemacro::from_json(&json).expect("parse should succeed");
        assert_eq!(parsed.name, m.name);
        assert_eq!(parsed.steps.len(), m.steps.len());
        assert_eq!(parsed.steps[0].intent, "go to gmail.com");
        assert_eq!(parsed.steps[1].delay_ms, 500);
        assert_eq!(parsed.id, m.id);
    }

    #[test]
    fn wrong_version_is_rejected() {
        let m = sample();
        let mut json: serde_json::Value = serde_json::from_str(&m.to_json().unwrap()).unwrap();
        json["vibe_version"] = serde_json::json!("99");
        let bad_json = json.to_string();
        assert!(Vibemacro::from_json(&bad_json).is_err());
    }

    #[test]
    fn safe_filename_slugifies() {
        let m = sample();
        let f = m.safe_filename();
        assert!(f.ends_with(".vibe"));
        assert!(f.starts_with("morning-routine-"));
        assert!(!f.contains(' '));
    }

    #[test]
    fn trigger_cron_round_trips() {
        let mut m = sample();
        m.triggers.push(MacroTrigger::Cron {
            cron: "0 9 * * 1-5".to_string(),
        });
        let json = m.to_json().unwrap();
        let parsed = Vibemacro::from_json(&json).unwrap();
        if let MacroTrigger::Cron { cron } = &parsed.triggers[0] {
            assert_eq!(cron, "0 9 * * 1-5");
        } else {
            panic!("Expected Cron trigger");
        }
    }
}
