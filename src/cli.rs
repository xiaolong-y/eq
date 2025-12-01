use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "eq")]
#[command(about = "Eisenhower Quadrants - A terminal-based task manager", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add a new task
    Add {
        /// Task title and priority notation (e.g., "Buy milk !!$$")
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,

        /// Schedule for tomorrow
        #[arg(long, short)]
        tomorrow: bool,
    },
    
    /// Mark a task as done
    Done {
        /// Task ID or index
        id: String,
    },

    /// Drop (delete) a task
    Drop {
        /// Task ID or index
        id: String,
    },

    /// Edit a task's priority
    Edit {
        /// Task ID or index
        id: String,
        
        /// New priority notation (e.g., u3i2)
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Show today's matrix (default)
    Today,

    /// Show tomorrow's matrix
    Tomorrow,

    /// Show weekly overview
    Week,

    /// Launch interactive TUI
    /// Launch interactive TUI
    Tui,

    /// Show productivity statistics
    Stats,
}
