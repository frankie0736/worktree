use std::path::Path;

use crate::error::Result;
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::{git, tmux, workspace::WorkspaceInitializer};

pub fn execute(name: String) -> Result<()> {
    let config = WtConfig::load()?;
    let mut store = TaskStore::load()?;

    // Check task exists and validate transition
    store.ensure_exists(&name)?;
    store.validate_transition(&name, TaskStatus::Archived)?;

    // Run archive script if configured
    if let Some(ref script) = config.archive_script {
        if let Some(instance) = store.get_instance(&name) {
            println!("Running archive script...");
            let source_dir = Path::new(".");
            let initializer = WorkspaceInitializer::new(&instance.worktree_path, source_dir);
            initializer.run_init_script(script)?;
        }
    }

    // Cleanup all resources
    if let Some(instance) = store.get_instance(&name) {
        println!("Archiving resources...");

        // Kill tmux window (may already be gone from merged)
        let _ = tmux::kill_window(&instance.tmux_session, &instance.tmux_window);

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

    // Update status and clear instance
    store.set_status(&name, TaskStatus::Archived);
    store.set_instance(&name, None);
    store.save_status()?;

    println!("Task '{}' archived.", name);
    Ok(())
}
