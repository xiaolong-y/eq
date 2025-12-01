use clap::Parser;
use eq::cli::{Cli, Commands};
use eq::models::store::TaskStore;
use eq::models::task::Task;
use eq::parser::input::parse_priority;
use chrono::{Local, NaiveDate, Duration};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
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
            println!("Added task: {} (U={}, I={}) -> {}", task.title, task.urgency, task.importance, task.quadrant());
            store.add_task(task);
            store.save()?;
        }
        Some(Commands::Done { id }) => {
            if let Some(task) = find_task(&mut store, id) {
                let task_id = task.id;
                store.complete_task(task_id);
                println!("Marked task as done: {}", id);
                store.save()?;
            } else {
                println!("Task not found: {}", id);
            }
        }
        Some(Commands::Drop { id }) => {
            if let Some(task) = find_task(&mut store, id) {
                let task_id = task.id;
                store.drop_task(task_id);
                println!("Dropped task: {}", id);
                store.save()?;
            } else {
                println!("Task not found: {}", id);
            }
        }
        Some(Commands::Edit { id, args }) => {
            if let Some(task) = find_task(&mut store, id) {
                let task_id = task.id;
                let current_title = task.title.clone();
                let mut urgency = task.urgency;
                let mut importance = task.importance;
                
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
            println!("Weekly view not implemented in CLI yet.");
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
    use std::collections::HashMap;
    use eq::models::task::{Quadrant, TaskStatus};

    let mut counts: HashMap<Quadrant, usize> = HashMap::new();
    let mut durations: HashMap<Quadrant, i64> = HashMap::new(); // Total seconds

    for task in &store.tasks {
        if task.status == TaskStatus::Completed {
            *counts.entry(task.quadrant()).or_default() += 1;
            
            if let Some(completed_at) = task.completed_at {
                let duration = completed_at.signed_duration_since(task.created_at).num_seconds();
                *durations.entry(task.quadrant()).or_default() += duration;
            }
        }
    }

    println!("\n📊 Productivity Stats (Completed Tasks)\n");
    
    let quadrants = [Quadrant::DoFirst, Quadrant::Schedule, Quadrant::Delegate, Quadrant::Drop];
    
    println!("Task Counts:");
    let max_count = counts.values().max().copied().unwrap_or(0);
    for q in &quadrants {
        let count = counts.get(q).copied().unwrap_or(0);
        let bar_len = if max_count > 0 { (count as f64 / max_count as f64 * 20.0) as usize } else { 0 };
        let bar = "█".repeat(bar_len);
        println!("{:<10} | {:<3} {}", q.to_string(), count, bar);
    }

    println!("\nAvg Time to Complete (Seconds):");
    let _max_duration = durations.values().max().copied().unwrap_or(0); // This is total, need avg for chart? 
    // Let's chart Average Duration
    
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
        let bar_len = if max_avg > 0 { (avg as f64 / max_avg as f64 * 20.0) as usize } else { 0 };
        let bar = "█".repeat(bar_len);
        println!("{:<10} | {:<5} {}", q.to_string(), avg, bar);
    }
    println!();
}

fn find_task<'a>(store: &'a mut TaskStore, id: &str) -> Option<&'a mut Task> {
    // Try to parse as index (1-based)
    if let Ok(idx) = id.parse::<usize>() {
        // This is tricky because indices depend on the view.
        // For CLI, we might need a way to map visual index to ID.
        // For now, let's assume UUID prefix or exact match if not number.
        // But the prompt examples use "eq done 3".
        // This implies we need a stable sort or the user is looking at a list.
        // Let's just support UUID prefix for now to be safe, or implement a lookup.
        // Actually, without a stateful view, index-based access is dangerous in CLI.
        // But let's try to match the "Today" view order.
        let today = Local::now().date_naive();
        let mut tasks: Vec<&mut Task> = store.tasks.iter_mut()
            .filter(|t| t.date == today && t.status == eq::models::task::TaskStatus::Pending)
            .collect();
        
        tasks.sort_by(|a, b| b.score().cmp(&a.score()));
        
        if idx > 0 && idx <= tasks.len() {
            // This is a bit unsafe if the list changed, but standard for CLI tools.
            // We need to find the task with the same ID in the real store.
            // Since we have mutable references, we can't easily return one from the filtered list 
            // and then use it because of borrowing rules if we constructed a new vector.
            // Let's just find the ID first.
            let target_id = tasks[idx-1].id;
            return store.tasks.iter_mut().find(|t| t.id == target_id);
        }
    }
    
    // Fallback to UUID prefix
    store.tasks.iter_mut().find(|t| t.id.to_string().starts_with(id))
}



fn print_matrix(store: &TaskStore, date: NaiveDate) {
    println!("Eisenhower Matrix for {}", date);
    // Filter and sort
    let mut tasks: Vec<&Task> = store.tasks.iter()
        .filter(|t| t.date == date && t.status == eq::models::task::TaskStatus::Pending)
        .collect();
    tasks.sort_by(|a, b| b.score().cmp(&a.score()));

    if tasks.is_empty() {
        println!("No pending tasks.");
        return;
    }

    for (i, task) in tasks.iter().enumerate() {
        println!("{}. [{}] {} (Score: {})", i + 1, task.quadrant(), task.title, task.score());
    }
}
