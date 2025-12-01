use crossterm::event::{Event, KeyCode, KeyEvent};
use crate::tui::app::{App, CurrentScreen};
use crate::models::task::{Task, Quadrant, TaskStatus};
use crate::ai::{ChatMessage, AIResponse};
use std::sync::mpsc;
use crate::parser::input::parse_priority;

pub fn handle_key_events(event: Event, app: &mut App) -> Option<bool> {
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

    match event {
        Event::Key(key) => {
            match app.current_screen {
                CurrentScreen::Main => handle_main_screen(key, app),
                CurrentScreen::Editing => handle_editing_screen(key, app),
                CurrentScreen::Chat => handle_chat_screen(key, app),
                CurrentScreen::Exiting => Some(true),
            }
        }
        _ => Some(false),
    }
}

fn handle_main_screen(key: KeyEvent, app: &mut App) -> Option<bool> {
    match key.code {
        KeyCode::Char('q') => return Some(true),
        KeyCode::Char('c') => {
            app.current_screen = CurrentScreen::Chat;
        }
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
        }
        KeyCode::Char('a') => {
            app.current_screen = CurrentScreen::Editing;
            app.input_mode = true;
            app.input_buffer.clear();
            app.editing_task_id = None;
        }
        KeyCode::Char('e') => {
            if let Some(task_id) = get_selected_task_id(app) {
                 if let Some(task) = app.store.tasks.iter().find(|t| t.id == task_id) {
                    app.input_buffer = format!("{} u{}i{}", task.title, task.urgency, task.importance);
                    app.editing_task_id = Some(task_id);
                    app.current_screen = CurrentScreen::Editing;
                    app.input_mode = true;
                }
            }
        }

        KeyCode::Char('d') | KeyCode::Enter => {
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.toggle_complete_task(task_id);
                let _ = app.store.save();
            }
        }
        KeyCode::Char('x') => {
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.drop_task(task_id);
                let _ = app.store.save();
            }
        }
        KeyCode::Char('t') => {
            app.view_date = if app.view_date == chrono::Local::now().date_naive() {
                chrono::Local::now().date_naive() + chrono::Duration::days(1)
            } else {
                chrono::Local::now().date_naive()
            };
        }
        KeyCode::Char('>') | KeyCode::Char('.') => {
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.move_task_to_date(task_id, app.view_date + chrono::Duration::days(1));
                let _ = app.store.save();
            }
        }
        KeyCode::Tab => {
            app.selected_quadrant = match app.selected_quadrant {
                Quadrant::DoFirst => Quadrant::Schedule,
                Quadrant::Schedule => Quadrant::Delegate,
                Quadrant::Delegate => Quadrant::Drop,
                Quadrant::Drop => Quadrant::DoFirst,
            };
            app.selected_task_index = 0;
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let count = get_task_count(app);
            if count > 0 {
                app.selected_task_index = (app.selected_task_index + 1) % count;
            }
        }
        KeyCode::Up | KeyCode::Char('k') => {
            let count = get_task_count(app);
            if count > 0 {
                if app.selected_task_index == 0 {
                    app.selected_task_index = count - 1;
                } else {
                    app.selected_task_index -= 1;
                }
            }
        }

        KeyCode::Left | KeyCode::Char('h') => {
            app.selected_quadrant = match app.selected_quadrant {
                Quadrant::Schedule => Quadrant::DoFirst,
                Quadrant::Drop => Quadrant::Delegate,
                _ => app.selected_quadrant,
            };
            app.selected_task_index = 0;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.selected_quadrant = match app.selected_quadrant {
                Quadrant::DoFirst => Quadrant::Schedule,
                Quadrant::Delegate => Quadrant::Drop,
                _ => app.selected_quadrant,
            };
            app.selected_task_index = 0;
        }
        _ => {}
    }
    None
}

fn handle_editing_screen(key: KeyEvent, app: &mut App) -> Option<bool> {
    match key.code {
        KeyCode::Enter => {
            let input = app.input_buffer.trim().to_string();
            if !input.is_empty() {
                let mut urgency = 1;
                let mut importance = 1;
                let mut title_parts = Vec::new();
                
                for part in input.split_whitespace() {
                    if let Some((u, i)) = parse_priority(part) {
                        urgency = u;
                        importance = i;
                    } else {
                        title_parts.push(part);
                    }
                }
                let title = title_parts.join(" ");

                if let Some(edit_id) = app.editing_task_id {
                    app.store.update_task(edit_id, title, urgency, importance);
                    app.editing_task_id = None;
                } else {
                    let task = Task::new(title, urgency, importance, app.view_date);
                    app.store.add_task(task);
                }
                let _ = app.store.save();
            }
            app.input_buffer.clear();
            app.input_mode = false;
            app.current_screen = CurrentScreen::Main;
        }
        KeyCode::Esc => {
            app.input_buffer.clear();
            app.input_mode = false;
            app.editing_task_id = None;
            app.current_screen = CurrentScreen::Main;
        }
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => {
            app.input_buffer.push(c);
        }
        _ => {}
    }
    None
}

fn handle_chat_screen(key: KeyEvent, app: &mut App) -> Option<bool> {
    match key.code {
        KeyCode::Esc => {
            app.current_screen = CurrentScreen::Main;
        }

        KeyCode::Char('w') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
            // Delete word
            if let Some(last_space) = app.chat_input.trim_end().rfind(' ') {
                app.chat_input.truncate(last_space + 1);
            } else {
                app.chat_input.clear();
            }
        }
        KeyCode::Backspace if key.modifiers.contains(crossterm::event::KeyModifiers::ALT) => {
            // Delete word (Alt+Backspace)
            if let Some(last_space) = app.chat_input.trim_end().rfind(' ') {
                app.chat_input.truncate(last_space + 1);
            } else {
                app.chat_input.clear();
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
            // Delete line
            app.chat_input.clear();
        }
        KeyCode::Enter => {
            if !app.chat_input.trim().is_empty() {
                let content = app.chat_input.trim().to_string();
                app.chat_history.push(ChatMessage {
                    role: "user".to_string(),
                    content: content.clone(),
                });
                
                // Send to AI
                if let Some(client) = &app.ai_client {
                    let (tx, rx) = mpsc::channel();
                    app.chat_receiver = Some(rx);
                    app.is_loading = true;
                    // Reset auto-scroll to true when sending a message
                    app.chat_auto_scroll = true;
                    
                    // Serialize tasks for context
                    let context = serde_json::to_string_pretty(&app.store.tasks).unwrap_or_default();
                    
                    client.send_message(app.chat_history.clone(), context, tx);
                } else {
                    app.chat_history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: "API Key not found. Please set OPENAI_API_KEY.".to_string(),
                    });
                }
                
                app.chat_input.clear();
            }
        }
        KeyCode::Backspace => {
            app.chat_input.pop();
        }
        KeyCode::Char(c) => {
            app.chat_input.push(c);
        }
        _ => {}
    }
    None
}

fn get_filtered_tasks<'a>(app: &'a App) -> Vec<&'a Task> {
    let mut tasks: Vec<&Task> = app.store.tasks.iter()
        .filter(|t| t.date == app.view_date 
            && t.status != TaskStatus::Dropped 
            && t.quadrant() == app.selected_quadrant)
        .collect();
    tasks.sort_by_key(|b| std::cmp::Reverse(b.score()));
    tasks
}

fn get_task_count(app: &App) -> usize {
    get_filtered_tasks(app).len()
}

fn get_selected_task_id(app: &App) -> Option<uuid::Uuid> {
    let tasks = get_filtered_tasks(app);
    if app.selected_task_index < tasks.len() {
        Some(tasks[app.selected_task_index].id)
    } else {
        None
    }
}
