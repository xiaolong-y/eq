use crate::storage::paths::history_log_path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub enum EventAction {
    Created,
    Completed,
    Dropped,
    Updated,
    Moved,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogEvent {
    pub id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub action: EventAction,
    pub task_id: Uuid,
    pub details: String,
}

impl LogEvent {
    pub fn new(action: EventAction, task_id: Uuid, details: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            timestamp: Utc::now(),
            action,
            task_id,
            details,
        }
    }
}

pub fn append_log(event: &LogEvent) -> std::io::Result<()> {
    let path = history_log_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;

    let json = serde_json::to_string(event)?;
    writeln!(file, "{}", json)?;
    Ok(())
}
