use std::io;
use crossterm::{
    event::{self},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use crate::models::store::TaskStore;
use crate::models::task::Quadrant;
use crate::tui::ui::ui;
use crate::tui::handlers::handle_key_events;
use chrono::{Local, NaiveDate, Duration};

use crate::ai::{AIClient, ChatMessage, AIResponse};
use std::sync::mpsc;

pub enum CurrentScreen {
    Main,
    Editing,
    Chat,
    Exiting,
}

pub struct App<'a> {
    pub store: &'a mut TaskStore,
    pub current_screen: CurrentScreen,
    pub selected_quadrant: Quadrant,
    pub selected_task_index: usize, // Index within the filtered quadrant list
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
}

impl<'a> App<'a> {
    pub fn new(store: &'a mut TaskStore) -> App<'a> {
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
            
            chat_history: Vec::new(),
            chat_input: String::new(),
            ai_client: AIClient::new(),
            chat_receiver: None,
            is_loading: false,
            chat_scroll: 0,
            chat_auto_scroll: true,
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

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen
    )?;
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
        terminal.draw(|f| ui(f, app))?;

        // Poll for AI responses (also done in handle_key_events, but we need it here for async updates)
        if let Some(receiver) = &app.chat_receiver {
            if let Ok(response) = receiver.try_recv() {
                app.is_loading = false;
                match response {
                    AIResponse::Success(content) => {
                        app.chat_history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content,
                        });
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
            if let Some(res) = handle_key_events(event, app) {
                if res {
                    return Ok(());
                }
            }
        }
    }
}
