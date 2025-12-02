use serde::{Deserialize, Serialize};

use reqwest::blocking::Client;
use std::sync::mpsc;
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

pub enum AIResponse {
    Success(String),
    Error(String),
}

pub struct AIClient {
    api_key: String,
    client: Client,
}

impl AIClient {
    pub fn new() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        Some(Self {
            api_key,
            client: Client::new(),
        })
    }

    pub fn send_message(
        &self,
        history: Vec<ChatMessage>,
        context: String,
        sender: mpsc::Sender<AIResponse>,
    ) {
        let api_key = self.api_key.clone();
        let client = self.client.clone();

        thread::spawn(move || {
            let mut modified_history = history.clone();
            if let Some(last_msg) = modified_history.last_mut() {
                if last_msg.role == "user" && last_msg.content.trim().eq_ignore_ascii_case("quote")
                {
                    last_msg.content =
                        format!("{} [System ID: {}]", last_msg.content, uuid::Uuid::new_v4());
                }
            }

            let mut messages = vec![ChatMessage {
                role: "system".to_string(),
                content: format!(
                    r#"You are a productivity coach in an Eisenhower Matrix task manager.

**YOUR ROLE:**
- Break down vague goals into concrete, actionable tasks
- Suggest urgency (1-3) and importance (1-3) ratings
- Challenge low-value tasks — should they be dropped or delegated?
- Keep responses brief and actionable

**TASK FORMAT:**
When suggesting tasks:
[ADD] Task name u<urgency>i<importance>

Examples:
[ADD] Draft proposal outline u2i3
[ADD] Schedule dentist u1i2

**QUADRANT GUIDE:**
- Q1 (DO FIRST): Due today/tomorrow, blocks other work
- Q2 (SCHEDULE): Important but not urgent — protect this time
- Q3 (DELEGATE): Urgent but not important — can someone else do it?
- Q4 (DROP): Neither — question why you're doing it

**QUOTE COMMAND:**
If user says "quote", respond with ONE inspirational quote in English, Japanese, or Chinese. Just the quote and author, nothing else.

**CURRENT TASKS:**
{}

Be concise. No fluff."#,
                    context
                ),
            }];
            messages.extend(modified_history);

            let body = serde_json::json!({
                "model": "gpt-4o",
                "temperature": 0.7,
                "presence_penalty": 0.3,
                "max_tokens": 500,
                "messages": messages,
            });

            let res = client
                .post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send();

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(json) = response.json::<serde_json::Value>() {
                            if let Some(content) = json["choices"][0]["message"]["content"].as_str()
                            {
                                let _ = sender.send(AIResponse::Success(content.to_string()));
                                return;
                            }
                        }
                        let _ = sender.send(AIResponse::Error(
                            "Failed to parse API response".to_string(),
                        ));
                    } else {
                        let _ = sender.send(AIResponse::Error(format!(
                            "API Error: {}",
                            response.status()
                        )));
                    }
                }
                Err(e) => {
                    let _ = sender.send(AIResponse::Error(format!("Network Error: {}", e)));
                }
            }
        });
    }
}
