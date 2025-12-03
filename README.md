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
- **Prioritization Help**: Ask AI how to categorize tasks.
- **Breakdown**: Split complex projects into steps.
- **Context Aware**: The AI knows your current task list.
- **Automatic Audit**: The AI helps you add tasks upon your confirmation. 

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
