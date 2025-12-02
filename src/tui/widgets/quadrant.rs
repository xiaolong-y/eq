use crate::models::task::{Quadrant, Task, TaskStatus};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
};

/// Fix #3: Refactored QuadrantWidget that's actually used by ui.rs
pub struct QuadrantWidget<'a> {
    pub tasks: Vec<&'a Task>,
    pub active: bool,
    pub quadrant_type: Quadrant,
    pub selected_index: Option<usize>,
}

impl<'a> QuadrantWidget<'a> {
    pub fn new(
        tasks: Vec<&'a Task>,
        active: bool,
        quadrant_type: Quadrant,
        selected_index: Option<usize>,
    ) -> Self {
        Self {
            tasks,
            active,
            quadrant_type,
            selected_index,
        }
    }

    fn get_quadrant_color(&self) -> Color {
        match self.quadrant_type {
            Quadrant::DoFirst => Color::Red,
            Quadrant::Schedule => Color::Blue,
            Quadrant::Delegate => Color::Yellow,
            Quadrant::Drop => Color::Gray,
        }
    }
}

impl<'a> Widget for QuadrantWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.active {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let title = format!(" {} ", self.quadrant_type);
        let block = Block::default()
            .title(title)
            .borders(Borders::ALL)
            .border_style(border_style);

        block.render(area, buf);

        let inner = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        let height = inner.height as usize;

        // Calculate scroll offset to ensure selected task is visible
        let start_index = if let Some(sel_idx) = self.selected_index {
            if sel_idx >= height {
                sel_idx - height + 1
            } else {
                0
            }
        } else {
            0
        };

        for (i, task) in self.tasks.iter().enumerate().skip(start_index) {
            let render_index = i - start_index;
            if render_index >= height {
                break;
            }

            let is_selected = self.selected_index == Some(i);

            let mut style = Style::default();
            let prefix = if is_selected { "› " } else { "  " };

            if is_selected {
                style = style.add_modifier(Modifier::BOLD);
            }

            if task.status == TaskStatus::Completed {
                style = style
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT);
            } else {
                style = style.fg(self.get_quadrant_color());
            }

            // Format: "› Task Title      [15]"
            let score_str = format!("[{}]", task.score());
            let max_title_width = (inner.width as usize)
                .saturating_sub(prefix.len())
                .saturating_sub(score_str.len())
                .saturating_sub(1); // Space before score

            let title = if task.title.len() > max_title_width {
                format!("{}…", &task.title[..max_title_width.saturating_sub(1)])
            } else {
                task.title.clone()
            };

            let padding = max_title_width.saturating_sub(title.len());
            let content = format!("{}{}{} {}", prefix, title, " ".repeat(padding), score_str);

            buf.set_string(inner.x, inner.y + render_index as u16, &content, style);
        }

        // Show count if there are more items than visible
        if self.tasks.len() > height {
            let more = self.tasks.len() - height;
            let indicator = format!("… +{} more", more);
            let style = Style::default().fg(Color::DarkGray);
            let x = inner.right().saturating_sub(indicator.len() as u16 + 1);
            let y = inner.bottom().saturating_sub(1);
            if y >= inner.y && x >= inner.x {
                buf.set_string(x, y, &indicator, style);
            }
        }
    }
}
