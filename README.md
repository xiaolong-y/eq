# EQ: Eisenhower Quadrants

**EQ** is a terminal-based task manager built in Rust, designed to help you prioritize tasks using the **Eisenhower Matrix** method. It focuses on speed, keyboard-centric navigation, and clear visual separation of priorities.

## 🚀 Installation

### Prerequisites
- Rust toolchain (`cargo`)
- (Optional) OpenAI API key for AI chat features

### Build & Install
To run `eq` from anywhere:

1.  **Install**:
    ```bash
    cargo install --path .
    ```

2.  **Add to PATH** (if not already):
    Add this to your `~/.zshrc` or `~/.bashrc`:
    ```bash
    export PATH="$HOME/.cargo/bin:$PATH"
    ```
    Then run `source ~/.zshrc`.

3.  **(Optional) Set up AI Chat**:
    Create a `.env` file in your working directory:
    ```bash
    OPENAI_API_KEY=your-api-key-here
    ```

Now you can just run:
```bash
eq tui
eq add "Task"
```

---

## 🧠 Core Concepts

### The Eisenhower Matrix
Tasks are categorized into four quadrants based on **Importance** and **Urgency**:

1.  **DO FIRST** (Q1): High Importance, High Urgency.
2.  **SCHEDULE** (Q2): High Importance, Low Urgency.
3.  **DELEGATE** (Q3): Low Importance, High Urgency.
4.  **DROP** (Q4): Low Importance, Low Urgency.

### Priority Scoring
Tasks are sorted within quadrants by a calculated score:
```
Score = (Importance × 3) + (Urgency × 2)
```
- **Importance**: 1 (Low) to 3 (High)
- **Urgency**: 1 (Low) to 3 (High)
- **Max Score**: 15 (I=3, U=3)

---

## ⌨️ TUI (Interactive Mode)

Launch with: `eq tui`

### Navigation
- **`Tab`**: Cycle through quadrants.
- **`h` / `l`** (or `←` / `→`): Move between columns.
- **`j` / `k`** (or `↓` / `↑`): Move up/down within a list.
- **`q`**: Quit.

### Actions
- **`a`**: **Add** a new task.
    - Input format: `Task Name !!!$$` or `Task Name u3i2`.
- **`e`**: **Edit** the selected task.
- **`d`** / **`Enter`**: Toggle task **Done** status (undo if already done).
- **`x`**: **Drop** (delete) the task.
- **`>`** / **`.`**: **Move** task to Tomorrow.
- **`t`**: Toggle view between **Today** and **Tomorrow**.
- **`?`**: Toggle help / wisdom.
- **`c`**: Open **AI Chat** interface.

### AI Chat Interface
- **`Enter`**: Send message.
- **`Esc`**: Close chat.
- **`PgUp` / `PgDn`**: Scroll chat history.
- **`Ctrl+K` / `Ctrl+J`**: Scroll one line.
- **`Home`**: Jump to top.
- **`End`**: Resume auto-scroll.
- **`Ctrl+L`**: Clear chat history.
- **`Ctrl+W`**: Delete word.
- **`Ctrl+U`**: Clear input line.
- **`?`**: Toggle help (when input is empty).

---

## 💻 CLI Usage

You can also manage tasks directly from the command line.

### Adding Tasks
Use `!` for urgency and `$` for importance, or `u<n>i<n>` shorthand.
```bash
eq add "Fix server crash !!!$$$"   # High Urgency (3), High Importance (3)
eq add "Buy milk u1i2"            # Urgency 1, Importance 2
eq add "Call Mom" --tomorrow      # Schedule for tomorrow
```

### Managing Tasks
```bash
eq today              # View today's matrix
eq tomorrow           # View tomorrow's matrix
eq yesterday          # View yesterday's matrix
eq week               # View weekly overview
eq done <id>          # Mark as complete
eq drop <id>          # Delete task
eq edit <id> u3i1     # Update priority
eq stats              # Show productivity statistics
```

---

## 💾 Data & Logging

### Storage Location
EQ stores all data in your operating system's standard application data directory, ensuring consistency regardless of where you run the command:

- **macOS**: `~/Library/Application Support/dev.quad_tasks.eq/`
- **Linux**: `~/.local/share/dev.quad_tasks.eq/`
- **Windows**: `%APPDATA%\quad_tasks\eq\data\`

### Data Files
- **`tasks.json`**: All your tasks with metadata
- **`chat_history.json`**: AI chat conversation history
- **`history.jsonl`**: Event log in JSON Lines format

### Custom Data Directory
You can override the default location by setting the `EQ_DATA_DIR` environment variable:
```bash
export EQ_DATA_DIR="$HOME/my-custom-location"
eq tui
```

### Event Log Format
Each line in `history.jsonl` is a JSON object tracking task lifecycle events:
```json
{"id":"...","timestamp":"...","action":"Created","task_id":"...","details":"Created task: Buy milk"}
```

---

## 🛠️ Recent Improvements

- **Chat Scrolling**: Navigate chat history with PgUp/PgDn, Ctrl+K/J
- **Cursor Visibility**: Input cursor now visible in chat and task editing
- **Index Stability**: Selection doesn't break after completing/dropping tasks
- **Persistent Chat**: Chat history saved between sessions
- **Week View**: See your entire week at a glance with `eq week`
- **Improved Parser**: Edge cases like `ui` without numbers handled gracefully
- **Cleaner Codebase**: Refactored widget system, simplified task lookup
