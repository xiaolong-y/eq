use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Widget},
    Frame,
};
use crate::tui::app::App;
use crate::models::task::{Quadrant, TaskStatus};
use unicode_width::UnicodeWidthStr;


pub fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Matrix
            Constraint::Length(3), // Footer/Input
        ].as_ref())
        .split(f.area());

    // Header
    let _titles = vec!["TODAY", "tomorrow"];
    let date_str = app.view_date.format("%a %b %d").to_string();
    let header_text = format!(" Xiaolong's Eisenhower Quadrants   {}   [?] [q]", date_str);
    
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

    // Helper to render quadrant
    let render_quad = |q: Quadrant, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer| {
        let mut q_tasks: Vec<_> = tasks.iter()
            .filter(|t| t.quadrant() == q)
            .collect();
        
        q_tasks.sort_by(|a, b| b.score().cmp(&a.score()));

        let is_active = app.selected_quadrant == q;
        
        // Custom rendering to handle selection highlight
        // We can't easily use the Widget trait if we need complex state logic inside render unless we pass it.
        // Let's manually render using the widget we created but we need to handle the selected item highlight.
        // The widget I wrote earlier was a bit too simple. Let's just inline the logic here or update the widget.
        // For now, let's just use Block and Paragraph for simplicity in this file, or update the widget.
        // I'll update the widget logic in my head: I'll just render manually here to ensure control.
        
        let border_style = if is_active && !app.input_mode {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let block = Block::default()
            .title(format!(" {} ", q))
            .borders(Borders::ALL)
            .border_style(border_style);
            
        block.render(area, buf);
        
        let inner = ratatui::layout::Rect {
            x: area.x + 1,
            y: area.y + 1,
            width: area.width.saturating_sub(2),
            height: area.height.saturating_sub(2),
        };

        for (i, task) in q_tasks.iter().enumerate() {
            if i >= inner.height as usize { break; }
            
            let mut style = Style::default();
            let mut prefix = "  ";
            
            if is_active && i == app.selected_task_index {
                style = style.add_modifier(Modifier::BOLD);
                prefix = "› ";
            }
            
            if task.status == TaskStatus::Completed {
                style = style.fg(Color::DarkGray).add_modifier(Modifier::CROSSED_OUT);
            } else {
                 match q {
                    Quadrant::DoFirst => style = style.fg(Color::Red),
                    Quadrant::Schedule => style = style.fg(Color::Blue),
                    Quadrant::Delegate => style = style.fg(Color::Yellow),
                    Quadrant::Drop => style = style.fg(Color::Gray),
                }
            }

            let content = format!("{}{:<width$} [{}]", prefix, task.title, task.score(), width = (inner.width as usize).saturating_sub(8));
            buf.set_string(inner.x, inner.y + i as u16, content, style);
        }
    };

    render_quad(Quadrant::DoFirst, top_row[0], f.buffer_mut());
    render_quad(Quadrant::Schedule, top_row[1], f.buffer_mut());
    render_quad(Quadrant::Delegate, bottom_row[0], f.buffer_mut());
    render_quad(Quadrant::Drop, bottom_row[1], f.buffer_mut());

    // Footer / Input
    if app.input_mode {
        let input = Paragraph::new(format!("Add Task: {}", app.input_buffer))
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL).title(" Input "));
        f.render_widget(input, chunks[2]);
    } else {
        let help = Paragraph::new("[a]dd  [d]one  [x]drop  [e]dit  [>]move  [↑↓←→]navigate  [tab]quadrant  [t]omorrow  [q]uit")
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
        
        // Clear the background of the popup
        f.render_widget(ratatui::widgets::Clear, area);
        f.render_widget(popup, area);
    }

    // Chat UI
    if let crate::tui::app::CurrentScreen::Chat = app.current_screen {
        let area = centered_rect(80, 80, f.area());
        f.render_widget(ratatui::widgets::Clear, area);
        
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ].as_ref())
            .split(area);

        let block = Block::default().borders(Borders::ALL).title(" Your friendly neighbor - chatgpt");
        f.render_widget(block, area);

        // Messages
        let mut messages_text = Vec::new();
        for msg in &app.chat_history {
            let role = if msg.role == "user" { "You" } else { "Chat" };
            let color = if msg.role == "user" { Color::Yellow } else { Color::Cyan };
            messages_text.push(Line::from(vec![
                ratatui::text::Span::styled(format!("{}: ", role), Style::default().fg(color).add_modifier(Modifier::BOLD)),
                ratatui::text::Span::raw(&msg.content),
            ]));
            messages_text.push(Line::from("")); // Spacing
        }
        
        if app.is_loading {
             messages_text.push(Line::from(ratatui::text::Span::styled("Minding your business...", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))));
        }

        // Scroll to bottom (basic implementation: take last N lines)
        // For a proper scroll, we'd need state, but let's just show what fits or last few.
        // Paragraph handles wrapping but not auto-scroll to bottom easily without offset state.
        // We'll just render it and hope it fits or user scrolls (we haven't implemented scroll keys for chat yet).
        // Let's just render all and let it clip top if too long (default behavior is clip bottom usually).
        // Actually, let's reverse it visually or just render.
        
        let messages_area = chunks[0].inner(ratatui::layout::Margin { vertical: 1, horizontal: 1 });
        
        // Calculate total height needed
        let width = messages_area.width as usize;
        let mut total_lines = 0;
        for msg in &app.chat_history {
            // Role line
            total_lines += 1; 
            // Content lines
            let wrapped = textwrap::wrap(&msg.content, width);
            total_lines += wrapped.len();
            // Spacing
            total_lines += 1;
        }
        if app.is_loading {
            total_lines += 1;
        }

        let height = messages_area.height as usize;
        let max_scroll = if total_lines > height {
            (total_lines - height) as u16
        } else {
            0
        };

        if app.chat_auto_scroll {
            app.chat_scroll = max_scroll;
        } else {
            // Clamp scroll
            if app.chat_scroll > max_scroll {
                app.chat_scroll = max_scroll;
            }
        }

        let messages = Paragraph::new(messages_text)
            .wrap(ratatui::widgets::Wrap { trim: true })
            .scroll((app.chat_scroll, 0));
            
        f.render_widget(messages, messages_area);

        // Input
        let input_area = chunks[1].inner(ratatui::layout::Margin { vertical: 0, horizontal: 1 });
        let input_width = input_area.width as usize;
        
        // Calculate input lines and cursor position
        let input_lines = textwrap::wrap(&app.chat_input, input_width);
        let total_input_lines = input_lines.len();
        
        // Account for Borders::TOP (1 line)
        let input_height = (input_area.height as usize).saturating_sub(1);
        let input_scroll = if total_input_lines > input_height {
            (total_input_lines - input_height) as u16
        } else {
            0
        };

        let input = Paragraph::new(app.chat_input.as_str())
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::TOP).title(" Message "))
            .wrap(ratatui::widgets::Wrap { trim: false })
            .scroll((input_scroll, 0));
        f.render_widget(input, input_area);

        // Set Cursor
        if app.input_mode || matches!(app.current_screen, crate::tui::app::CurrentScreen::Chat) {
             // Only show cursor in chat if we are in chat screen
             // Actually app.input_mode is for the main editing. Chat has its own state.
             // But we are inside the Chat screen block.
             
             // Find cursor coordinates
             // The cursor is at the end of the text.
             if let Some(last_line) = input_lines.last() {
                 let cursor_x = input_area.x + last_line.width() as u16;
                 let cursor_y = input_area.y + (total_input_lines as u16).saturating_sub(1).saturating_sub(input_scroll);
                 
                 // Ensure cursor is within input area
                 if cursor_y >= input_area.y && cursor_y < input_area.y + input_area.height {
                     f.set_cursor_position((cursor_x, cursor_y));
                 }
             } else {
                 // Empty input
                 f.set_cursor_position((input_area.x, input_area.y));
             }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: ratatui::layout::Rect) -> ratatui::layout::Rect {
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
