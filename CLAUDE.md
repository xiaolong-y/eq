# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**EQ** is a terminal-based Eisenhower Matrix task manager written in Rust. It uses the Eisenhower Matrix (4-quadrant priority system) to help users categorize tasks by urgency and importance, with an interactive TUI built on `ratatui` and an AI chat assistant powered by OpenAI.

## Build & Development Commands

```bash
# Build the project
cargo build

# Run the TUI interface
cargo run -- tui

# Run specific CLI commands
cargo run -- add "Task name" u2i3
cargo run -- today
cargo run -- week

# Install locally (makes `eq` available globally)
cargo install --path .

# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run tests in a specific module
cargo test parser::input::tests
```

## Architecture

### Core Data Model

The Eisenhower Matrix is implemented through three key components in `src/models/`:

- **`task.rs`**: Defines `Task` struct with `urgency` (1-3) and `importance` (1-3) fields. The `quadrant()` method calculates which quadrant a task belongs to:
  - `DoFirst`: importance ≥ 2 AND urgency ≥ 2
  - `Schedule`: importance ≥ 2 AND urgency = 1
  - `Delegate`: importance = 1 AND urgency ≥ 2
  - `Drop`: importance = 1 AND urgency = 1

  Tasks are scored via `score() = (importance × 3) + (urgency × 2)` for sorting within quadrants.

- **`store.rs`**: `TaskStore` manages all tasks and persistence. Uses atomic writes (write to .tmp then rename) for data safety. Chat history is persisted separately in `data/chat_history.json`. The `find_task_id()` method handles both 1-based index lookup (for CLI) and UUID prefix matching.

- **`log.rs`**: Append-only event log in JSONL format at `data/history.jsonl` tracks all task lifecycle events (Created, Updated, Completed, Dropped, Moved).

### TUI Architecture (`src/tui/`)

The TUI is organized into clear separation of concerns:

- **`app.rs`**: Contains `App` struct (the application state container) and the main event loop in `run_app()`. Polls for AI responses asynchronously via `mpsc::Receiver`. Chat history is loaded on startup and saved on exit and after each AI response.

- **`handlers.rs`**: All keyboard input handling. Different handler functions for Main, Editing, and Chat screens. Key bindings are centralized here.

- **`ui.rs`**: All rendering logic using `ratatui`. Dispatches to different rendering functions based on `CurrentScreen`.

- **`widgets/`**: Custom reusable widgets. `quadrant.rs` contains the `QuadrantWidget` that renders individual quadrants with task lists.

### Priority Input Parser (`src/parser/input.rs`)

Supports two formats for specifying task priority:
1. Symbol notation: `!!!$$$` (3 urgency, 3 importance)
2. Shorthand notation: `u3i2` or `i2u3` (order-independent)

The parser defaults missing values to 1 if the other is specified (e.g., `!!` → urgency=2, importance=1). Contains unit tests for edge cases.

### AI Integration (`src/ai.rs`)

OpenAI GPT-4o integration with:
- System prompt that understands Eisenhower Matrix context
- Task suggestion format: `[ADD] Task name u<urgency>i<importance>`
- Special "quote" command that returns inspirational quotes in random languages
- Async message sending via thread + mpsc channel to avoid blocking the TUI

The API key is loaded from `.env` file via `dotenv` crate.

## Data Storage

All data is stored in the `./data/` directory (relative to where the command is run):
- `tasks.json`: All tasks (uses pretty JSON for readability)
- `chat_history.json`: Persisted AI chat messages
- `history.jsonl`: Append-only event log (one JSON object per line)

The store uses atomic writes to prevent data corruption. Always write to a `.tmp` file, sync to disk, then rename to the actual file.

## Task Lifecycle & State Management

Tasks have three states: `Pending`, `Completed`, `Dropped`. The TUI maintains selection state per quadrant, and the selection index is clamped after task operations to prevent out-of-bounds errors (see `clamp_selected_index()` in `app.rs`).

When toggling task completion in the TUI, `toggle_complete_task()` allows undoing completion. The CLI uses `complete_task()` which is one-way only.

## Recent Bug Fixes

The codebase has been hardened against several edge cases:
- **Index stability**: Selection index is clamped after completing/dropping tasks to prevent crashes
- **Parser robustness**: Handles edge cases like `ui` without numbers gracefully
- **Chat scrolling**: Auto-scroll behavior with manual override (End key resumes auto-scroll)
- **Cursor visibility**: Input cursor is now visible in chat and editing modes
- **Chat persistence**: History is saved after each AI response and on exit

## Development Notes

- The project uses `crossterm` for terminal manipulation and `ratatui` for TUI rendering
- AI features are optional and gracefully degrade if `OPENAI_API_KEY` is not set
- The TUI runs in raw mode with an alternate screen buffer (restored on exit)
- CLI commands use 1-based indexing for user-friendliness, but internal code uses 0-based
- Week view starts on Monday and shows top 3 tasks per day with overflow indication
