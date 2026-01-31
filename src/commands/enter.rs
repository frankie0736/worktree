use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::error::{Result, WtError};
use crate::models::{TaskStore, WtConfig};
use crate::services::tmux;

pub fn execute(name: Option<String>) -> Result<()> {
    let config = WtConfig::load()?;
    let session = &config.tmux_session;

    // Check if session exists
    if !tmux::session_exists(session) {
        return Err(WtError::Tmux(format!(
            "Session '{}' does not exist. Start a task first with: wt start <task>",
            session
        )));
    }

    // If task name provided, verify it's running and window exists
    if let Some(ref task_name) = name {
        let store = TaskStore::load()?;

        // Check task exists
        let _task = store
            .get(task_name)
            .ok_or_else(|| WtError::TaskNotFound(task_name.clone()))?;

        // Check if task has an active instance (from StatusStore)
        if store.get_instance(task_name).is_none() {
            return Err(WtError::Tmux(format!(
                "Task '{}' is not running. Start it first with: wt start {}",
                task_name, task_name
            )));
        }

        // Check if tmux window actually exists (agent may have exited)
        if !tmux::window_exists(session, task_name) {
            return Err(WtError::Tmux(format!(
                "Window '{}' no longer exists (agent may have exited).\n\
                 The task status is still 'running'. You can:\n\
                 - Run `wt done {}` to mark it as done\n\
                 - Run `wt start {}` to restart it",
                task_name, task_name, task_name
            )));
        }
    }

    // Build tmux attach command
    let target = match &name {
        Some(task_name) => format!("{}:{}", session, task_name),
        None => session.clone(),
    };

    println!("Entering tmux session: {}", target);
    println!("(Press Ctrl+b d to detach)");

    // Replace current process with tmux attach
    let err = Command::new("tmux")
        .args(["attach-session", "-t", &target])
        .exec();

    // exec() only returns if there was an error
    Err(WtError::Tmux(format!("Failed to attach to tmux: {}", err)))
}

#[cfg(test)]
mod tests {
    // Note: Most tests for enter require actual tmux which is hard to test
    // The main logic is tested via CLI integration tests
}
