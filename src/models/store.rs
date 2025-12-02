use crate::models::task::{Task, TaskStatus};
use crate::models::log::{append_log, LogEvent, EventAction};

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;
use chrono::NaiveDate;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TaskStore {
    pub tasks: Vec<Task>,
}

/// Chat message for persistence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl TaskStore {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let path = Self::get_path()?;
        
        if !path.exists() {
            return Ok(TaskStore::default());
        }

        let content = fs::read_to_string(path)?;
        let store = serde_json::from_str(&content)?;
        Ok(store)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Self::get_path()?;
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        
        // Atomic write: write to .tmp then rename
        let tmp_path = path.with_extension("tmp");
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?; // Ensure written to disk
        
        fs::rename(tmp_path, path)?;
        Ok(())
    }

    fn get_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(PathBuf::from("data").join("tasks.json"))
    }

    /// Load chat history from file
    pub fn load_chat_history() -> Vec<ChatMessage> {
        let path = PathBuf::from("data").join("chat_history.json");
        if !path.exists() {
            return Vec::new();
        }
        
        match fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Vec::new(),
        }
    }

    /// Save chat history to file
    pub fn save_chat_history(history: &[ChatMessage]) -> Result<(), Box<dyn std::error::Error>> {
        let path = PathBuf::from("data").join("chat_history.json");
        
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(history)?;
        let tmp_path = path.with_extension("tmp");
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?;
        
        fs::rename(tmp_path, path)?;
        Ok(())
    }

    pub fn add_task(&mut self, task: Task) {
        let event = LogEvent::new(EventAction::Created, task.id, format!("Created task: {}", task.title));
        let _ = append_log(&event);
        self.tasks.push(task);
    }

    pub fn toggle_complete_task(&mut self, id: Uuid) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            if task.status == TaskStatus::Completed {
                task.undo_complete();
                let event = LogEvent::new(EventAction::Updated, id, format!("Undone task: {}", task.title));
                let _ = append_log(&event);
            } else {
                task.complete();
                let event = LogEvent::new(EventAction::Completed, id, format!("Completed task: {}", task.title));
                let _ = append_log(&event);
            }
            return true;
        }
        false
    }

    pub fn complete_task(&mut self, id: Uuid) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            if task.status != TaskStatus::Completed {
                task.complete();
                let event = LogEvent::new(EventAction::Completed, id, format!("Completed task: {}", task.title));
                let _ = append_log(&event);
                return true;
            }
        }
        false
    }

    pub fn drop_task(&mut self, id: Uuid) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            if task.status != TaskStatus::Dropped {
                task.drop_task();
                let event = LogEvent::new(EventAction::Dropped, id, format!("Dropped task: {}", task.title));
                let _ = append_log(&event);
                return true;
            }
        }
        false
    }

    pub fn update_task(&mut self, id: Uuid, title: String, urgency: u8, importance: u8) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            let old_details = format!("{} (u{}i{})", task.title, task.urgency, task.importance);
            task.title = title;
            task.urgency = urgency;
            task.importance = importance;
            let new_details = format!("{} (u{}i{})", task.title, task.urgency, task.importance);
            
            let event = LogEvent::new(EventAction::Updated, id, format!("Updated: {} -> {}", old_details, new_details));
            let _ = append_log(&event);
            return true;
        }
        false
    }

    pub fn move_task_to_date(&mut self, id: Uuid, date: NaiveDate) -> bool {
        if let Some(task) = self.tasks.iter_mut().find(|t| t.id == id) {
            let old_date = task.date;
            task.date = date;
            let event = LogEvent::new(EventAction::Moved, id, format!("Moved: {} -> {}", old_date, date));
            let _ = append_log(&event);
            return true;
        }
        false
    }

    /// Find a task by ID prefix or index (Fix #6 - simplified)
    pub fn find_task_id(&self, id_or_index: &str, filter_date: Option<NaiveDate>) -> Option<Uuid> {
        // Try to parse as 1-based index
        if let Ok(idx) = id_or_index.parse::<usize>() {
            let mut tasks: Vec<&Task> = self.tasks.iter()
                .filter(|t| {
                    t.status == TaskStatus::Pending && 
                    filter_date.map_or(true, |d| t.date == d)
                })
                .collect();
            tasks.sort_by_key(|t| std::cmp::Reverse(t.score()));
            
            if idx > 0 && idx <= tasks.len() {
                return Some(tasks[idx - 1].id);
            }
        }
        
        // Fallback to UUID prefix match
        self.tasks.iter()
            .find(|t| t.id.to_string().starts_with(id_or_index))
            .map(|t| t.id)
    }
}
