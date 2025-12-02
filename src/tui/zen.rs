use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
};
use std::time::Instant;

/// A floating particle
#[derive(Clone)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub char: char,
    pub color: Color,
}

impl Particle {
    pub fn new(width: u16, height: u16) -> Self {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};

        // Simple random using hasher
        let mut hasher = RandomState::new().build_hasher();
        hasher.write_usize(Instant::now().elapsed().as_nanos() as usize);
        let rand1 = hasher.finish();
        hasher.write_u64(rand1);
        let rand2 = hasher.finish();
        hasher.write_u64(rand2);
        let rand3 = hasher.finish();

        let chars = ['·', '∘', '○', '◦', '•', '✦', '✧', '⋆', '˚', '✵'];
        let colors = [
            Color::Rgb(100, 120, 140),
            Color::Rgb(80, 100, 120),
            Color::Rgb(120, 140, 160),
            Color::Rgb(90, 110, 130),
            Color::Rgb(70, 90, 110),
        ];

        Self {
            x: (rand1 % width as u64) as f32,
            y: (rand2 % height as u64) as f32,
            vx: ((rand1 % 100) as f32 - 50.0) / 200.0,
            vy: ((rand2 % 100) as f32 - 50.0) / 300.0 - 0.05, // Slight upward bias
            char: chars[(rand3 % chars.len() as u64) as usize],
            color: colors[(rand1 % colors.len() as u64) as usize],
        }
    }

    pub fn update(&mut self, width: u16, height: u16) {
        self.x += self.vx;
        self.y += self.vy;

        // Wrap around screen
        if self.x < 0.0 { self.x = width as f32 - 1.0; }
        if self.x >= width as f32 { self.x = 0.0; }
        if self.y < 0.0 { self.y = height as f32 - 1.0; }
        if self.y >= height as f32 { self.y = 0.0; }
    }
}

/// Pomodoro timer state
pub struct Pomodoro {
    pub start: Instant,
    pub duration_secs: u64,
    pub is_break: bool,
}

impl Pomodoro {
    pub fn new(duration_mins: u64) -> Self {
        Self {
            start: Instant::now(),
            duration_secs: duration_mins * 60,
            is_break: false,
        }
    }

    pub fn elapsed_secs(&self) -> u64 {
        self.start.elapsed().as_secs()
    }

    pub fn remaining_secs(&self) -> u64 {
        self.duration_secs.saturating_sub(self.elapsed_secs())
    }

    pub fn progress(&self) -> f64 {
        (self.elapsed_secs() as f64 / self.duration_secs as f64).min(1.0)
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed_secs() >= self.duration_secs
    }

    pub fn format_remaining(&self) -> String {
        let remaining = self.remaining_secs();
        let mins = remaining / 60;
        let secs = remaining % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}

/// Zen mode state
pub struct ZenState {
    pub particles: Vec<Particle>,
    pub pomodoro: Option<Pomodoro>,
    pub tick: u64,
    pub message: String,
}

impl ZenState {
    pub fn new(width: u16, height: u16, duration_mins: u64) -> Self {
        let particle_count = ((width * height) / 80) as usize; // Sparse particles
        let particles = (0..particle_count)
            .map(|_| Particle::new(width, height))
            .collect();

        Self {
            particles,
            pomodoro: Some(Pomodoro::new(duration_mins)),
            tick: 0,
            message: String::from("Focus on what matters"),
        }
    }

    pub fn update(&mut self, width: u16, height: u16) {
        self.tick = self.tick.wrapping_add(1);

        for particle in &mut self.particles {
            particle.update(width, height);
        }

        // Check pomodoro completion
        if let Some(ref pomo) = self.pomodoro {
            if pomo.is_complete() && !pomo.is_break {
                self.message = String::from("Time for a break! 🍵");
            }
        }
    }

    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        // Render particles
        for particle in &self.particles {
            let x = particle.x as u16;
            let y = particle.y as u16;
            if x < area.width && y < area.height {
                buf.set_string(
                    area.x + x,
                    area.y + y,
                    particle.char.to_string(),
                    Style::default().fg(particle.color),
                );
            }
        }

        // Render pomodoro timer (centered)
        if let Some(ref pomo) = self.pomodoro {
            let center_x = area.x + area.width / 2;
            let center_y = area.y + area.height / 2;

            // Timer display
            let time_str = pomo.format_remaining();
            let time_x = center_x.saturating_sub(time_str.len() as u16 / 2);
            buf.set_string(
                time_x,
                center_y.saturating_sub(2),
                &time_str,
                Style::default().fg(Color::White).add_modifier(ratatui::style::Modifier::BOLD),
            );

            // Progress bar
            let bar_width = 30u16.min(area.width.saturating_sub(4));
            let bar_x = center_x.saturating_sub(bar_width / 2);
            let filled = (bar_width as f64 * pomo.progress()) as u16;

            let progress_color = if pomo.progress() < 0.75 {
                Color::Rgb(100, 180, 100)
            } else if pomo.progress() < 0.9 {
                Color::Rgb(180, 180, 100)
            } else {
                Color::Rgb(180, 100, 100)
            };

            for i in 0..bar_width {
                let char = if i < filled { "━" } else { "─" };
                let color = if i < filled { progress_color } else { Color::DarkGray };
                buf.set_string(bar_x + i, center_y, char, Style::default().fg(color));
            }

            // Message
            let msg_x = center_x.saturating_sub(self.message.len() as u16 / 2);
            buf.set_string(
                msg_x,
                center_y + 2,
                &self.message,
                Style::default().fg(Color::Rgb(150, 150, 170)),
            );

            // Breathing indicator
            let breath_chars = ["◯", "◎", "●", "◉", "●", "◎"];
            let breath_idx = ((self.tick / 8) % breath_chars.len() as u64) as usize;
            buf.set_string(
                center_x,
                center_y + 4,
                breath_chars[breath_idx],
                Style::default().fg(Color::Rgb(120, 140, 160)),
            );
        }

        // Instructions at bottom
        let help = "Press 'z' to exit · 'r' to reset timer";
        let help_x = area.x + area.width.saturating_sub(help.len() as u16) / 2;
        let help_y = area.y + area.height.saturating_sub(2);
        buf.set_string(help_x, help_y, help, Style::default().fg(Color::DarkGray));
    }
}
