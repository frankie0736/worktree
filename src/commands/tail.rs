//! Tail command - view last assistant messages from task transcript.

use std::path::Path;

use serde::Serialize;

use crate::error::{Result, WtError};
use crate::models::{TaskStatus, TaskStore};
use crate::services::transcript;

#[derive(Serialize)]
struct Message {
    role: &'static str,
    content: String,
}

pub fn execute(name: String, count: usize) -> Result<()> {
    let store = TaskStore::load()?;

    // Check task exists
    let _task = store
        .get(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;

    // Check status - only Pending is not allowed
    let status = store.get_status(&name);
    if status == TaskStatus::Pending {
        return Err(WtError::TaskNotStarted(name));
    }

    // Get instance info
    let instance = store
        .get_instance(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;

    // Check worktree exists
    let worktree_path = &instance.worktree_path;
    if !Path::new(worktree_path).exists() {
        return Err(WtError::WorktreeNotFound(name));
    }

    // Check session_id exists
    let session_id = instance
        .session_id
        .as_ref()
        .ok_or_else(|| WtError::NoSessionId(name.clone()))?;

    // Find transcript file
    let transcript_path = transcript::transcript_path(worktree_path, session_id)
        .ok_or_else(|| WtError::TranscriptNotFound(name.clone()))?;

    if !transcript_path.exists() {
        return Err(WtError::TranscriptNotFound(name.clone()));
    }

    // Get last N messages
    let messages = transcript::get_last_messages(&transcript_path, count)
        .ok_or_else(|| WtError::TranscriptParseFailed(name.clone()))?;

    if messages.is_empty() {
        return Err(WtError::NoAssistantMessages(name));
    }

    // Always output JSON
    let output: Vec<Message> = messages
        .iter()
        .map(|content| Message {
            role: "assistant",
            content: content.clone(),
        })
        .collect();

    println!("{}", serde_json::to_string_pretty(&output).unwrap());

    Ok(())
}
