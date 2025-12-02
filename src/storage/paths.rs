use std::env;
use std::fs;
use std::io;
use std::path::PathBuf;

use directories::ProjectDirs;

const ENV_DATA_DIR: &str = "EQ_DATA_DIR";
const QUALIFIER: &str = "dev";
const ORGANIZATION: &str = "quad_tasks";
const APPLICATION: &str = "eq";

/// Resolve the base directory for all persisted data.
pub fn data_dir() -> io::Result<PathBuf> {
    let path = determine_data_dir()?;
    fs::create_dir_all(&path)?;
    Ok(path)
}

/// Path to the primary tasks JSON file.
pub fn tasks_file_path() -> io::Result<PathBuf> {
    Ok(data_dir()?.join("tasks.json"))
}

/// Path to the chat history JSON file.
pub fn chat_history_path() -> io::Result<PathBuf> {
    Ok(data_dir()?.join("chat_history.json"))
}

/// Path to the event history log file.
pub fn history_log_path() -> io::Result<PathBuf> {
    Ok(data_dir()?.join("history.jsonl"))
}

fn determine_data_dir() -> io::Result<PathBuf> {
    // Priority 1: Explicit environment variable override
    if let Some(env_dir) = env::var_os(ENV_DATA_DIR) {
        return Ok(PathBuf::from(env_dir));
    }

    // Priority 2: OS-standard application data directory
    if let Some(project_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        return Ok(project_dirs.data_local_dir().to_path_buf());
    }

    // Should never reach here on modern systems, but provide helpful error
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "Could not determine data directory. Please set EQ_DATA_DIR environment variable.",
    ))
}
