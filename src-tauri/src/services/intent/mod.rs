use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The 7 core intent categories supported by the Vibe Browser.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentType {
    Navigation,
    Search,
    SystemCommand,
    AiQuery,
    MacroTrigger,
    SettingsAction,
    UnknownFallback,
}

/// The result of parsing a natural language command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub intent_type: IntentType,
    pub confidence: u8,
    pub parameters: HashMap<String, String>,
}

impl IntentResult {
    /// A naive natural language parser for Phase 4.
    /// In a production system, this would be backed by an LLM or a sophisticated NLU model.
    pub fn parse(input: &str) -> Self {
        let input_lower = input.trim().to_lowercase();
        let mut parameters = HashMap::new();

        if input_lower.is_empty() {
            return IntentResult {
                intent_type: IntentType::UnknownFallback,
                confidence: 100,
                parameters,
            };
        }

        if input_lower.starts_with("go to ")
            || input_lower.starts_with("navigate to ")
            || input_lower.starts_with("open ")
        {
            let target = input_lower
                .replace("go to ", "")
                .replace("navigate to ", "")
                .replace("open ", "");
            // exception for "open settings"
            if target.trim() == "settings" {
                parameters.insert("action".to_string(), "open".to_string());
                return IntentResult {
                    intent_type: IntentType::SettingsAction,
                    confidence: 90,
                    parameters,
                };
            }
            parameters.insert("url".to_string(), target.trim().to_string());
            return IntentResult {
                intent_type: IntentType::Navigation,
                confidence: 90,
                parameters,
            };
        }

        if input_lower.starts_with("search for ") || input_lower.starts_with("google ") {
            let target = input_lower
                .replace("search for ", "")
                .replace("google ", "");
            parameters.insert("query".to_string(), target.trim().to_string());
            return IntentResult {
                intent_type: IntentType::Search,
                confidence: 90,
                parameters,
            };
        }

        if input_lower.starts_with("bitcoin price")
            || input_lower.starts_with("btc price")
            || input_lower.starts_with("price of bitcoin")
        {
            parameters.insert("command".to_string(), "bitcoin_price".to_string());
            return IntentResult {
                intent_type: IntentType::SystemCommand,
                confidence: 95,
                parameters,
            };
        }

        if input_lower.starts_with("take screenshot") || input_lower.starts_with("screenshot") {
            parameters.insert("command".to_string(), "screenshot".to_string());
            return IntentResult {
                intent_type: IntentType::SystemCommand,
                confidence: 95,
                parameters,
            };
        }

        if input_lower.starts_with("mute volume") || input_lower.starts_with("mute") {
            parameters.insert("command".to_string(), "mute".to_string());
            return IntentResult {
                intent_type: IntentType::SystemCommand,
                confidence: 95,
                parameters,
            };
        }

        if input_lower.starts_with("settings") {
            parameters.insert("action".to_string(), "open".to_string());
            return IntentResult {
                intent_type: IntentType::SettingsAction,
                confidence: 90,
                parameters,
            };
        }

        if input_lower.starts_with("trigger macro") || input_lower.starts_with("run macro") {
            let target = input_lower
                .replace("trigger macro ", "")
                .replace("run macro ", "");
            parameters.insert("macro_id".to_string(), target.trim().to_string());
            return IntentResult {
                intent_type: IntentType::MacroTrigger,
                confidence: 90,
                parameters,
            };
        }

        // By default, treat it as an AI query if it doesn't match known structures
        parameters.insert("query".to_string(), input.to_string());
        IntentResult {
            intent_type: IntentType::AiQuery,
            confidence: 70,
            parameters,
        }
    }
}
