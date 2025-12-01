# EQ: Eisenhower Quadrants

**EQ** is a terminal-based task manager built in Rust, designed to help you prioritize tasks using the **Eisenhower Matrix** method. It focuses on speed, keyboard-centric navigation, and clear visual separation of priorities.

## 🚀 Installation

### Prerequisites
- Rust toolchain (`cargo`)

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
eq done <id>          # Mark as complete
eq drop <id>          # Delete task
eq edit <id> u3i1     # Update priority
```

---

## 💾 Data & Logging

### Storage
Tasks are stored in a `data` subdirectory relative to where you run the command (or the project root if running locally):
- **Location**: `./data/tasks.json`

### Event Log
- **Location**: `./data/history.jsonl`
- **Format**: JSON Lines (one JSON object per line).

```json
{"id":"...","timestamp":"...","action":"Created","task_id":"...","details":"Created task: Buy milk"}
```
