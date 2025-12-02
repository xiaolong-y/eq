use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Quadrant {
    DoFirst,
    Schedule,
    Delegate,
    Drop,
}

impl fmt::Display for Quadrant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Quadrant::DoFirst => write!(f, "DO FIRST"),
            Quadrant::Schedule => write!(f, "SCHEDULE"),
            Quadrant::Delegate => write!(f, "DELEGATE"),
            Quadrant::Drop => write!(f, "DROP"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    Completed,
    Dropped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Uuid,
    pub title: String,
    pub urgency: u8,
    pub importance: u8,
    pub status: TaskStatus,
    pub date: NaiveDate,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub fn new(title: String, urgency: u8, importance: u8, date: NaiveDate) -> Self {
        Self {
            id: Uuid::new_v4(),
            title,
            urgency: urgency.clamp(1, 3),
            importance: importance.clamp(1, 3),
            status: TaskStatus::Pending,
            date,
            created_at: Utc::now(),
            completed_at: None,
        }
    }

    pub fn score(&self) -> u8 {
        (self.importance * 3) + (self.urgency * 2)
    }

    pub fn quadrant(&self) -> Quadrant {
        if self.importance >= 2 && self.urgency >= 2 {
            Quadrant::DoFirst
        } else if self.importance >= 2 && self.urgency == 1 {
            Quadrant::Schedule
        } else if self.importance == 1 && self.urgency >= 2 {
            Quadrant::Delegate
        } else {
            Quadrant::Drop
        }
    }

    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.completed_at = Some(Utc::now());
    }

    pub fn undo_complete(&mut self) {
        self.status = TaskStatus::Pending;
        self.completed_at = None;
    }

    pub fn drop_task(&mut self) {
        self.status = TaskStatus::Dropped;
    }
}
