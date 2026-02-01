use std::time::SystemTime;

use serde::Serialize;

use crate::constants::IDLE_THRESHOLD_SECS;
use crate::display::format_duration;
use crate::error::Result;
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::{git, tmux, transcript};

#[derive(Serialize)]
struct TaskMetrics {
    name: String,
    status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_secs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_human: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additions: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deletions: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commits: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idle_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tmux_alive: Option<bool>,
}

#[derive(Serialize)]
struct StatusOutput {
    tasks: Vec<TaskMetrics>,
    summary: StatusSummary,
}

#[derive(Serialize)]
struct StatusSummary {
    running: usize,
    done: usize,
    total_additions: i32,
    total_deletions: i32,
}

pub fn execute(json: bool) -> Result<()> {
    // Verify we're in a wt project directory
    WtConfig::load()?;

    if json {
        // JSON output for agents/scripts
        display_status(true)
    } else if atty::is(atty::Stream::Stdout) {
        // Interactive TUI mode (default for humans)
        let action = crate::tui::run()?;
        handle_tui_action(action)
    } else {
        // Non-TTY: auto-degrade to JSON
        display_status(true)
    }
}

fn handle_tui_action(action: crate::tui::TuiAction) -> Result<()> {
    use crate::tui::TuiAction;
    use std::process::Command;

    match action {
        TuiAction::Quit => Ok(()),
        TuiAction::SwitchTmuxWindow { .. } => {
            // This should be handled within TUI, not here
            Ok(())
        }
        TuiAction::AttachTmux { session, window } => {
            // Outside tmux: directly attach to session
            Command::new("tmux")
                .args(["attach", "-t", &format!("{}:{}", session, window)])
                .status()
                .ok();
            Ok(())
        }
        TuiAction::ShowResume {
            worktree,
            session_id,
            claude_command,
        } => {
            eprintln!("Tmux window closed. Run this command to resume:");
            println!("cd {} && {} -r {}", worktree, claude_command, session_id);
            Ok(())
        }
        TuiAction::Review { name } => {
            // Execute review command
            crate::commands::review::execute(name, false)
        }
    }
}

fn display_status(json: bool) -> Result<()> {
    let mut store = TaskStore::load()?;
    let config = WtConfig::load().ok();

    let mut metrics_list = Vec::new();
    let mut running_count = 0;
    let mut done_count = 0;
    let mut total_additions = 0;
    let mut total_deletions = 0;
    let mut tasks_to_mark_done: Vec<String> = Vec::new();

    for task in store.list() {
        let status = store.get_status(task.name());

        // Only show Running and Done tasks
        if status != TaskStatus::Running && status != TaskStatus::Done {
            continue;
        }

        let instance = store.get_instance(task.name());

        // Check if tmux window is alive (for auto-done detection)
        let tmux_alive = instance
            .map(|i| tmux::window_exists(&i.tmux_session, &i.tmux_window))
            .unwrap_or(false);

        // Auto-mark as Done if Running but tmux window gone
        let final_status = if status == TaskStatus::Running && !tmux_alive {
            tasks_to_mark_done.push(task.name().to_string());
            TaskStatus::Done
        } else {
            status
        };

        if final_status == TaskStatus::Running {
            running_count += 1;
        } else {
            done_count += 1;
        }

        let instance = store.get_instance(task.name());
        let worktree_path = instance.map(|i| i.worktree_path.as_str());

        // Parse transcript for duration
        let transcript_metrics = instance.and_then(|inst| {
            inst.session_id
                .as_ref()
                .and_then(|sid| transcript::transcript_path(&inst.worktree_path, sid))
                .and_then(|path| transcript::parse_transcript(&path))
        });

        // Duration from transcript timestamps
        let (duration_secs, duration_human) = transcript_metrics
            .as_ref()
            .and_then(|m| m.duration_secs())
            .map(|secs| (Some(secs), Some(format_duration(secs))))
            .unwrap_or((None, None));

        // Get diff stats
        let (additions, deletions) = worktree_path
            .and_then(git::get_diff_stats)
            .unwrap_or((0, 0));

        total_additions += additions;
        total_deletions += deletions;

        // Get commit count
        let commits = worktree_path
            .and_then(|p| {
                config.as_ref().and_then(|_| git::get_commit_count(p, "HEAD~100"))
            });

        // tmux_alive for JSON output (only meaningful for running tasks)
        let tmux_alive_for_output = if final_status == TaskStatus::Running {
            Some(tmux_alive)
        } else {
            None
        };

        // Get idle time and activity status
        let (idle_secs, active) = if let Some(path) = worktree_path {
            if let Some(last_activity) = git::get_last_activity(path) {
                let idle = SystemTime::now()
                    .duration_since(last_activity)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                let is_active = idle < IDLE_THRESHOLD_SECS;
                (Some(idle), Some(is_active))
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        metrics_list.push(TaskMetrics {
            name: task.name().to_string(),
            status: final_status,
            duration_secs,
            duration_human,
            additions: if additions > 0 { Some(additions) } else { None },
            deletions: if deletions > 0 { Some(deletions) } else { None },
            commits,
            idle_secs,
            active,
            tmux_alive: tmux_alive_for_output,
        });
    }

    // Auto-mark tasks as Done
    if !tasks_to_mark_done.is_empty() {
        for name in &tasks_to_mark_done {
            store.set_status(name, TaskStatus::Done);
        }
        store.save_status()?;
    }

    let output = StatusOutput {
        tasks: metrics_list,
        summary: StatusSummary {
            running: running_count,
            done: done_count,
            total_additions,
            total_deletions,
        },
    };

    if json {
        println!("{}", serde_json::to_string_pretty(&output).unwrap_or_default());
    } else {
        print_human_readable(&output);
    }

    Ok(())
}

fn print_human_readable(output: &StatusOutput) {
    if output.tasks.is_empty() {
        println!("No running or done tasks.");
        return;
    }

    println!("Tasks:");
    println!();

    for task in &output.tasks {
        // tmux_alive takes precedence: if window is dead, show warning
        let status_indicator = match task.tmux_alive {
            Some(false) => " ⚠️  (tmux window closed)",
            _ => match task.active {
                Some(true) => " 🟢",
                Some(false) => " 💤",
                None => "",
            },
        };

        println!(
            "{} {} ({}){}",
            task.status.icon(),
            task.name,
            task.status.display_name(),
            status_indicator
        );

        if let Some(ref duration) = task.duration_human {
            println!("    Duration: {}", duration);
        }

        let has_changes = task.additions.is_some() || task.deletions.is_some();
        if has_changes {
            let adds = task.additions.unwrap_or(0);
            let dels = task.deletions.unwrap_or(0);
            println!("    Changes:  +{} -{}", adds, dels);
        }

        if let Some(commits) = task.commits {
            if commits > 0 {
                println!("    Commits:  {}", commits);
            }
        }

        println!();
    }

    println!("---");
    println!(
        "Summary: {} running, {} done | +{} -{}",
        output.summary.running,
        output.summary.done,
        output.summary.total_additions,
        output.summary.total_deletions
    );
}

