use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crate::tui::app::{App, CurrentScreen};
use crate::models::task::{Task, Quadrant, TaskStatus};
use crate::ai::{ChatMessage, AIResponse};
use std::sync::mpsc;
use crate::parser::input::parse_priority;
use crate::tui::zen::Pomodoro;

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

    match event {
        Event::Key(key) => {
            match app.current_screen {
                CurrentScreen::Main => handle_main_screen(key, app),
                CurrentScreen::Editing => handle_editing_screen(key, app),
                CurrentScreen::Chat => handle_chat_screen(key, app),
                CurrentScreen::Focus => handle_focus_screen(key, app),
                CurrentScreen::ZenMode => handle_zen_screen(key, app),
                CurrentScreen::Exiting => Some(true),
            }
        }
        _ => Some(false),
    }
}

fn handle_main_screen(key: KeyEvent, app: &mut App) -> Option<bool> {
    match key.code {
        KeyCode::Char('q') => return Some(true),
        KeyCode::Char('z') => {
            // Enter Focus mode (full-screen quadrant)
            app.current_screen = CurrentScreen::Focus;
        }
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
                // Fix #4: Clamp index after mutation
                app.clamp_selected_index();
            }
        }
        KeyCode::Char('x') => {
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.drop_task(task_id);
                let _ = app.store.save();
                // Fix #4: Clamp index after mutation
                app.clamp_selected_index();
            }
        }
        KeyCode::Char('t') => {
            app.view_date = if app.view_date == chrono::Local::now().date_naive() {
                chrono::Local::now().date_naive() + chrono::Duration::days(1)
            } else {
                chrono::Local::now().date_naive()
            };
            // Fix #4: Clamp index when switching views
            app.clamp_selected_index();
        }
        KeyCode::Char('>') | KeyCode::Char('.') => {
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.move_task_to_date(task_id, app.view_date + chrono::Duration::days(1));
                let _ = app.store.save();
                // Fix #4: Clamp index after mutation
                app.clamp_selected_index();
            }
        }
        KeyCode::Tab => {
            app.selected_quadrant = match app.selected_quadrant {
                Quadrant::DoFirst => Quadrant::Schedule,
                Quadrant::Schedule => Quadrant::Delegate,
                Quadrant::Delegate => Quadrant::Drop,
                Quadrant::Drop => Quadrant::DoFirst,
            };
            // Fix #4: Reset and clamp index when switching quadrants
            app.selected_task_index = 0;
            app.clamp_selected_index();
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
        KeyCode::PageDown => {
            let count = get_task_count(app);
            if count > 0 {
                // Jump down by 5 items or to the end
                app.selected_task_index = (app.selected_task_index + 5).min(count - 1);
            }
        }
        KeyCode::PageUp => {
            let count = get_task_count(app);
            if count > 0 {
                // Jump up by 5 items or to the start
                if app.selected_task_index >= 5 {
                    app.selected_task_index -= 5;
                } else {
                    app.selected_task_index = 0;
                }
            }
        }

        KeyCode::Left | KeyCode::Char('h') => {
            app.selected_quadrant = match app.selected_quadrant {
                Quadrant::Schedule => Quadrant::DoFirst,
                Quadrant::Drop => Quadrant::Delegate,
                _ => app.selected_quadrant,
            };
            // Fix #4: Reset and clamp index
            app.selected_task_index = 0;
            app.clamp_selected_index();
        }
        KeyCode::Right | KeyCode::Char('l') => {
            app.selected_quadrant = match app.selected_quadrant {
                Quadrant::DoFirst => Quadrant::Schedule,
                Quadrant::Delegate => Quadrant::Drop,
                _ => app.selected_quadrant,
            };
            // Fix #4: Reset and clamp index
            app.selected_task_index = 0;
            app.clamp_selected_index();
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
            // Fix #4: Clamp after adding/editing
            app.clamp_selected_index();
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
            // Fix #8: Save chat on exit
            app.save_chat_history();
        }

        // Fix #5: Toggle chat help
        KeyCode::Char('?') if app.chat_input.is_empty() => {
            app.show_chat_help = !app.show_chat_help;
        }

        // Fix #1: Scroll up in chat history
        KeyCode::PageUp => {
            app.chat_auto_scroll = false;
            if app.chat_scroll > 5 {
                app.chat_scroll -= 5;
            } else {
                app.chat_scroll = 0;
            }
        }

        // Fix #1: Scroll down in chat history
        KeyCode::PageDown => {
            app.chat_scroll += 5;
            // Will be clamped in ui.rs
        }

        // Fix #1: Scroll up with Ctrl+K
        KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.chat_auto_scroll = false;
            if app.chat_scroll > 0 {
                app.chat_scroll -= 1;
            }
        }

        // Fix #1: Scroll down with Ctrl+J
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.chat_scroll += 1;
        }

        // Fix #1: Jump to bottom (resume auto-scroll)
        KeyCode::End => {
            app.chat_auto_scroll = true;
        }

        // Fix #1: Jump to top
        KeyCode::Home => {
            app.chat_auto_scroll = false;
            app.chat_scroll = 0;
        }

        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Delete word
            if let Some(last_space) = app.chat_input.trim_end().rfind(' ') {
                app.chat_input.truncate(last_space + 1);
            } else {
                app.chat_input.clear();
            }
        }
        KeyCode::Backspace if key.modifiers.contains(KeyModifiers::ALT) => {
            // Delete word (Alt+Backspace)
            if let Some(last_space) = app.chat_input.trim_end().rfind(' ') {
                app.chat_input.truncate(last_space + 1);
            } else {
                app.chat_input.clear();
            }
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            // Delete line
            app.chat_input.clear();
        }

        // Clear chat history
        KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.chat_history.clear();
            app.chat_scroll = 0;
            app.save_chat_history();
        }

        KeyCode::Enter => {
            if !app.chat_input.trim().is_empty() {
                let content = app.chat_input.trim().to_string();
                app.chat_history.push(ChatMessage {
                    role: "user".to_string(),
                    content: content.clone(),
                });
                
                // Save after user message
                app.save_chat_history();
                
                // Send to AI
                if let Some(client) = &app.ai_client {
                    let (tx, rx) = mpsc::channel();
                    app.chat_receiver = Some(rx);
                    app.is_loading = true;
                    app.chat_auto_scroll = true;
                    
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

fn handle_focus_screen(key: KeyEvent, app: &mut App) -> Option<bool> {
    match key.code {
        KeyCode::Esc => {
            // Exit to main screen
            app.current_screen = CurrentScreen::Main;
        }
        KeyCode::Char('z') => {
            // Enter Zen mode (single task focus)
            app.current_screen = CurrentScreen::ZenMode;
        }
        KeyCode::Char('d') | KeyCode::Enter => {
            // Toggle task completion
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.toggle_complete_task(task_id);
                let _ = app.store.save();
                app.clamp_selected_index();
            }
        }
        KeyCode::Char('x') => {
            // Drop task
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.drop_task(task_id);
                let _ = app.store.save();
                app.clamp_selected_index();
            }
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
        KeyCode::PageDown => {
            let count = get_task_count(app);
            if count > 0 {
                app.selected_task_index = (app.selected_task_index + 5).min(count - 1);
            }
        }
        KeyCode::PageUp => {
            let count = get_task_count(app);
            if count > 0 {
                if app.selected_task_index >= 5 {
                    app.selected_task_index -= 5;
                } else {
                    app.selected_task_index = 0;
                }
            }
        }
        _ => {}
    }
    None
}

fn handle_zen_screen(key: KeyEvent, app: &mut App) -> Option<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('z') => {
            // Exit to focus screen
            app.current_screen = CurrentScreen::Focus;
        }
        KeyCode::Char('d') | KeyCode::Enter | KeyCode::Char(' ') => {
            // Mark done and move to next task
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.toggle_complete_task(task_id);
                let _ = app.store.save();
                app.clamp_selected_index();

                // Auto-advance to next task if available
                if get_task_count(app) == 0 {
                    // No more tasks, exit to focus view
                    app.current_screen = CurrentScreen::Focus;
                }
            }
        }
        KeyCode::Char('s') => {
            // Skip to next task without completing
            let count = get_task_count(app);
            if count > 0 {
                app.selected_task_index = (app.selected_task_index + 1) % count;
            }
        }
        KeyCode::Char('x') => {
            // Drop task and move to next
            if let Some(task_id) = get_selected_task_id(app) {
                app.store.drop_task(task_id);
                let _ = app.store.save();
                app.clamp_selected_index();

                // Auto-exit if no more tasks
                if get_task_count(app) == 0 {
                    app.current_screen = CurrentScreen::Focus;
                }
            }
        }
        KeyCode::Char('r') => {
            // Reset pomodoro timer
            if let Some(ref mut zen_state) = app.zen_state {
                zen_state.pomodoro = Some(Pomodoro::new(25)); // Reset to 25 minutes
                zen_state.message = String::from("Focus on what matters");
            }
        }
        _ => {}
    }
    None
}
