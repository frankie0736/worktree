use crate::error::Result;
use crate::models::{TaskStatus, TaskStore};
use crate::services::tmux;

pub fn execute(name: String) -> Result<()> {
    let mut store = TaskStore::load()?;

    // Check task exists and validate transition
    store.ensure_exists(&name)?;
    store.validate_transition(&name, TaskStatus::Done)?;

    // Close tmux window if still alive
    if let Some(instance) = store.get_instance(&name) {
        if tmux::kill_window_if_exists(&instance.tmux_session, &instance.tmux_window)? {
            println!("Closed tmux window {}:{}", instance.tmux_session, instance.tmux_window);
        }
    }

    store.set_status(&name, TaskStatus::Done);
    store.save_status()?;

    println!("Task '{}' marked as done.", name);
    println!("After PR is merged, run: wt merged {}", name);
    Ok(())
}
