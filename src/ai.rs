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

/// Curated, verified quotes from Paul Graham's essays
/// These are exact quotes with sources for attribution
const PAUL_GRAHAM_QUOTES: &[(&str, &str)] = &[
    // From "How to Do Great Work"
    ("The way to figure out what to work on is by working. If you're not sure what to work on, guess. But pick something and get going.", "How to Do Great Work"),
    ("Develop a habit of working on your own projects. Don't let 'work' mean something other people tell you to do.", "How to Do Great Work"),
    ("The three most powerful motives are curiosity, delight, and the desire to do something impressive. Sometimes they converge, and that combination is the most powerful of all.", "How to Do Great Work"),
    ("Writing a page a day doesn't sound like much, but if you do it every day you'll write a book a year. That's the key: consistency.", "How to Do Great Work"),
    ("People who do great things don't get a lot done every day. They get something done, rather than nothing.", "How to Do Great Work"),
    ("Work doesn't just happen when you're trying to. There's a kind of undirected thinking you do when walking or taking a shower or lying in bed that can be very powerful.", "How to Do Great Work"),
    ("It's usually a mistake to lie to yourself if you want to do great work, but this is one of the rare cases where it isn't. When I'm reluctant to start work in the morning, I often trick myself by saying 'I'll just read over what I've got so far.'", "How to Do Great Work"),
    ("Try to finish what you start, though, even if it turns out to be more work than you expected. Finishing things is not just an exercise in tidiness or self-discipline.", "How to Do Great Work"),
    ("The reason we're surprised is that we underestimate the cumulative effect of work.", "How to Do Great Work"),
    ("Curiosity is the best guide. Your curiosity never lies, and it knows more than you do about what's worth paying attention to.", "How to Do Great Work"),
    ("If you made it this far, you must be interested in doing great work. And if so you're already further along than you might realize.", "How to Do Great Work"),
    ("Don't worry about being presumptuous. You don't have to tell anyone. And if it's too hard and you fail, so what? Lots of people have worse problems than that.", "How to Do Great Work"),
    ("The discoveries are out there, waiting to be made. Why not by you?", "How to Do Great Work"),
    
    // From "Keep Your Identity Small"
    ("The more labels you have for yourself, the dumber they make you.", "Keep Your Identity Small"),
    ("If people can't think clearly about anything that has become part of their identity, then all other things being equal, the best plan is to let as few things into your identity as possible.", "Keep Your Identity Small"),
    
    // From "Do Things That Don't Scale"
    ("Actually startups take off because the founders make them take off.", "Do Things That Don't Scale"),
    ("The question to ask about an early stage startup is not 'is this company taking over the world?' but 'how big could this company get if the founders did the right things?'", "Do Things That Don't Scale"),
    ("I have never once seen a startup lured down a blind alley by trying too hard to make their initial users happy.", "Do Things That Don't Scale"),
    ("It's not enough just to do something extraordinary initially. You have to make an extraordinary effort initially.", "Do Things That Don't Scale"),
    
    // From "Maker's Schedule, Manager's Schedule"
    ("When you're operating on the maker's schedule, meetings are a disaster. A single meeting can blow a whole afternoon, by breaking it into two pieces each too small to do anything hard in.", "Maker's Schedule, Manager's Schedule"),
    ("For someone on the maker's schedule, having a meeting is like throwing an exception. It doesn't merely cause you to switch from one task to another; it changes the mode in which you work.", "Maker's Schedule, Manager's Schedule"),
    ("Don't your spirits rise at the thought of having an entire day free to work, with no appointments at all?", "Maker's Schedule, Manager's Schedule"),
    
    // From "How to Start a Startup"
    ("What matters is not ideas, but the people who have them. Good people can fix bad ideas, but good ideas can't save bad people.", "How to Start a Startup"),
    ("The smarter they are, the less pressure they feel to act smart. So as a rule you can recognize genuinely smart people by their ability to say things like 'I don't know,' 'Maybe you're right,' and 'I don't understand x well enough.'", "How to Start a Startup"),
    ("It's worth trying very, very hard to make technology easy to use. Hackers are so used to computers that they have no idea how horrifying software seems to normal people.", "How to Start a Startup"),
    ("In technology, the low end always eats the high end. It's easier to make an inexpensive product more powerful than to make a powerful product cheaper.", "How to Start a Startup"),
    
    // From "The Bus Ticket Theory of Genius"
    ("If I had to put the recipe for genius into one sentence, that might be it: to have a disinterested obsession with something that matters.", "The Bus Ticket Theory of Genius"),
    ("An obsessive interest will even bring you luck, to the extent anything can. Chance, as Pasteur said, favors the prepared mind, and if there's one thing an obsessed mind is, it's prepared.", "The Bus Ticket Theory of Genius"),
    ("Perhaps the reason people have fewer new ideas as they get older is not simply that they're losing their edge. It may also be because once you become established, you can no longer mess about with irresponsible side projects.", "The Bus Ticket Theory of Genius"),
    ("The solution to that is obvious: remain irresponsible.", "The Bus Ticket Theory of Genius"),
];

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
            let is_quote_request = history
                .last()
                .map(|m| m.content.trim().eq_ignore_ascii_case("quote"))
                .unwrap_or(false);

            // Use different parameters for quote mode vs regular chat
            let (temperature, max_tokens) = if is_quote_request {
                (0.3, 150) // Lower temperature for accurate quote retrieval
            } else {
                (0.5, 600) // Balanced for task planning
            };

            let system_prompt = build_system_prompt(&context);
            
            let mut messages = vec![ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            }];
            messages.extend(history);

            let body = serde_json::json!({
                "model": "gpt-4o",
                "temperature": temperature,
                "presence_penalty": 0.2,
                "frequency_penalty": 0.3,
                "max_tokens": max_tokens,
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

fn build_system_prompt(context: &str) -> String {
    // Build the quote bank string from the curated quotes
    let quote_bank: String = PAUL_GRAHAM_QUOTES
        .iter()
        .map(|(quote, source)| format!("- \"{}\" — Paul Graham, {}", quote, source))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"You are Xiaolong's executive assistant specializing in the Eisenhower Matrix methodology. You combine the precision of a professional secretary with strategic thinking.

## CORE RESPONSIBILITIES

### Task Decomposition (GTD-Inspired)
When the user describes a goal or project:
1. Identify the **next physical action** — what's the very first concrete step that takes < 30 min?
2. Break larger tasks into 15-45 minute actionable chunks
3. Surface hidden dependencies: "Before X, you need Y"
4. Question scope: "Is this actually one task or three?"
5. Suggest time-boxing: "This looks like a 2-hour deep work block"

### Priority Assessment
Apply these criteria rigorously:

**Urgency (1-3):**
- 3: Due within 24h OR blocks others OR external deadline today
- 2: Due this week OR has scheduling constraint  
- 1: No time pressure, flexible timing

**Importance (1-3):**
- 3: Directly advances key goals (research, thesis, career), high-stakes, or irreversible
- 2: Contributes meaningfully but not critical path
- 1: Nice-to-have, low impact if skipped

### Challenge Low-Value Work
- For Q3 (Delegate): "Can this be delegated, automated, batched, or declined?"
- For Q4 (Drop): "Why is this on your list? Should it be dropped entirely?"
- Spot "urgency theater" — tasks that feel urgent but aren't truly important

## OUTPUT FORMAT
When suggesting tasks, use exactly:
[ADD] Task name u<1-3>i<1-3>

Examples:
[ADD] Draft email to Prof. Imai re: meeting agenda u2i3
[ADD] Review evalITR test failures u3i2
[ADD] Organize Obsidian research notes u1i2
[ADD] Buy groceries u2i1

## QUOTE COMMAND
When user says "quote" (case-insensitive), respond with ONE quote from the verified bank below, when using quote not from the bank, make sure it is a verified quote.
- Select randomly from the bank; don't repeat recent selections
- For variety, select quotes from authors across domains and eras: scientists, philosophers, artists, business people, etc.
- Output format: "[quote text]" — [author], [essay title]
- Rotate languages when using non-PG quotes: include Seneca, and others
- NEVER invent or paraphrase quotes; use exact wording from the bank

### VERIFIED PAUL GRAHAM QUOTE BANK:
{}

### ALTERNATE VERIFIED QUOTES (for variety):
- "事上磨练" — 王阳明 (Practice and refine yourself through action)
- "天下古今之庸人，皆以一惰字致败" — 曾国藩 (Mediocrity stems from laziness)
- "It is not that we have a short time to live, but that we waste a lot of it." — Seneca
- "予定は決意の半分である" — 松下幸之助 (A plan is half the commitment)
- "The best time to plant a tree was 20 years ago. The second best time is now."

## CURRENT TASKS IN SYSTEM:
{}

## STYLE GUIDELINES
- Be direct and concise; no filler phrases like "Great question!"
- One clear recommendation per response when possible
- Ask ONE clarifying question if the task is too vague to decompose
- Match the user's language (English/Chinese) when appropriate
- For complex planning, use structured output with clear next actions"#,
        quote_bank, context
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quote_bank_not_empty() {
        assert!(!PAUL_GRAHAM_QUOTES.is_empty());
        assert!(PAUL_GRAHAM_QUOTES.len() >= 20);
    }

    #[test]
    fn test_quotes_have_sources() {
        for (quote, source) in PAUL_GRAHAM_QUOTES {
            assert!(!quote.is_empty());
            assert!(!source.is_empty());
        }
    }

    #[test]
    fn test_system_prompt_includes_quotes() {
        let prompt = build_system_prompt("[]");
        assert!(prompt.contains("Paul Graham"));
        assert!(prompt.contains("How to Do Great Work"));
    }
}
