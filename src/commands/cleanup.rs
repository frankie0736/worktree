use crate::error::Result;
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::{git, tmux};

pub fn execute(all: bool) -> Result<()> {
    let config = WtConfig::load()?;
    let mut store = TaskStore::load()?;

    let task_names: Vec<String> = store
        .tasks
        .keys()
        .filter(|name| {
            let has_instance = store.get_instance(name).is_some();
            if all {
                has_instance
            } else {
                store.get_status(name) == TaskStatus::Merged && has_instance
            }
        })
        .cloned()
        .collect();

    if task_names.is_empty() {
        println!("Nothing to clean up.");
        return Ok(());
    }

    for name in &task_names {
        if let Some(instance) = store.get_instance(name).cloned() {
            println!("Cleaning up '{}'...", name);

            if let Err(e) = tmux::kill_window(&instance.tmux_session, &instance.tmux_window) {
                eprintln!("  Warning: Failed to kill tmux window: {}", e);
            }

            if let Err(e) = git::remove_worktree(&instance.worktree_path) {
                eprintln!("  Warning: Failed to remove worktree: {}", e);
            }

            if let Err(e) = git::delete_branch(&instance.branch) {
                eprintln!("  Warning: Failed to delete branch: {}", e);
            }

            store.set_instance(name, None);
        }
    }

    store.save_status()?;

    if all && tmux::session_exists(&config.tmux_session) {
        println!("Killing tmux session '{}'...", config.tmux_session);
        if let Err(e) = tmux::kill_session(&config.tmux_session) {
            eprintln!("  Warning: Failed to kill tmux session: {}", e);
        }
    }

    println!("Cleanup complete.");
    Ok(())
}
