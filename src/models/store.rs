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

    // Keep this for CLI explicit 'done' command if needed, or just use toggle?
    // CLI 'done' usually implies idempotent 'make it done'.
    // Let's keep a separate explicit complete if we want strictness, but for now I'll replace it 
    // or add a new one. The user asked for "double click on done undone", which is toggle.
    // I'll rename the existing one to `complete_task` (keep it) and add `toggle_complete_task`?
    // Actually, let's just modify `complete_task` to be `toggle_complete_task`?
    // No, `eq done` in CLI should probably not undo if run twice.
    // So I will ADD `toggle_completion` and keep `complete_task` (idempotent).
    
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
}
