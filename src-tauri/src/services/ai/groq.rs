use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{CompletionRequest, CompletionResponse, Provider, Tier};
use crate::error::CntrlError;
use crate::services::keychain;

const GROQ_API_BASE: &str = "https://api.groq.com/openai/v1/chat/completions";
const GROQ_DEFAULT_MODEL: &str = "llama3-8b-8192";

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct GroqRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
}

#[derive(Deserialize)]
struct GroqUsage {
    total_tokens: Option<u32>,
}

#[derive(Deserialize)]
struct GroqChoice {
    message: GroqMessage,
}

#[derive(Deserialize)]
struct GroqMessage {
    content: String,
}

#[derive(Deserialize)]
struct GroqResponse {
    choices: Vec<GroqChoice>,
    usage: Option<GroqUsage>,
}

pub struct GroqProvider {
    client: Client,
    model: String,
}

impl GroqProvider {
    #[must_use]
    pub fn new(model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            model: model.unwrap_or_else(|| GROQ_DEFAULT_MODEL.to_string()),
        }
    }
}

#[async_trait]
impl Provider for GroqProvider {
    fn name(&self) -> &str {
        "Groq"
    }

    fn tier(&self) -> Tier {
        Tier::Freemium
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse, CntrlError> {
        let api_key = keychain::retrieve_secret(keychain::KEY_GROQ)?;

        let mut messages = vec![];
        if let Some(system) = &req.system {
            messages.push(ChatMessage {
                role: "system",
                content: system.as_str(),
            });
        }
        messages.push(ChatMessage {
            role: "user",
            content: &req.prompt,
        });

        let body = GroqRequest {
            model: &self.model,
            messages,
        };

        let res = self
            .client
            .post(GROQ_API_BASE)
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await
            .map_err(|e| CntrlError::Ai(format!("Groq connection error: {e}")))?;

        if !res.status().is_success() {
            let status = res.status();
            let err_text = res.text().await.unwrap_or_default();
            return Err(CntrlError::Ai(format!(
                "Groq API error {status}: {err_text}"
            )));
        }

        let data: GroqResponse = res
            .json()
            .await
            .map_err(|e| CntrlError::Ai(format!("Groq JSON parse error: {e}")))?;

        let text = data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .unwrap_or_default();

        if text.is_empty() {
            return Err(CntrlError::Ai(
                "Groq returned an empty response".to_string(),
            ));
        }

        let tokens_used = data.usage.and_then(|u| u.total_tokens);

        Ok(CompletionResponse {
            text,
            tokens_used,
            provider: self.name().to_string(),
        })
    }

    async fn health_check(&self) -> bool {
        keychain::secret_exists(keychain::KEY_GROQ)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_metadata() {
        let p = GroqProvider::new(None);
        assert_eq!(p.name(), "Groq");
        assert_eq!(p.tier(), Tier::Freemium);
        assert_eq!(p.model, GROQ_DEFAULT_MODEL);
    }

    #[test]
    fn provider_metadata_custom_model() {
        let p = GroqProvider::new(Some("mixtral-8x7b-32768".to_string()));
        assert_eq!(p.model, "mixtral-8x7b-32768");
    }

    #[test]
    fn request_body_user_only() {
        let messages = vec![ChatMessage {
            role: "user",
            content: "Hello",
        }];
        let req = GroqRequest {
            model: GROQ_DEFAULT_MODEL,
            messages,
        };
        let json = serde_json::to_value(&req).expect("should serialise");
        assert_eq!(json["model"], GROQ_DEFAULT_MODEL);
        assert_eq!(json["messages"][0]["role"], "user");
        assert_eq!(json["messages"][0]["content"], "Hello");
    }

    #[test]
    fn request_body_with_system_message() {
        let messages = vec![
            ChatMessage {
                role: "system",
                content: "Be brief.",
            },
            ChatMessage {
                role: "user",
                content: "Hi",
            },
        ];
        let req = GroqRequest {
            model: GROQ_DEFAULT_MODEL,
            messages,
        };
        let json = serde_json::to_value(&req).expect("should serialise");
        assert_eq!(json["messages"][0]["role"], "system");
        assert_eq!(json["messages"][1]["role"], "user");
    }
}
