use cntrl_browser_lib::services::intent::{IntentResult, IntentType};
use cntrl_browser_lib::services::planner::{Planner, Step};

#[test]
fn test_navigation_intents() {
    let intents = [
        "go to google.com",
        "navigate to https://github.com",
        "open localhost:3000",
    ];

    for input in intents {
        let result = IntentResult::parse(input);
        assert_eq!(result.intent_type, IntentType::Navigation);
        assert!(result.parameters.contains_key("url"));
        
        let plan = Planner::plan(result);
        assert_eq!(plan.len(), 2);
        assert!(matches!(plan[1], Step::Navigate { .. }));
    }
}

#[test]
fn test_search_intents() {
    let intents = [
        "search for rust programming",
        "google best solidjs tutorials",
    ];

    for input in intents {
        let result = IntentResult::parse(input);
        assert_eq!(result.intent_type, IntentType::Search);
        assert!(result.parameters.contains_key("query"));
        
        let plan = Planner::plan(result);
        assert_eq!(plan.len(), 2);
        assert!(matches!(plan[1], Step::Navigate { .. }));
    }
}

#[test]
fn test_system_command_intents() {
    let commands = [
        ("bitcoin price", "bitcoin_price"),
        ("btc price", "bitcoin_price"),
        ("take screenshot", "screenshot"),
        ("screenshot", "screenshot"),
        ("mute volume", "mute"),
        ("mute", "mute"),
    ];

    for (input, expected_cmd) in commands {
        let result = IntentResult::parse(input);
        assert_eq!(result.intent_type, IntentType::SystemCommand);
        assert_eq!(result.parameters.get("command").unwrap(), expected_cmd);
        
        let plan = Planner::plan(result);
        assert_eq!(plan.len(), 1);
        if let Step::BuiltinCommand { command } = &plan[0] {
            assert_eq!(command, expected_cmd);
        } else {
            panic!("Expected BuiltinCommand");
        }
    }
}

#[test]
fn test_ai_query_intents() {
    let intents = [
        "what is the capital of France?",
        "write a python script to parse json",
        "how does quantum computing work",
        "summarize this article",
    ];

    for input in intents {
        let result = IntentResult::parse(input);
        assert_eq!(result.intent_type, IntentType::AiQuery);
        assert!(result.parameters.contains_key("query"));
        
        let plan = Planner::plan(result);
        assert_eq!(plan.len(), 1);
        assert!(matches!(plan[0], Step::AiQuery { .. }));
    }
}

#[test]
fn test_settings_intents() {
    let intents = [
        "settings",
        "open settings",
    ];

    for input in intents {
        let result = IntentResult::parse(input);
        assert_eq!(result.intent_type, IntentType::SettingsAction);
        
        let plan = Planner::plan(result);
        assert_eq!(plan.len(), 2);
        if let Step::Navigate { url } = &plan[1] {
            assert_eq!(url, "cntrl://settings");
        } else {
            panic!("Expected Navigate");
        }
    }
}

#[test]
fn test_macro_trigger_intents() {
    let intents = [
        "trigger macro morning_routine",
        "run macro build_project",
    ];

    for input in intents {
        let result = IntentResult::parse(input);
        assert_eq!(result.intent_type, IntentType::MacroTrigger);
        assert!(result.parameters.contains_key("macro_id"));
        
        let plan = Planner::plan(result);
        assert_eq!(plan.len(), 1);
        assert!(matches!(plan[0], Step::DisplayResult { .. }));
    }
}

#[test]
fn test_unknown_fallback_intents() {
    // We didn't define specific patterns for UnknownFallback in our naive parser because everything falls back to AiQuery by default!
    // But let's test if we can manually construct it and plan it, or if our parser can be forced into it.
    // Actually, in our parse logic, UnknownFallback is never returned currently, it returns AiQuery.
    // We should fix the parse logic to actually return UnknownFallback for empty strings or gibberish.
    // Wait, the prompt says "unknown/fallback". In our parse, everything unknown is an AI query.
    // Let's create an explicit parser fallback for empty input.
    let intents = [
        "",
        "   ",
    ];

    for input in intents {
        let result = IntentResult::parse(input);
        // We need to update mod.rs to return UnknownFallback for empty strings
        assert_eq!(result.intent_type, IntentType::UnknownFallback);
        
        let plan = Planner::plan(result);
        assert_eq!(plan.len(), 1);
        assert!(matches!(plan[0], Step::DisplayResult { .. }));
    }
}
