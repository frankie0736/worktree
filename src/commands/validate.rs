use crate::constants::TASKS_DIR;
use crate::error::{Result, WtError};
use crate::models::TaskStore;

pub fn execute(name: Option<String>) -> Result<()> {
    let store = TaskStore::load()?;

    // If specific task name is given, check it exists
    if let Some(ref task_name) = name {
        if !store.tasks.contains_key(task_name) {
            return Err(WtError::TaskNotFound(task_name.clone()));
        }
    }

    if store.tasks.is_empty() {
        println!("No tasks found in {}/", TASKS_DIR);
        return Ok(());
    }

    let errors = store.validate();

    // Filter by name if specified
    let errors: Vec<_> = if let Some(ref name) = name {
        errors
            .into_iter()
            .filter(|(n, _)| n == name || n.contains(name))
            .collect()
    } else {
        errors
    };

    if errors.is_empty() {
        let count = if name.is_some() { 1 } else { store.tasks.len() };
        println!("✓ All {} task(s) valid.", count);
    } else {
        for (task, error) in &errors {
            println!("✗ {}: {}", task, error);
        }
        println!();
        println!("{} error(s) found.", errors.len());
    }

    Ok(())
}
