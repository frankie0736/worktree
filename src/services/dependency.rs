use crate::error::{Result, WtError};
use crate::models::{TaskStatus, TaskStore};

pub fn check_dependencies_merged(store: &TaskStore, task_name: &str) -> Result<()> {
    let task = store
        .get(task_name)
        .ok_or_else(|| WtError::TaskNotFound(task_name.to_string()))?;

    for dep_name in task.depends() {
        // Check dependency exists
        let _dep = store
            .get(dep_name)
            .ok_or_else(|| WtError::DependencyNotFound(dep_name.clone()))?;

        // Check dependency status from StatusStore
        if store.get_status(dep_name) != TaskStatus::Merged {
            return Err(WtError::DependencyNotMerged {
                task: task_name.to_string(),
                dep: dep_name.clone(),
            });
        }
    }
    Ok(())
}
