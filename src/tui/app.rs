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
                        app.chat_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content,
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
