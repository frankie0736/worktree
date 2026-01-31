use std::fs;
use std::path::Path;

use crate::constants::log_path;
use crate::error::{Result, WtError};
use crate::models::{TaskStatus, TaskStore};
use crate::services::{dependency, git, tmux};

pub fn execute(name: String) -> Result<()> {
    let mut store = TaskStore::load()?;

    // Check task exists
    let _task = store
        .get(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;

    let current_status = store.get_status(&name);

    // If already Pending, silently succeed (idempotent)
    if current_status == TaskStatus::Pending {
        println!("Task '{}' is already pending.", name);
        return Ok(());
    }

    // Check for non-pending dependents
    let dependents = dependency::find_non_pending_dependents(&store, &name);
    if let Some((dep_name, dep_status)) = dependents.first() {
        return Err(WtError::HasDependents {
            task: name.clone(),
            dependent: dep_name.clone(),
            status: dep_status.display_name().to_string(),
        });
    }

    // Cleanup resources (best-effort)
    if let Some(instance) = store.get_instance(&name) {
        println!("Cleaning up resources...");

        // Kill tmux window
        if let Err(e) = tmux::kill_window(&instance.tmux_session, &instance.tmux_window) {
            eprintln!("  Warning: Failed to kill tmux window: {}", e);
        } else {
            println!("  Killed tmux window: {}:{}", instance.tmux_session, instance.tmux_window);
        }

        // Remove worktree
        if let Err(e) = git::remove_worktree(&instance.worktree_path) {
            eprintln!("  Warning: Failed to remove worktree: {}", e);
        } else {
            println!("  Removed worktree: {}", instance.worktree_path);
        }

        // Delete branch
        if let Err(e) = git::delete_branch(&instance.branch) {
            eprintln!("  Warning: Failed to delete branch: {}", e);
        } else {
            println!("  Deleted branch: {}", instance.branch);
        }
    }

    // Delete log file if exists
    let log_file = log_path(&name);
    if Path::new(&log_file).exists() {
        if let Err(e) = fs::remove_file(&log_file) {
            eprintln!("  Warning: Failed to delete log file: {}", e);
        } else {
            println!("  Deleted log: {}", log_file);
        }
    }

    // Update status to Pending and clear instance
    store.set_status(&name, TaskStatus::Pending);
    store.set_instance(&name, None);
    store.save_status()?;

    println!("Task '{}' reset to pending.", name);
    Ok(())
}
