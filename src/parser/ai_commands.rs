use crate::parser::input::parse_priority;

#[derive(Debug, PartialEq, Clone)]
pub enum AICommand {
    Add(ParsedTask),
    Done(TaskIdentifier),
    Drop(TaskIdentifier),
    Edit {
        target: TaskIdentifier,
        new_title: Option<String>,
        new_urgency: Option<u8>,
        new_importance: Option<u8>,
    },
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParsedTask {
    pub title: String,
    pub urgency: u8,
    pub importance: u8,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TaskIdentifier {
    Index(usize),
    Title(String),
}

#[derive(Default, Debug)]
pub struct CommandResults {
    pub tasks_added: Vec<ParsedTask>,
    pub tasks_completed: Vec<String>,
    pub tasks_dropped: Vec<String>,
    pub tasks_edited: Vec<String>,
    pub errors: Vec<String>,
}

impl CommandResults {
    pub fn format_confirmation(&self) -> String {
        let mut msg = String::from("\n━━━ Command Results ━━━\n");
        
        if !self.tasks_completed.is_empty() {
            msg.push_str("✓ Completed:\n");
            for t in &self.tasks_completed {
                msg.push_str(&format!("  • {}\n", t));
            }
        }
        
        if !self.tasks_dropped.is_empty() {
            msg.push_str("✓ Dropped:\n");
            for t in &self.tasks_dropped {
                msg.push_str(&format!("  • {}\n", t));
            }
        }
        
        if !self.tasks_added.is_empty() {
            msg.push_str("✓ Added:\n");
            for t in &self.tasks_added {
                msg.push_str(&format!("  • {} (u{}i{})\n", t.title, t.urgency, t.importance));
            }
        }

        if !self.tasks_edited.is_empty() {
            msg.push_str("✓ Edited:\n");
            for t in &self.tasks_edited {
                msg.push_str(&format!("  • {}\n", t));
            }
        }
        
        if !self.errors.is_empty() {
            msg.push_str("⚠ Errors:\n");
            for e in &self.errors {
                msg.push_str(&format!("  • {}\n", e));
            }
        }
        
        msg
    }
}

/// Parse all commands from an AI response
pub fn parse_commands(response: &str) -> Vec<AICommand> {
    let mut commands = Vec::new();
    
    for line in response.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("[ADD]") {
            if let Some(task) = parse_add_command(rest.trim()) {
                commands.push(AICommand::Add(task));
            }
        } else if let Some(rest) = trimmed.strip_prefix("[DONE]") {
            if let Some(id) = parse_task_identifier(rest.trim()) {
                commands.push(AICommand::Done(id));
            }
        } else if let Some(rest) = trimmed.strip_prefix("[DROP]") {
            if let Some(id) = parse_task_identifier(rest.trim()) {
                commands.push(AICommand::Drop(id));
            }
        } else if let Some(rest) = trimmed.strip_prefix("[EDIT]") {
            if let Some(edit) = parse_edit_command(rest.trim()) {
                commands.push(edit);
            }
        }
    }

    commands
}

/// Parse [ADD] command
fn parse_add_command(input: &str) -> Option<ParsedTask> {
    if input.is_empty() {
        return None;
    }

    let mut urgency = 1u8;
    let mut importance = 1u8;
    let mut title_parts = Vec::new();

    for word in input.split_whitespace() {
        if let Some((u, i)) = parse_priority(word) {
            urgency = u;
            importance = i;
        } else {
            title_parts.push(word);
        }
    }

    let title = title_parts.join(" ");
    if title.is_empty() {
        return None;
    }

    Some(ParsedTask {
        title,
        urgency,
        importance,
    })
}

/// Parse task identifier (title or #index)
fn parse_task_identifier(input: &str) -> Option<TaskIdentifier> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Check for #N index syntax
    if let Some(rest) = trimmed.strip_prefix('#') {
        if let Ok(idx) = rest.trim().parse::<usize>() {
            return Some(TaskIdentifier::Index(idx));
        }
    }

    // Check for just a number
    if let Ok(idx) = trimmed.parse::<usize>() {
        return Some(TaskIdentifier::Index(idx));
    }

    // Otherwise treat as title fragment
    Some(TaskIdentifier::Title(trimmed.to_string()))
}

/// Parse [EDIT] command
/// Format: [EDIT] old title -> new title u2i3
/// Or: [EDIT] old title u2i3 (just change priority)
fn parse_edit_command(input: &str) -> Option<AICommand> {
    if input.is_empty() {
        return None;
    }

    // Check for arrow syntax: "old title -> new title u2i3"
    if let Some((left, right)) = input.split_once("->") {
        let target = parse_task_identifier(left.trim())?;

        // Parse the right side for new title and priority
        let mut new_urgency = None;
        let mut new_importance = None;
        let mut title_parts = Vec::new();

        for word in right.trim().split_whitespace() {
            if let Some((u, i)) = parse_priority(word) {
                new_urgency = Some(u);
                new_importance = Some(i);
            } else {
                title_parts.push(word);
            }
        }

        let new_title = if title_parts.is_empty() {
            None
        } else {
            Some(title_parts.join(" "))
        };

        return Some(AICommand::Edit {
            target,
            new_title,
            new_urgency,
            new_importance,
        });
    }

    // No arrow: "task title u2i3" - just update priority
    let mut urgency = None;
    let mut importance = None;
    let mut title_parts = Vec::new();

    for word in input.split_whitespace() {
        if let Some((u, i)) = parse_priority(word) {
            urgency = Some(u);
            importance = Some(i);
        } else {
            title_parts.push(word);
        }
    }

    let title = title_parts.join(" ");
    if title.is_empty() {
        return None;
    }

    Some(AICommand::Edit {
        target: TaskIdentifier::Title(title),
        new_title: None,
        new_urgency: urgency,
        new_importance: importance,
    })
}

// ============================================================================
// Legacy API for backward compatibility
// ============================================================================

/// Extract all [ADD] commands from an AI response (legacy)
pub fn parse_add_commands(response: &str) -> Vec<ParsedTask> {
    parse_commands(response)
        .into_iter()
        .filter_map(|cmd| match cmd {
            AICommand::Add(task) => Some(task),
            _ => None,
        })
        .collect()
}

/// Format a confirmation message for added tasks (legacy)
pub fn format_task_confirmation(tasks: &[ParsedTask]) -> String {
    if tasks.is_empty() {
        return String::new();
    }

    let mut msg = String::from("\n━━━ ✓ Tasks Added ━━━\n");
    for task in tasks {
        msg.push_str(&format!(
            "• {} (u{}i{})\n",
            task.title, task.urgency, task.importance
        ));
    }
    msg
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_add() {
        let cmds = parse_commands("[ADD] Review notes u2i3");
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            AICommand::Add(task) => {
                assert_eq!(task.title, "Review notes");
                assert_eq!(task.urgency, 2);
                assert_eq!(task.importance, 3);
            }
            _ => panic!("Expected Add command"),
        }
    }

    #[test]
    fn test_parse_done_by_title() {
        let cmds = parse_commands("[DONE] Fix server crash");
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            AICommand::Done(TaskIdentifier::Title(t)) => {
                assert_eq!(t, "Fix server crash");
            }
            _ => panic!("Expected Done with title"),
        }
    }

    #[test]
    fn test_parse_done_by_index() {
        let cmds = parse_commands("[DONE] #1");
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            AICommand::Done(TaskIdentifier::Index(1)) => {}
            _ => panic!("Expected Done with index 1"),
        }

        let cmds = parse_commands("[DONE] 2");
        match &cmds[0] {
            AICommand::Done(TaskIdentifier::Index(2)) => {}
            _ => panic!("Expected Done with index 2"),
        }
    }

    #[test]
    fn test_parse_drop() {
        let cmds = parse_commands("[DROP] Scroll Twitter");
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            AICommand::Drop(TaskIdentifier::Title(t)) => {
                assert_eq!(t, "Scroll Twitter");
            }
            _ => panic!("Expected Drop command"),
        }
    }

    #[test]
    fn test_parse_edit_with_arrow() {
        let cmds = parse_commands("[EDIT] Old task -> New task name u3i2");
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            AICommand::Edit {
                target: TaskIdentifier::Title(t),
                new_title: Some(new),
                new_urgency: Some(3),
                new_importance: Some(2),
            } => {
                assert_eq!(t, "Old task");
                assert_eq!(new, "New task name");
            }
            _ => panic!("Expected Edit command with arrow"),
        }
    }

    #[test]
    fn test_parse_edit_priority_only() {
        let cmds = parse_commands("[EDIT] Some task u1i3");
        assert_eq!(cmds.len(), 1);
        match &cmds[0] {
            AICommand::Edit {
                target: TaskIdentifier::Title(t),
                new_title: None,
                new_urgency: Some(1),
                new_importance: Some(3),
            } => {
                assert_eq!(t, "Some task");
            }
            _ => panic!("Expected Edit command with priority only"),
        }
    }

    #[test]
    fn test_parse_multiple_commands() {
        let response = r#"Here's what I'll do:
[ADD] New task u2i2
[DONE] Old task
[DROP] Useless task
Done!"#;
        let cmds = parse_commands(response);
        assert_eq!(cmds.len(), 3);
        assert!(matches!(cmds[0], AICommand::Add(_)));
        assert!(matches!(cmds[1], AICommand::Done(_)));
        assert!(matches!(cmds[2], AICommand::Drop(_)));
    }

    #[test]
    fn test_command_results_format() {
        let mut results = CommandResults::default();
        results.tasks_added.push(ParsedTask {
            title: "Task A".into(),
            urgency: 2,
            importance: 3,
        });
        results.tasks_completed.push("Task B".into());
        results.tasks_dropped.push("Task C".into());

        let msg = results.format_confirmation();
        assert!(msg.contains("Task A"));
        assert!(msg.contains("Completed"));
        assert!(msg.contains("Task B"));
        assert!(msg.contains("Dropped"));
    }
}
