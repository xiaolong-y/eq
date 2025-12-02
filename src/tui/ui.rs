use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};
use crate::tui::app::{App, CurrentScreen};
use crate::tui::widgets::quadrant::QuadrantWidget;
use crate::tui::zen::ZenState;
use crate::models::task::{Quadrant, TaskStatus};

pub fn ui(f: &mut Frame, app: &mut App) {
    // Handle special screen modes
    match app.current_screen {
        CurrentScreen::Chat => {
            render_chat(f, app);
            return;
        }
        CurrentScreen::Focus => {
            render_focus(f, app);
            return;
        }
        CurrentScreen::ZenMode => {
            render_zen(f, app);
            return;
        }
        _ => {}
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Matrix
            Constraint::Length(3), // Footer/Input
        ].as_ref())
        .split(f.area());

    // Header
    let date_str = app.view_date.format("%a %b %d").to_string();
    let header_text = format!(" Xiaolong's Eisenhower Quadrants   {}    [?] [q]", date_str);

    let header = Paragraph::new(header_text)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Main Matrix (2x2)
    let matrix_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[1]);

    let top_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(matrix_chunks[0]);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(matrix_chunks[1]);

    // Filter tasks for current view
    let tasks: Vec<_> = app.store.tasks.iter()
        .filter(|t| t.date == app.view_date && t.status != TaskStatus::Dropped)
        .collect();

    // Fix #3: Use QuadrantWidget for rendering
    render_quadrant(f, Quadrant::DoFirst, top_row[0], &tasks, app);
    render_quadrant(f, Quadrant::Schedule, top_row[1], &tasks, app);
    render_quadrant(f, Quadrant::Delegate, bottom_row[0], &tasks, app);
    render_quadrant(f, Quadrant::Drop, bottom_row[1], &tasks, app);

    // Footer / Input
    if app.input_mode {
        let input = Paragraph::new(format!("Add Task: {}", app.input_buffer))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Input "));
        f.render_widget(input, chunks[2]);
        
        // Show cursor for input
        let x = chunks[2].x + 11 + app.input_buffer.len() as u16;
        let y = chunks[2].y + 1;
        f.set_cursor_position((x.min(chunks[2].right() - 2), y));
    } else {
        let help = Paragraph::new("[a]dd  [d]one  [x]drop  [e]dit  [z]focus  [>]move  [↑↓←→]navigate  [tab]quadrant  [t]omorrow  [c]hat  [q]uit")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::TOP));
        f.render_widget(help, chunks[2]);
    }

    // Easter Egg Popup
    if app.show_help {
        let area = centered_rect(60, 20, f.area());
        let text = "Hey, if you wonder if you need one more productivity tool to be more productive, the answer is probably no. Alas, we are here.";
        let popup = Paragraph::new(text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title(" Wisdom "))
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(Clear, area);
        f.render_widget(popup, area);
    }
}

/// Fix #3: Refactored to use QuadrantWidget
fn render_quadrant(
    f: &mut Frame,
    q: Quadrant,
    area: Rect,
    all_tasks: &[&crate::models::task::Task],
    app: &App,
) {
    let mut q_tasks: Vec<_> = all_tasks.iter()
        .filter(|t| t.quadrant() == q)
        .cloned()
        .collect();
    q_tasks.sort_by_key(|t| std::cmp::Reverse(t.score()));

    let is_active = app.selected_quadrant == q && !app.input_mode;
    let selected_idx = if is_active { Some(app.selected_task_index) } else { None };

    let widget = QuadrantWidget::new(q_tasks, is_active, q, selected_idx);
    f.render_widget(widget, area);
}

fn render_chat(f: &mut Frame, app: &mut App) {
    let area = centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(3),
        ].as_ref())
        .split(area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" AI Chat (Esc to close) ");
    f.render_widget(block, area);

    // Messages area
    let messages_area = chunks[0].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
    
    // Build message lines with wrapping
    let width = messages_area.width as usize;
    let mut lines: Vec<Line> = Vec::new();
    
    for msg in &app.chat_history {
        let (role, color) = if msg.role == "user" { 
            ("You", Color::Yellow) 
        } else { 
            ("AI", Color::Cyan) 
        };
        
        // Role header
        lines.push(Line::from(Span::styled(
            format!("{}:", role),
            Style::default().fg(color).add_modifier(Modifier::BOLD)
        )));
        
        // Wrap content
        let wrapped = textwrap::wrap(&msg.content, width.saturating_sub(2));
        for line in wrapped {
            lines.push(Line::from(Span::raw(format!("  {}", line))));
        }
        lines.push(Line::from("")); // Spacing
    }
    
    if app.is_loading {
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let frame = frames[app.spinner_state as usize % frames.len()];
        lines.push(Line::from(Span::styled(
            format!("{} Thinking...", frame),
            Style::default().fg(Color::Green)
        )));
    }

    // Calculate scroll
    let height = messages_area.height as usize;
    let total_lines = lines.len();
    let max_scroll = if total_lines > height {
        (total_lines - height) as u16
    } else {
        0
    };

    if app.chat_auto_scroll {
        app.chat_scroll = max_scroll;
    } else if app.chat_scroll > max_scroll {
        app.chat_scroll = max_scroll;
    }

    let messages = Paragraph::new(lines)
        .scroll((app.chat_scroll, 0));
    f.render_widget(messages, messages_area);

    // Scroll indicator
    if max_scroll > 0 {
        let scroll_pct = if max_scroll > 0 {
            (app.chat_scroll as f32 / max_scroll as f32 * 100.0) as u16
        } else {
            100
        };
        let indicator = if app.chat_auto_scroll {
            String::from("AUTO")
        } else {
            format!("{}%", scroll_pct)
        };
        let indicator_span = Span::styled(
            indicator,
            Style::default().fg(Color::DarkGray)
        );
        let x = messages_area.right().saturating_sub(6);
        let y = messages_area.top();
        f.buffer_mut().set_span(x, y, &indicator_span, 6);
    }

    // Input area
    let input_area = chunks[1].inner(ratatui::layout::Margin { vertical: 0, horizontal: 1 });
    let input_block = Block::default()
        .borders(Borders::TOP)
        .title(" Message (PgUp/PgDn to scroll, Ctrl+L clear) ");
    
    let input = Paragraph::new(app.chat_input.as_str())
        .style(Style::default().fg(Color::White))
        .block(input_block);
    f.render_widget(input, input_area);

    // Fix #5: Show cursor in chat input
    let cursor_x = input_area.x + app.chat_input.len() as u16;
    let cursor_y = input_area.y + 1;
    f.set_cursor_position((cursor_x.min(input_area.right() - 1), cursor_y));

    // Fix #5: Chat help overlay
    if app.show_chat_help {
        let help_area = centered_rect(50, 40, f.area());
        f.render_widget(Clear, help_area);
        
        let help_text = vec![
            Line::from(Span::styled("Chat Keyboard Shortcuts", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("PgUp/PgDn    Scroll history"),
            Line::from("Ctrl+K/J     Scroll one line"),
            Line::from("Home         Jump to top"),
            Line::from("End          Resume auto-scroll"),
            Line::from("Ctrl+L       Clear chat history"),
            Line::from("Ctrl+W       Delete word"),
            Line::from("Ctrl+U       Clear input"),
            Line::from("Esc          Close chat"),
            Line::from(""),
            Line::from(Span::styled("Press ? to close", Style::default().fg(Color::DarkGray))),
        ];
        
        let help = Paragraph::new(help_text)
            .block(Block::default().borders(Borders::ALL).title(" Help "))
            .alignment(Alignment::Left);
        f.render_widget(help, help_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ].as_ref())
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ].as_ref())
        .split(popup_layout[1])[1]
}

fn render_focus(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Quadrant content
            Constraint::Length(3), // Footer
        ].as_ref())
        .split(f.area());

    // Header
    let quadrant_name = match app.selected_quadrant {
        Quadrant::DoFirst => "DO NOW - Urgent & Important",
        Quadrant::Schedule => "SCHEDULE - Important, Not Urgent",
        Quadrant::Delegate => "DELEGATE - Urgent, Not Important",
        Quadrant::Drop => "ELIMINATE - Neither Urgent nor Important",
    };

    let header = Paragraph::new(format!(" FOCUS MODE: {}   [z] Zen Mode  [Esc] Exit ", quadrant_name))
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(header, chunks[0]);

    // Quadrant content (full screen)
    let tasks: Vec<_> = app.store.tasks.iter()
        .filter(|t| t.date == app.view_date && t.status != TaskStatus::Dropped)
        .collect();

    let mut q_tasks: Vec<_> = tasks.iter()
        .filter(|t| t.quadrant() == app.selected_quadrant)
        .cloned()
        .collect();
    q_tasks.sort_by_key(|t| std::cmp::Reverse(t.score()));

    let widget = QuadrantWidget::new(q_tasks, true, app.selected_quadrant, Some(app.selected_task_index));
    f.render_widget(widget, chunks[1]);

    // Footer
    let footer = Paragraph::new("[↑↓]navigate  [d/Enter]done  [x]drop  [z]zen  [Esc]exit")
        .style(Style::default().fg(Color::DarkGray))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::TOP));
    f.render_widget(footer, chunks[2]);
}

fn render_zen(f: &mut Frame, app: &mut App) {
    // Initialize zen state if needed
    let area = f.area();
    if app.zen_state.is_none() {
        app.zen_state = Some(ZenState::new(area.width, area.height, 25)); // 25 min pomodoro
    }

    // Update and render zen state (particles and pomodoro)
    if let Some(ref mut zen_state) = app.zen_state {
        zen_state.update(area.width, area.height);
        zen_state.render(area, f.buffer_mut());
    }

    // Get the current task
    let tasks: Vec<_> = app.store.tasks.iter()
        .filter(|t| {
            t.date == app.view_date
            && t.status != TaskStatus::Dropped
            && t.quadrant() == app.selected_quadrant
        })
        .collect();

    let mut sorted_tasks = tasks.clone();
    sorted_tasks.sort_by_key(|t| std::cmp::Reverse(t.score()));

    let current_task = if app.selected_task_index < sorted_tasks.len() {
        Some(sorted_tasks[app.selected_task_index])
    } else {
        None
    };

    // Render task on top of particles in centered area
    let task_area = centered_rect(80, 40, area);

    if let Some(task) = current_task {
        // Task title style - add strikethrough if completed
        let title_style = if task.status == TaskStatus::Completed {
            Style::default()
                .fg(Color::Rgb(120, 120, 120))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::CROSSED_OUT)
        } else {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
        };

        let task_lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                &task.title,
                title_style,
            )),
            Line::from(""),
            Line::from(Span::styled(
                format!("Urgency: {}  •  Importance: {}", task.urgency, task.importance),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
        ];

        let task_display = Paragraph::new(task_lines)
            .alignment(Alignment::Center)
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(task_display, task_area);
    } else {
        let empty_lines = vec![
            Line::from(""),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled(
                "No tasks in this quadrant.",
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press Esc to return.",
                Style::default().fg(Color::Yellow),
            )),
        ];

        let empty_display = Paragraph::new(empty_lines)
            .alignment(Alignment::Center);
        f.render_widget(empty_display, task_area);
    }
}
