use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use directories::ProjectDirs;

const DATA_DIR_NAME: &str = "data";
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
    if let Some(env_dir) = env::var_os(ENV_DATA_DIR) {
        return Ok(PathBuf::from(env_dir));
    }

    if let Some(repo_dir) = repo_root_from_exe() {
        return Ok(repo_dir.join(DATA_DIR_NAME));
    }

    if let Some(project_dirs) = ProjectDirs::from(QUALIFIER, ORGANIZATION, APPLICATION) {
        return Ok(project_dirs.data_local_dir().to_path_buf());
    }

    // Fallback to current directory if everything else fails.
    Ok(env::current_dir()?.join(DATA_DIR_NAME))
}

fn repo_root_from_exe() -> Option<PathBuf> {
    let exe = env::current_exe().ok()?;
    let mut current = exe.parent();

    while let Some(dir) = current {
        if is_project_root(dir) {
            return Some(dir.to_path_buf());
        }
        current = dir.parent();
    }

    None
}

fn is_project_root(dir: &Path) -> bool {
    dir.join("Cargo.toml").exists() && dir.join("src").exists()
}
