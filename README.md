<div align="center">
  <img src="assets/logo.png" alt="eq Logo" width="120"/>
  <h1>eq</h1>
  <p><strong>Focus on what matters.</strong></p>
  <p>
    <a href="https://xiaolong-y.github.io/eq/">Website</a> •
    <a href="#installation">Installation</a> •
    <a href="#features">Features</a>
  </p>
</div>

---

**eq** is a terminal-based task manager designed for speed and focus. It uses the Eisenhower Matrix to prioritize tasks by urgency and importance, keeping your workflow efficient and keyboard-driven.

## Installation

```bash
cargo install eq
```

## Features

- **Eisenhower Matrix**
  Automatically sorts tasks into four quadrants: Do First, Schedule, Delegate, and Drop.

- **Zen Mode**
  A distraction-free interface with a built-in Pomodoro timer and floating particles to help you maintain flow.

- **AI Integration**
  Chat with your tasks. The AI helps you prioritize, break down complex projects, and organize your day.

- **Keyboard Driven**
  Navigate and manage tasks entirely from your keyboard using Vim-like shortcuts.

- **Privacy First**
  All data is stored locally on your machine.

## Usage

Launch the interactive interface:

```bash
eq tui
```

### Shortcuts

| Key | Action |
| :--- | :--- |
| `a` | Add task |
| `d` | Toggle done |
| `x` | Delete task |
| `z` | Zen Mode |
| `c` | AI Chat |
| `Tab` | Switch Quadrant |
| `?` | Help |

### CLI

Add tasks directly from your terminal:

```bash
# Invoke eq tui to launch the interactive interface
eq tui

# Add a high-priority task (Urgency 3, Importance 3)
eq add "Fix server crash !!!$$$"

# Add a scheduled task (Urgency 1, Importance 3)
eq add "Plan roadmap u1i3"

# View stats
eq stats
```

## Configuration

Data is stored in your system's standard application data directory. To enable AI features, set `OPENAI_API_KEY` in your environment or a `.env` file.action-free focus mode.
- **Pomodoro Timer**: Built-in 25-minute timer.
- **Visuals**: Floating particles to help you flow.
- **Single Task**: Focus on one thing at a time.

### AI Integration

**eq** doesn't just use ChatGPT's API—it implements custom prompt engineering to function as an effective executive assistant specialized in the Eisenhower Matrix methodology.

#### Engineered Capabilities

**GTD-Inspired Task Decomposition**
The AI is prompt-engineered to identify the "next physical action" for any goal, breaking large projects into 15-45 minute actionable chunks and surfacing hidden dependencies.

```
You: "I need to submit my research paper"

AI: Let me break this down into next actions:
[ADD] Export bibliography from Zotero u3i3
[ADD] Proofread introduction section u2i3
[ADD] Format figures to journal specifications u2i2
[ADD] Complete author checklist form u3i2

Before submitting, you need the co-author approval. Should we add that?
```

**Rigorous Priority Assessment**
Rather than accepting vague priorities, the AI applies explicit criteria for urgency (1-3) and importance (1-3), challenging assumptions and spotting "urgency theater."

```
You: "Add task: respond to all emails"

AI: Let's decompose this. "All emails" is too broad—which ones are actually urgent?
- Are any blocking other people's work? (u3)
- Are any from your advisor with time-sensitive questions? (u3i3)
- Can routine updates be batched for Friday afternoon? (u1i1)

Consider:
[ADD] Reply to Prof. Chen re: dataset access u3i3
[ADD] Batch process newsletter emails u1i1
```

**Multilingual Inspirational Quotes**
Type `quote` to receive verified quotes from dual sources: a handpicked collection of Paul Graham essays, classical Chinese philosophy, Stoic wisdom, and Japanese business thought—plus the AI's ability to extract and verify quotes from its extensive knowledge base with extremely low hallucination probability due to implemented guardrails which define its quote retrieval behaviors.

```
You: quote

AI: "事上磨练" — 王阳明 (Practice and refine yourself through action)

You: quote

AI: "People who do great things don't get a lot done every day.
They get something done, rather than nothing."
— Paul Graham, How to Do Great Work
```

#### Technical Implementation

The system prompt uses:
- **Context injection**: Current task list is passed to GPT-4o for relevance
- **Temperature tuning**: Lower temperature (0.3) for quote retrieval, balanced (0.5) for task planning
- **Structured output format**: `[ADD] Task name u<1-3>i<1-3>` for automatic parsing
- **Role-specific instructions**: Trained to challenge low-value work and identify tasks for Q3 (Delegate) and Q4 (Drop)

This demonstrates practical prompt engineering deployed in service of productivity—turning a general language model into a domain-specific task management assistant. 

### CLI Usage
Add tasks quickly from your shell:
```bash
eq add "Fix server crash !!!$$$"   # High Urgency (3), High Importance (3) -> Q1
eq add "Buy milk u1i2"            # Urgency 1, Importance 2 -> Q3
eq add "Call Mom" --tomorrow      # Schedule for tomorrow
```

---

## Data
Data is stored locally in your system's standard data directory (e.g., `~/Library/Application Support/dev.quad_tasks.eq/` on macOS).
- `tasks.json`: Task database.
- `history.jsonl`: Event log.
- `chat_history.json`: Saved AI conversations.
