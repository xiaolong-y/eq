use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};
use crate::models::task::{Task, Quadrant};

pub struct QuadrantWidget<'a> {
    pub title: String,
    pub tasks: Vec<&'a Task>,
    pub active: bool,
    pub quadrant_type: Quadrant,
}


impl<'a> QuadrantWidget<'a> {
    pub fn new(tasks: Vec<&'a Task>, active: bool, quadrant_type: Quadrant) -> Self {
        let title = format!(" {} ", quadrant_type);
        Self {
            title,
            tasks,
            active,
            quadrant_type,
        }
    }
}

impl<'a> Widget for QuadrantWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.active {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(self.title)
            .borders(Borders::ALL)
            .border_style(border_style);

        block.render(area, buf);

        // Render tasks inside
        let inner_area = Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        for (i, task) in self.tasks.iter().enumerate() {
            if i >= inner_area.height as usize {
                break;
            }

            let task_style = if task.status == crate::models::task::TaskStatus::Completed {
                Style::default().fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT)
            } else {
                match self.quadrant_type {
                    Quadrant::DoFirst => Style::default().fg(Color::Red),
                    Quadrant::Schedule => Style::default().fg(Color::Blue),
                    Quadrant::Delegate => Style::default().fg(Color::Yellow),
                    Quadrant::Drop => Style::default().fg(Color::Gray),
                }
            };

            // Highlight if active and selected (logic handled in parent or here if we passed index)
            // For simplicity, we just render the list. Selection highlighting logic needs to be passed in.
            // Let's assume the parent handles the "selected" state by passing a modified style or prefix.
            // Actually, `QuadrantWidget` should probably take `selected_index` if it's active.
            
            let line = Line::from(vec![
                Span::styled(format!("{} ", task.title), task_style),
                Span::styled(format!("[{}]", task.score()), Style::default().fg(Color::DarkGray)),
            ]);
            
            buf.set_line(inner_area.x, inner_area.y + i as u16, &line, inner_area.width);
        }
    }
}
