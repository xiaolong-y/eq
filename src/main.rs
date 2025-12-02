use chrono::{Datelike, Duration, Local, NaiveDate, Weekday};
use clap::Parser;
use eq::cli::{Cli, Commands};
use eq::models::store::TaskStore;
use eq::models::task::{Quadrant, Task, TaskStatus};
use eq::parser::input::parse_priority;
use std::collections::HashMap;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Load .env file from current directory
    dotenv::dotenv().ok();
    let cli = Cli::parse();
    let mut store = TaskStore::load()?;

    match &cli.command {
        Some(Commands::Add { args, tomorrow }) => {
            let mut urgency = 1;
            let mut importance = 1;
            let mut title_parts = Vec::new();

            for arg in args {
                if let Some((u, i)) = parse_priority(arg) {
                    urgency = u;
                    importance = i;
                } else {
                    title_parts.push(arg.clone());
                }
            }

            let title = title_parts.join(" ");

            let date = if *tomorrow {
                Local::now().date_naive() + Duration::days(1)
            } else {
                Local::now().date_naive()
            };

            let task = Task::new(title, urgency, importance, date);
            println!(
                "Added task: {} (U={}, I={}) -> {}",
                task.title,
                task.urgency,
                task.importance,
                task.quadrant()
            );
            store.add_task(task);
            store.save()?;
        }
        Some(Commands::Done { id }) => {
            let today = Local::now().date_naive();
            if let Some(task_id) = store.find_task_id(id, Some(today)) {
                store.complete_task(task_id);
                println!("Marked task as done: {}", id);
                store.save()?;
            } else {
                println!("Task not found: {}", id);
            }
        }
        Some(Commands::Drop { id }) => {
            let today = Local::now().date_naive();
            if let Some(task_id) = store.find_task_id(id, Some(today)) {
                store.drop_task(task_id);
                println!("Dropped task: {}", id);
                store.save()?;
            } else {
                println!("Task not found: {}", id);
            }
        }
        Some(Commands::Edit { id, args }) => {
            let today = Local::now().date_naive();
            if let Some(task_id) = store.find_task_id(id, Some(today)) {
                // Get current task info
                let (current_title, current_u, current_i) = {
                    let task = store.tasks.iter().find(|t| t.id == task_id).unwrap();
                    (task.title.clone(), task.urgency, task.importance)
                };

                let mut urgency = current_u;
                let mut importance = current_i;

                let input = args.join(" ");
                if let Some((u, i)) = parse_priority(&input) {
                    urgency = u;
                    importance = i;
                }

                store.update_task(task_id, current_title, urgency, importance);
                println!("Updated task: {}", id);
                store.save()?;
            } else {
                println!("Task not found: {}", id);
            }
        }
        Some(Commands::Today) | None => {
            print_matrix(&store, Local::now().date_naive());
        }
        Some(Commands::Tomorrow) => {
            print_matrix(&store, Local::now().date_naive() + Duration::days(1));
        }
        Some(Commands::Week) => {
            print_week(&store);
        }
        Some(Commands::Tui) => {
            eq::tui::app::run(&mut store)?;
        }
        Some(Commands::Stats) => {
            print_stats(&store);
        }
    }

    Ok(())
}

fn print_stats(store: &TaskStore) {
    let mut counts: HashMap<Quadrant, usize> = HashMap::new();
    let mut durations: HashMap<Quadrant, i64> = HashMap::new();

    for task in &store.tasks {
        if task.status == TaskStatus::Completed {
            *counts.entry(task.quadrant()).or_default() += 1;

            if let Some(completed_at) = task.completed_at {
                let duration = completed_at
                    .signed_duration_since(task.created_at)
                    .num_seconds();
                *durations.entry(task.quadrant()).or_default() += duration;
            }
        }
    }

    println!("\n📊 Productivity Stats (Completed Tasks)\n");

    let quadrants = [
        Quadrant::DoFirst,
        Quadrant::Schedule,
        Quadrant::Delegate,
        Quadrant::Drop,
    ];

    println!("Task Counts:");
    let max_count = counts.values().max().copied().unwrap_or(0);
    for q in &quadrants {
        let count = counts.get(q).copied().unwrap_or(0);
        let bar_len = if max_count > 0 {
            (count as f64 / max_count as f64 * 20.0) as usize
        } else {
            0
        };
        let bar = "█".repeat(bar_len);
        println!("{:<10} | {:<3} {}", q.to_string(), count, bar);
    }

    println!("\nAvg Time to Complete (Seconds):");

    let mut avgs = HashMap::new();
    for q in &quadrants {
        let count = counts.get(q).copied().unwrap_or(0);
        let total = durations.get(q).copied().unwrap_or(0);
        let avg = if count > 0 { total / count as i64 } else { 0 };
        avgs.insert(q, avg);
    }

    let max_avg = avgs.values().max().copied().unwrap_or(0);

    for q in &quadrants {
        let avg = avgs.get(q).copied().unwrap_or(0);
        let bar_len = if max_avg > 0 {
            (avg as f64 / max_avg as f64 * 20.0) as usize
        } else {
            0
        };
        let bar = "█".repeat(bar_len);
        println!("{:<10} | {:<5} {}", q.to_string(), avg, bar);
    }
    println!();
}

fn print_matrix(store: &TaskStore, date: NaiveDate) {
    println!("Eisenhower Matrix for {}", date);
    let mut tasks: Vec<&Task> = store
        .tasks
        .iter()
        .filter(|t| t.date == date && t.status == TaskStatus::Pending)
        .collect();
    tasks.sort_by_key(|b| std::cmp::Reverse(b.score()));

    if tasks.is_empty() {
        println!("No pending tasks.");
        return;
    }

    for (i, task) in tasks.iter().enumerate() {
        println!(
            "{}. [{}] {} (Score: {})",
            i + 1,
            task.quadrant(),
            task.title,
            task.score()
        );
    }
}

/// Fix #7: Week view implementation
fn print_week(store: &TaskStore) {
    let today = Local::now().date_naive();

    // Find start of week (Monday)
    let days_since_monday = today.weekday().num_days_from_monday();
    let week_start = today - Duration::days(days_since_monday as i64);

    println!(
        "\n📅 Week Overview ({} - {})\n",
        week_start.format("%b %d"),
        (week_start + Duration::days(6)).format("%b %d")
    );

    let weekdays = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];

    for (i, _weekday) in weekdays.iter().enumerate() {
        let date = week_start + Duration::days(i as i64);
        let is_today = date == today;

        let mut tasks: Vec<&Task> = store
            .tasks
            .iter()
            .filter(|t| t.date == date && t.status == TaskStatus::Pending)
            .collect();
        tasks.sort_by_key(|t| std::cmp::Reverse(t.score()));

        let completed: Vec<&Task> = store
            .tasks
            .iter()
            .filter(|t| t.date == date && t.status == TaskStatus::Completed)
            .collect();

        let marker = if is_today { "→" } else { " " };
        let day_name = date.format("%a %b %d").to_string();

        println!(
            "{} {} ({} pending, {} done)",
            marker,
            day_name,
            tasks.len(),
            completed.len()
        );

        // Show top 3 tasks for each day
        for task in tasks.iter().take(3) {
            let quadrant_icon = match task.quadrant() {
                Quadrant::DoFirst => "🔴",
                Quadrant::Schedule => "🔵",
                Quadrant::Delegate => "🟡",
                Quadrant::Drop => "⚪",
            };
            println!("    {} {}", quadrant_icon, task.title);
        }

        if tasks.len() > 3 {
            println!("    ... and {} more", tasks.len() - 3);
        }
        println!();
    }
}
