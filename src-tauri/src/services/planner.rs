use serde::{Deserialize, Serialize};
use super::intent::{IntentResult, IntentType};

/// Represents a discrete action emitted by the planner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Step {
    Navigate { url: String },
    AiQuery { prompt: String },
    BuiltinCommand { command: String },
    DisplayResult { markdown: String },
}

pub struct Planner;

impl Planner {
    /// Takes an IntentResult and produces an ordered execution plan (sequence of Steps).
    pub fn plan(intent: IntentResult) -> Vec<Step> {
        let mut steps = Vec::new();

        match intent.intent_type {
            IntentType::Navigation => {
                if let Some(url) = intent.parameters.get("url") {
                    steps.push(Step::DisplayResult {
                        markdown: format!("Navigating to **{}**...", url),
                    });
                    steps.push(Step::Navigate { url: url.clone() });
                }
            }
            IntentType::Search => {
                if let Some(query) = intent.parameters.get("query") {
                    let encoded = urlencoding::encode(query);
                    let url = format!("https://google.com/search?q={}", encoded);
                    steps.push(Step::DisplayResult {
                        markdown: format!("Searching for **{}**...", query),
                    });
                    steps.push(Step::Navigate { url });
                }
            }
            IntentType::SystemCommand => {
                if let Some(cmd) = intent.parameters.get("command") {
                    steps.push(Step::BuiltinCommand { command: cmd.clone() });
                }
            }
            IntentType::AiQuery => {
                if let Some(query) = intent.parameters.get("query") {
                    steps.push(Step::AiQuery { prompt: query.clone() });
                }
            }
            IntentType::SettingsAction => {
                steps.push(Step::DisplayResult {
                    markdown: "Opening settings...".to_string(),
                });
                steps.push(Step::Navigate {
                    url: "cntrl://settings".to_string(),
                });
            }
            IntentType::MacroTrigger => {
                if let Some(macro_id) = intent.parameters.get("macro_id") {
                    steps.push(Step::DisplayResult {
                        markdown: format!("*Triggering macro: **{}***\n\n> Note: Macro execution is part of Phase 6.", macro_id),
                    });
                }
            }
            IntentType::UnknownFallback => {
                if let Some(query) = intent.parameters.get("query") {
                    steps.push(Step::DisplayResult {
                        markdown: format!("Unknown intent for: {}", query),
                    });
                } else {
                    steps.push(Step::DisplayResult {
                        markdown: "Unknown intent.".to_string(),
                    });
                }
            }
        }

        steps
    }
}
