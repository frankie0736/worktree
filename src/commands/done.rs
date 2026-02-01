use crate::error::{Result, WtError};
use crate::models::{TaskStatus, TaskStore};
use crate::services::tmux;

pub fn execute(name: String) -> Result<()> {
    let mut store = TaskStore::load()?;

    // Check task exists
    let _task = store
        .get(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;

    let current_status = store.get_status(&name);
    if !current_status.can_transition_to(&TaskStatus::Done) {
        return Err(WtError::InvalidStateTransition {
            from: current_status.display_name().to_string(),
            to: TaskStatus::Done.display_name().to_string(),
        });
    }

    // Close tmux window if still alive
    if let Some(instance) = store.get_instance(&name) {
        if tmux::window_exists(&instance.tmux_session, &instance.tmux_window) {
            tmux::kill_window(&instance.tmux_session, &instance.tmux_window)?;
            println!("Closed tmux window {}:{}", instance.tmux_session, instance.tmux_window);
        }
    }

    store.set_status(&name, TaskStatus::Done);
    store.save_status()?;

    println!("Task '{}' marked as done.", name);
    println!("After PR is merged, run: wt merged {}", name);
    Ok(())
}
