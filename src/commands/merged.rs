use crate::error::Result;
use crate::models::{TaskStatus, TaskStore};
use crate::services::tmux;

pub fn execute(name: String) -> Result<()> {
    let mut store = TaskStore::load()?;

    // Check task exists
    store.ensure_exists(&name)?;

    let current_status = store.get_status(&name);
    if !current_status.can_transition_to(&TaskStatus::Merged) {
        println!(
            "Warning: Task '{}' was in {} state (expected done or running).",
            name,
            current_status.display_name()
        );
    }

    // Only close tmux window, keep worktree and branch for review
    if let Some(instance) = store.get_instance(&name) {
        if let Err(e) = tmux::kill_window(&instance.tmux_session, &instance.tmux_window) {
            eprintln!("  Warning: Failed to kill tmux window: {}", e);
        } else {
            println!("  Closed tmux window: {}:{}", instance.tmux_session, instance.tmux_window);
        }
        // Keep instance data for archive command
    }

    store.set_status(&name, TaskStatus::Merged);
    // Keep instance (worktree_path, branch) for archive
    store.save_status()?;

    println!("Task '{}' marked as merged.", name);
    println!("Worktree and branch preserved for review.");
    println!("Run 'wt archive {}' to cleanup resources.", name);
    Ok(())
}
