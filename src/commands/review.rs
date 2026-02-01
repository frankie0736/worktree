//! Review command - view task results and optionally continue conversation.

use std::path::Path;

use serde::Serialize;

use crate::display::format_duration;
use crate::error::{Result, WtError};
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::{git, tmux, transcript};

/// JSON output for review command
#[derive(Serialize)]
struct ReviewOutput {
    task: String,
    status: String,
    worktree_path: String,
    session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    metrics: Option<MetricsOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_changes: Option<CodeChangesOutput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    commands: CommandsOutput,
}

#[derive(Serialize)]
struct MetricsOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_secs: Option<i64>,
    input_tokens: u64,
    output_tokens: u64,
    num_turns: u32,
    context_percent: u8,
}

#[derive(Serialize)]
struct CodeChangesOutput {
    insertions: i32,
    deletions: i32,
}

#[derive(Serialize)]
struct CommandsOutput {
    interactive: String,
    non_interactive: String,
}

pub fn execute(name: String, json: bool) -> Result<()> {
    let _config = WtConfig::load()?;
    let mut store = TaskStore::load()?;

    // Check task exists
    let _task = store
        .get(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;

    // Get instance info (needed for tmux check)
    let instance = store
        .get_instance(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?
        .clone();

    // Check status
    let status = store.get_status(&name);
    match status {
        TaskStatus::Running => {
            // Check if tmux window still exists
            if tmux::window_exists(&instance.tmux_session, &instance.tmux_window) {
                // Still running, can't review
                return Err(WtError::CannotReviewRunning(name));
            }
            // Tmux window gone, agent has exited - auto mark as Done
            if !json {
                println!("Agent has exited. Auto-marking task as done...");
            }
            store.set_status(&name, TaskStatus::Done);
            store.save_status()?;
        }
        TaskStatus::Pending | TaskStatus::Merged => return Err(WtError::TaskNotDone(name)),
        TaskStatus::Done => {}
    }

    // Re-get instance after potential status update
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

    // Parse transcript
    let metrics = transcript::parse_transcript(&transcript_path);

    // Build commands
    let interactive_cmd = format!("cd {} && claude -r {}", worktree_path, session_id);
    let non_interactive_cmd = format!(
        "cd {} && claude --output-format stream-json -r {} -p \"继续完成任务\"",
        worktree_path, session_id
    );

    if json {
        output_json(
            &name,
            worktree_path,
            session_id,
            &metrics,
            &interactive_cmd,
            &non_interactive_cmd,
        );
    } else {
        output_text(
            &name,
            worktree_path,
            &metrics,
            &interactive_cmd,
            &non_interactive_cmd,
        );
    }

    Ok(())
}

fn output_json(
    name: &str,
    worktree_path: &str,
    session_id: &str,
    metrics: &Option<transcript::TranscriptMetrics>,
    interactive_cmd: &str,
    non_interactive_cmd: &str,
) {
    let code_changes = git::get_diff_stats(worktree_path).and_then(|(adds, dels)| {
        if adds > 0 || dels > 0 {
            Some(CodeChangesOutput {
                insertions: adds,
                deletions: dels,
            })
        } else {
            None
        }
    });

    let metrics_output = metrics.as_ref().map(|m| MetricsOutput {
        duration_secs: m.duration_secs(),
        input_tokens: m.input_tokens,
        output_tokens: m.output_tokens,
        num_turns: m.num_turns,
        context_percent: m.context_percent(),
    });

    let summary = metrics.as_ref().and_then(|m| m.summary.clone());

    let output = ReviewOutput {
        task: name.to_string(),
        status: "done".to_string(),
        worktree_path: worktree_path.to_string(),
        session_id: session_id.to_string(),
        metrics: metrics_output,
        code_changes,
        summary,
        commands: CommandsOutput {
            interactive: interactive_cmd.to_string(),
            non_interactive: non_interactive_cmd.to_string(),
        },
    };

    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

fn output_text(
    name: &str,
    worktree_path: &str,
    metrics: &Option<transcript::TranscriptMetrics>,
    interactive_cmd: &str,
    non_interactive_cmd: &str,
) {
    println!("Task: {} (Done)", name);
    println!();

    if let Some(ref m) = metrics {
        // Duration from transcript timestamps
        if let Some(secs) = m.duration_secs() {
            println!("Duration: {}", format_duration(secs));
        }

        // Statistics
        println!();
        println!("## Statistics");
        println!(
            "  Input: {} tokens | Output: {} tokens | Turns: {}",
            m.input_tokens, m.output_tokens, m.num_turns
        );
        println!("  Context usage: {}%", m.context_percent());

        // Code changes
        if let Some((adds, dels)) = git::get_diff_stats(worktree_path) {
            if adds > 0 || dels > 0 {
                println!();
                println!("## Code Changes");
                println!("  +{} -{}", adds, dels);
            }
        }

        // Summary
        if let Some(ref summary) = m.summary {
            println!();
            println!("## Result");
            // Truncate if too long
            let display_summary = if summary.len() > 2000 {
                format!("{}...", &summary[..2000])
            } else {
                summary.clone()
            };
            println!("{}", display_summary);
        }
    } else {
        println!("(Unable to parse transcript)");
    }

    // Output commands for continuing conversation
    println!();
    println!("---");
    println!("# Resume conversation (interactive, for human):");
    println!("{}", interactive_cmd);
    println!();
    println!("# Continue with prompt (for agent):");
    println!("{}", non_interactive_cmd);
}
