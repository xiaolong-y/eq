use serde::{Deserialize, Serialize};

use std::sync::mpsc;
use std::thread;
use reqwest::blocking::Client;

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

    pub fn send_message(&self, history: Vec<ChatMessage>, context: String, sender: mpsc::Sender<AIResponse>) {
        let api_key = self.api_key.clone();
        let client = self.client.clone();
        
        thread::spawn(move || {
            let mut modified_history = history.clone();
            if let Some(last_msg) = modified_history.last_mut() {
                if last_msg.role == "user" && last_msg.content.trim().eq_ignore_ascii_case("quote") {
                    last_msg.content = format!("{} [System ID: {}]", last_msg.content, uuid::Uuid::new_v4());
                }
            }

            let mut messages = vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: format!("You are a fast, practical productivity assistant embedded in an Eisenhower Matrix task manager. \
Your goal is to prioritize speed, clarity, and usefulness.

**CORE BEHAVIORS:**
1. Give concise, text-only answers.
2. Transform vague ideas into concrete tasks for the Eisenhower Matrix.
3. Maintain a friendly, efficient tone.

**TASK CREATION:**
If the user asks to add a task or mentions something they need to do, respond with a task suggestion in this format:
[ADD] Task name u<urgency>i<importance>
For example: [ADD] Review quarterly report u2i3

**SPECIAL INSTRUCTION: QUOTE GENERATION**
If the user inputs the single word \"quote\" (or variations like \"give me a quote\"), you MUST follow this strict process:
1. **Randomly select ONE language** from this list: [English, Japanese, Chinese].
2. **Select a unique, distinct inspirational quote** from a credible source. (Do not use the same famous quotes repeatedly; aim for variety).
3. **Output ONLY the quote and the author** in that SINGLE selected language.
4. **DO NOT** provide translations.
5. **DO NOT** explain your choice.

**Output Format for Quotes:**
\"[Quote text]\" — [Author Name]

The user's current task list context is: {}", context),
                }
            ];
            messages.extend(modified_history);

            let body = serde_json::json!({
                "model": "gpt-4o",
                "temperature": 1.1,
                "presence_penalty": 0.5,
                "max_tokens": 150,
                "messages": messages,
            });

            let res = client.post("https://api.openai.com/v1/chat/completions")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&body)
                .send();

            match res {
                Ok(response) => {
                    if response.status().is_success() {
                        if let Ok(json) = response.json::<serde_json::Value>() {
                            if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                                let _ = sender.send(AIResponse::Success(content.to_string()));
                                return;
                            }
                        }
                        let _ = sender.send(AIResponse::Error("Failed to parse API response".to_string()));
                    } else {
                        let _ = sender.send(AIResponse::Error(format!("API Error: {}", response.status())));
                    }
                }
                Err(e) => {
                    let _ = sender.send(AIResponse::Error(format!("Network Error: {}", e)));
                }
            }
        });
    }
}
