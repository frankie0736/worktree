//! Logs command - extract filtered transcripts for all tasks.

use crate::error::Result;
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::transcript;

pub fn execute() -> Result<()> {
    let config = WtConfig::load()?;
    let store = TaskStore::load()?;

    let mut generated = 0;
    let mut skipped = 0;

    for task in store.list() {
        let status = store.get_status(task.name());

        // Skip pending tasks (no transcript)
        if status == TaskStatus::Pending {
            continue;
        }

        // Get instance info
        let instance = match store.get_instance(task.name()) {
            Some(i) => i,
            None => {
                skipped += 1;
                continue;
            }
        };

        // Need session_id
        let session_id = match &instance.session_id {
            Some(id) => id,
            None => {
                skipped += 1;
                continue;
            }
        };

        // Find transcript file
        let transcript_path = match transcript::transcript_path(&instance.worktree_path, session_id)
        {
            Some(p) if p.exists() => p,
            _ => {
                skipped += 1;
                continue;
            }
        };

        // Generate log file
        let log_path = transcript::log_path(task.name(), session_id);

        match transcript::extract_to_log(
            &transcript_path,
            &log_path,
            &config.logs.exclude_types,
            &config.logs.exclude_fields,
        ) {
            Some(count) => {
                println!(
                    "  {} -> {} ({} entries)",
                    task.name(),
                    log_path.display(),
                    count
                );
                generated += 1;
            }
            None => {
                skipped += 1;
            }
        }
    }

    println!();
    println!("Generated: {}, Skipped: {}", generated, skipped);

    Ok(())
}
