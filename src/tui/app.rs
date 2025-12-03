use crate::models::store::TaskStore;
use crate::models::task::{Quadrant, TaskStatus};
use chrono::{Duration, Local, NaiveDate};
use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

use super::zen::ZenState;
use crate::ai::{AIClient, AIResponse, ChatMessage};
use crate::parser::ai_commands::{
    parse_commands, AICommand, CommandResults, TaskIdentifier,
};
use std::sync::mpsc;

pub enum CurrentScreen {
    Main,
    Editing,
    Chat,
    Focus,   // Full-screen quadrant view
    ZenMode, // Single task focus mode
    Exiting,
}

pub struct App<'a> {
    pub store: &'a mut TaskStore,
    pub current_screen: CurrentScreen,
    pub selected_quadrant: Quadrant,
    pub selected_task_index: usize,
    pub view_date: NaiveDate,
    pub input_buffer: String,
    pub input_mode: bool,
    pub editing_task_id: Option<uuid::Uuid>,
    pub show_help: bool,

    // AI Chat State
    pub chat_history: Vec<ChatMessage>,
    pub chat_input: String,
    pub ai_client: Option<AIClient>,
    pub chat_receiver: Option<mpsc::Receiver<AIResponse>>,
    pub is_loading: bool,
    pub chat_scroll: u16,
    pub chat_auto_scroll: bool,
    pub show_chat_help: bool,        // Fix #5: Chat help toggle
    pub spinner_state: u8,           // Spinner animation state
    pub zen_state: Option<ZenState>, // Zen mode state with particles and pomodoro

    // Pending AI commands
    pub pending_commands: Vec<AICommand>,
}

impl<'a> App<'a> {
    pub fn new(store: &'a mut TaskStore) -> App<'a> {
        // Fix #8: Load persisted chat history
        let saved_history = TaskStore::load_chat_history();
        let chat_history: Vec<ChatMessage> = saved_history
            .into_iter()
            .map(|m| ChatMessage {
                role: m.role,
                content: m.content,
            })
            .collect();

        App {
            store,
            current_screen: CurrentScreen::Main,
            selected_quadrant: Quadrant::DoFirst,
            selected_task_index: 0,
            view_date: Local::now().date_naive(),
            input_buffer: String::new(),
            input_mode: false,
            editing_task_id: None,
            show_help: false,

            chat_history,
            chat_input: String::new(),
            ai_client: AIClient::new(),
            chat_receiver: None,
            is_loading: false,
            chat_scroll: 0,
            chat_auto_scroll: true,
            show_chat_help: false,
            spinner_state: 0,
            zen_state: None,
            pending_commands: Vec::new(),
        }
    }

    pub fn toggle_view_date(&mut self) {
        let today = Local::now().date_naive();
        if self.view_date == today {
            self.view_date = today + Duration::days(1);
        } else {
            self.view_date = today;
        }
    }

    /// Fix #4: Get task count for current quadrant and clamp index if needed
    pub fn get_current_task_count(&self) -> usize {
        self.store
            .tasks
            .iter()
            .filter(|t| {
                t.date == self.view_date
                    && t.status != TaskStatus::Dropped
                    && t.quadrant() == self.selected_quadrant
            })
            .count()
    }

    /// Fix #4: Clamp the selected index to valid range
    pub fn clamp_selected_index(&mut self) {
        let count = self.get_current_task_count();
        if count == 0 {
            self.selected_task_index = 0;
        } else if self.selected_task_index >= count {
            self.selected_task_index = count - 1;
        }
    }

    /// Fix #8: Save chat history to disk
    pub fn save_chat_history(&self) {
        let history: Vec<crate::models::store::ChatMessage> = self
            .chat_history
            .iter()
            .map(|m| crate::models::store::ChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
            })
            .collect();
        let _ = TaskStore::save_chat_history(&history);
    }

    /// Process AI response and extract commands
    pub fn process_ai_response(&mut self, content: String) -> String {
        let commands = parse_commands(&content);
        if commands.is_empty() {
            return content;
        }

        self.pending_commands = commands;
        
        // Format the pending commands for display
        let mut msg = content;
        msg.push_str("\n\n━━━ Pending Commands ━━━\n");
        
        for (i, cmd) in self.pending_commands.iter().enumerate() {
            match cmd {
                AICommand::Add(t) => {
                    msg.push_str(&format!("  {}. ADD: {} (u{}i{})\n", i + 1, t.title, t.urgency, t.importance));
                }
                AICommand::Done(id) => {
                    msg.push_str(&format!("  {}. DONE: {}\n", i + 1, self.format_identifier(id)));
                }
                AICommand::Drop(id) => {
                    msg.push_str(&format!("  {}. DROP: {}\n", i + 1, self.format_identifier(id)));
                }
                AICommand::Edit { target, new_title, new_urgency, new_importance } => {
                    let target_str = self.format_identifier(target);
                    let mut changes = Vec::new();
                    if let Some(t) = new_title { changes.push(format!("title='{}'", t)); }
                    if let Some(u) = new_urgency { changes.push(format!("urgency={}", u)); }
                    if let Some(i) = new_importance { changes.push(format!("importance={}", i)); }
                    
                    msg.push_str(&format!("  {}. EDIT: {} → {}\n", i + 1, target_str, changes.join(", ")));
                }
            }
        }
        
        msg.push_str("\n⚡ Press [y] to execute, [n] to cancel");
        msg
    }

    /// Execute all pending commands
    pub fn execute_pending_commands(&mut self) -> String {
        if self.pending_commands.is_empty() {
            return String::new();
        }

        let commands = std::mem::take(&mut self.pending_commands);
        let mut results = CommandResults::default();

        for cmd in commands {
            match cmd {
                AICommand::Add(parsed) => {
                    let task = crate::models::task::Task::new(
                        parsed.title.clone(),
                        parsed.urgency,
                        parsed.importance,
                        self.view_date,
                    );
                    self.store.add_task(task);
                    results.tasks_added.push(parsed);
                }

                AICommand::Done(identifier) => {
                    if let Some((task_id, title)) = self.find_task_by_identifier(&identifier) {
                        self.store.toggle_complete_task(task_id);
                        results.tasks_completed.push(title);
                    } else {
                        results.errors.push(format!(
                            "Could not find task: {}",
                            self.format_identifier(&identifier)
                        ));
                    }
                }

                AICommand::Drop(identifier) => {
                    if let Some((task_id, title)) = self.find_task_by_identifier(&identifier) {
                        self.store.drop_task(task_id);
                        results.tasks_dropped.push(title);
                    } else {
                        results.errors.push(format!(
                            "Could not find task: {}",
                            self.format_identifier(&identifier)
                        ));
                    }
                }

                AICommand::Edit {
                    target,
                    new_title,
                    new_urgency,
                    new_importance,
                } => {
                    if let Some((task_id, old_title)) = self.find_task_by_identifier(&target) {
                        let (current_title, current_u, current_i) = {
                            let task = self.store.tasks.iter().find(|t| t.id == task_id).unwrap();
                            (task.title.clone(), task.urgency, task.importance)
                        };

                        let final_title = new_title.unwrap_or(current_title);
                        let final_u = new_urgency.unwrap_or(current_u);
                        let final_i = new_importance.unwrap_or(current_i);

                        self.store.update_task(task_id, final_title.clone(), final_u, final_i);
                        results.tasks_edited.push(format!(
                            "{} → {} (u{}i{})",
                            old_title, final_title, final_u, final_i
                        ));
                    } else {
                        results.errors.push(format!(
                            "Could not find task: {}",
                            self.format_identifier(&target)
                        ));
                    }
                }
            }
        }

        // Save the store if we made any changes
        if !results.tasks_added.is_empty()
            || !results.tasks_completed.is_empty()
            || !results.tasks_dropped.is_empty()
            || !results.tasks_edited.is_empty()
        {
            let _ = self.store.save();
            self.clamp_selected_index();
        }

        results.format_confirmation()
    }

    /// Cancel pending commands without executing
    pub fn cancel_pending_commands(&mut self) -> String {
        let count = self.pending_commands.len();
        self.pending_commands.clear();
        format!("\n\n━━━ Cancelled {} command(s) ━━━", count)
    }

    /// Check if there are pending commands awaiting confirmation
    pub fn has_pending_commands(&self) -> bool {
        !self.pending_commands.is_empty()
    }

    /// Find a task by identifier (title fragment or index)
    fn find_task_by_identifier(&self, identifier: &TaskIdentifier) -> Option<(uuid::Uuid, String)> {
        match identifier {
            TaskIdentifier::Index(idx) => {
                // Get tasks in current quadrant, sorted by score
                let mut tasks: Vec<&crate::models::task::Task> = self
                    .store
                    .tasks
                    .iter()
                    .filter(|t| {
                        t.date == self.view_date
                            && t.status == TaskStatus::Pending
                            && t.quadrant() == self.selected_quadrant
                    })
                    .collect();
                tasks.sort_by_key(|t| std::cmp::Reverse(t.score()));

                // 1-based index
                if *idx > 0 && *idx <= tasks.len() {
                    let task = tasks[*idx - 1];
                    Some((task.id, task.title.clone()))
                } else {
                    None
                }
            }
            TaskIdentifier::Title(title_fragment) => {
                // Case-insensitive substring match on today's pending tasks
                let fragment_lower = title_fragment.to_lowercase();
                self.store
                    .tasks
                    .iter()
                    .filter(|t| t.date == self.view_date && t.status == TaskStatus::Pending)
                    .find(|t| t.title.to_lowercase().contains(&fragment_lower))
                    .map(|t| (t.id, t.title.clone()))
            }
        }
    }

    /// Format identifier for error messages
    fn format_identifier(&self, identifier: &TaskIdentifier) -> String {
        match identifier {
            TaskIdentifier::Index(idx) => format!("#{}", idx),
            TaskIdentifier::Title(t) => format!("\"{}\"", t),
        }
    }
}

pub fn run(store: &mut TaskStore) -> Result<(), Box<dyn std::error::Error>> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new(store);

    // Run loop
    let res = run_app(&mut terminal, &mut app);

    // Fix #8: Save chat history on exit
    app.save_chat_history();

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err);
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        // Increment spinner state for animation
        app.spinner_state = app.spinner_state.wrapping_add(1);

        terminal.draw(|f| crate::tui::ui::ui(f, app))?;

        // Poll for AI responses
        if let Some(receiver) = &app.chat_receiver {
            if let Ok(response) = receiver.try_recv() {
                app.is_loading = false;
                match response {
                    AIResponse::Success(content) => {
                        // Process response and auto-add any [ADD] tasks
                        let full_content = app.process_ai_response(content);

                        app.chat_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: full_content,
                        });
                        // Fix #8: Auto-save after AI response
                        app.save_chat_history();
                    }
                    AIResponse::Error(err) => {
                        app.chat_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: format!("Error: {}", err),
                        });
                    }
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            let event = event::read()?;
            if let Some(res) = crate::tui::handlers::handle_key_events(event, app) {
                if res {
                    return Ok(());
                }
            }
        }
    }
}
