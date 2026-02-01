use crate::error::{Result, WtError};
use crate::models::{TaskStatus, TaskStore};
use crate::services::{git, tmux};

pub fn execute(name: String) -> Result<()> {
    let mut store = TaskStore::load()?;

    // Check task exists
    let _task = store
        .get(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;

    let current_status = store.get_status(&name);
    if !current_status.can_transition_to(&TaskStatus::Merged) {
        println!(
            "Warning: Task '{}' was in {} state (expected done or running).",
            name,
            current_status.display_name()
        );
    }

    if let Some(instance) = store.get_instance(&name) {
        println!("Cleaning up resources...");

        if let Err(e) = tmux::kill_window(&instance.tmux_session, &instance.tmux_window) {
            eprintln!("  Warning: Failed to kill tmux window: {}", e);
        }

        if let Err(e) = git::remove_worktree(&instance.worktree_path) {
            eprintln!("  Warning: Failed to remove worktree: {}", e);
        }

        if let Err(e) = git::delete_branch(&instance.branch) {
            eprintln!("  Warning: Failed to delete branch: {}", e);
        }

        println!("  Removed worktree: {}", instance.worktree_path);
        println!("  Deleted branch: {}", instance.branch);
    }

    store.set_status(&name, TaskStatus::Merged);
    store.set_instance(&name, None);
    store.save_status()?;

    println!("Task '{}' marked as merged.", name);
    println!("Dependent tasks can now be started.");
    Ok(())
}
