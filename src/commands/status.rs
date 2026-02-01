use std::collections::HashMap;
use std::time::SystemTime;

use serde::Serialize;

use crate::constants::IDLE_THRESHOLD_SECS;
use crate::display::format_duration;
use crate::error::Result;
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::{git, tmux, transcript};
use crate::tui::{App, TuiAction};

#[derive(Serialize)]
struct TaskMetrics {
    name: String,
    status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_secs: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_human: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    context_percent: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    current_tool: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    additions: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    deletions: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    commits: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    has_conflict: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    idle_secs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    active: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tmux_alive: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transcript_exists: Option<bool>,
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

// Action response structures
#[derive(Serialize)]
struct ActionResponse {
    action: String,
    success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    task: Option<TaskInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    available_actions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    unavailable_actions: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<CommandInfo>,
}

#[derive(Serialize)]
struct TaskInfo {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_before: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status_after: Option<TaskStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tmux_alive: Option<bool>,
}

#[derive(Serialize, Default)]
struct CommandInfo {
    #[serde(rename = "type")]
    cmd_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    window: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    worktree: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shell_command: Option<String>,
}

pub fn execute(json: bool, action: Option<String>, task: Option<String>) -> Result<()> {
    // Verify we're in a wt project directory
    WtConfig::load()?;

    // Handle --action parameter
    if let Some(action_name) = action {
        return execute_action(&action_name, task);
    }

    if json {
        // JSON output for agents/scripts
        display_status(true)
    } else if atty::is(atty::Stream::Stdout) {
        // Interactive TUI mode (default for humans)
        let tui_action = crate::tui::run()?;
        handle_tui_action(tui_action)
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
        TuiAction::Tail { name } => {
            // Execute tail command (default: 1 turn)
            crate::commands::tail::execute(name, 1)
        }
    }
}

fn display_status(json: bool) -> Result<()> {
    let mut store = TaskStore::load()?;

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

        // Get session_id and transcript path info
        let session_id = instance.and_then(|i| i.session_id.clone());

        // Try session_id first, fall back to finding latest transcript
        let transcript_path_opt = instance.and_then(|inst| {
            let path_from_id = inst
                .session_id
                .as_ref()
                .and_then(|sid| transcript::transcript_path(&inst.worktree_path, sid))
                .filter(|p| p.exists());

            path_from_id.or_else(|| transcript::find_latest_transcript(&inst.worktree_path))
        });
        let transcript_exists = transcript_path_opt.as_ref().map(|p| p.exists());

        // Parse transcript for metrics
        let transcript_metrics = transcript_path_opt
            .as_ref()
            .and_then(|path| transcript::parse_transcript(path));

        // Duration from transcript timestamps
        let (duration_secs, duration_human) = transcript_metrics
            .as_ref()
            .and_then(|m| m.duration_secs())
            .map(|secs| (Some(secs), Some(format_duration(secs))))
            .unwrap_or((None, None));

        // Context percent and current tool from transcript
        let context_percent = transcript_metrics.as_ref().map(|m| m.context_percent());
        let current_tool = transcript_metrics.as_ref().and_then(|m| m.current_tool.clone());

        // Get diff stats
        let (additions, deletions) = worktree_path
            .and_then(git::get_diff_stats)
            .unwrap_or((0, 0));

        total_additions += additions;
        total_deletions += deletions;

        // Get commit count and conflict status
        let commits = worktree_path.and_then(|p| {
            git::get_commit_count(p, "main")
                .or_else(|| git::get_commit_count(p, "master"))
        });
        let has_conflict = worktree_path.map(git::has_conflicts);

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
            context_percent,
            current_tool,
            additions: if additions > 0 { Some(additions) } else { None },
            deletions: if deletions > 0 { Some(deletions) } else { None },
            commits,
            has_conflict,
            idle_secs,
            active,
            tmux_alive: tmux_alive_for_output,
            session_id,
            transcript_exists,
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

// === Action execution ===

fn execute_action(action: &str, task_name: Option<String>) -> Result<()> {
    // Missing --task parameter
    let task_name = match task_name {
        Some(name) => name,
        None => {
            let response = ActionResponse {
                action: action.to_string(),
                success: false,
                error: Some("--task is required with --action".to_string()),
                task: None,
                available_actions: None,
                unavailable_actions: None,
                command: None,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&response).unwrap_or_default()
            );
            std::process::exit(1);
        }
    };

    let mut app = App::new()?;

    // Find target task in app.tasks
    let task_idx = match app.tasks.iter().position(|t| t.name == task_name) {
        Some(idx) => idx,
        None => {
            let response = ActionResponse {
                action: action.to_string(),
                success: false,
                error: Some(format!(
                    "Task '{}' not found (only running/done/merged tasks are available)",
                    task_name
                )),
                task: Some(TaskInfo {
                    name: task_name,
                    status: None,
                    status_before: None,
                    status_after: None,
                    tmux_alive: None,
                }),
                available_actions: None,
                unavailable_actions: None,
                command: None,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&response).unwrap_or_default()
            );
            std::process::exit(1);
        }
    };

    app.selected = task_idx;

    let response = match action {
        "list" => action_list(&app, &task_name),
        "done" => action_done(&mut app, &task_name),
        "merged" => action_merged(&mut app, &task_name),
        "archive" => action_archive(&mut app, &task_name),
        "enter" => action_enter(&app, &task_name),
        "tail" => action_tail(&task_name),
        _ => ActionResponse {
            action: action.to_string(),
            success: false,
            error: Some(format!("Unknown action: {}", action)),
            task: Some(TaskInfo {
                name: task_name,
                status: None,
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        },
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&response).unwrap_or_default()
    );

    if response.success {
        Ok(())
    } else {
        std::process::exit(1);
    }
}

fn action_list(app: &App, task_name: &str) -> ActionResponse {
    let task = app.selected_task().unwrap();

    let mut available = vec![];
    let mut unavailable = HashMap::new();

    // tail/enter available for Running/Done
    if matches!(task.status, TaskStatus::Running | TaskStatus::Done) {
        available.push("tail".to_string());
        available.push("enter".to_string());
    } else {
        unavailable.insert(
            "tail".to_string(),
            format!("task is {} (need running or done)", task.status.display_name()),
        );
        unavailable.insert(
            "enter".to_string(),
            format!("task is {} (need running or done)", task.status.display_name()),
        );
    }

    // done check
    if app.can_mark_done() {
        available.push("done".to_string());
    } else {
        unavailable.insert(
            "done".to_string(),
            format!("task is {} (need running)", task.status.display_name()),
        );
    }

    // merged check
    if app.can_mark_merged() {
        available.push("merged".to_string());
    } else {
        unavailable.insert(
            "merged".to_string(),
            format!("task is {} (need done)", task.status.display_name()),
        );
    }

    // archive check
    if app.can_archive() {
        available.push("archive".to_string());
    } else {
        unavailable.insert(
            "archive".to_string(),
            format!("task is {} (need merged)", task.status.display_name()),
        );
    }

    ActionResponse {
        action: "list".to_string(),
        success: true,
        error: None,
        task: Some(TaskInfo {
            name: task_name.to_string(),
            status: Some(task.status.clone()),
            status_before: None,
            status_after: None,
            tmux_alive: Some(task.tmux_alive),
        }),
        available_actions: Some(available),
        unavailable_actions: Some(unavailable),
        command: None,
    }
}

fn action_done(app: &mut App, task_name: &str) -> ActionResponse {
    let task = app.selected_task().unwrap();
    let status_before = task.status.clone();

    if !app.can_mark_done() {
        return ActionResponse {
            action: "done".to_string(),
            success: false,
            error: Some("Cannot mark as done: task is not running".to_string()),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: Some(status_before),
                status_before: None,
                status_after: None,
                tmux_alive: Some(task.tmux_alive),
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        };
    }

    if let Err(e) = app.mark_done() {
        return ActionResponse {
            action: "done".to_string(),
            success: false,
            error: Some(format!("Failed to mark as done: {}", e)),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: Some(status_before),
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        };
    }

    ActionResponse {
        action: "done".to_string(),
        success: true,
        error: None,
        task: Some(TaskInfo {
            name: task_name.to_string(),
            status: None,
            status_before: Some(status_before),
            status_after: Some(TaskStatus::Done),
            tmux_alive: None,
        }),
        available_actions: None,
        unavailable_actions: None,
        command: None,
    }
}

fn action_merged(app: &mut App, task_name: &str) -> ActionResponse {
    let task = app.selected_task().unwrap();
    let status_before = task.status.clone();

    if !app.can_mark_merged() {
        return ActionResponse {
            action: "merged".to_string(),
            success: false,
            error: Some(format!(
                "Cannot mark as merged: task is {} (need done)",
                status_before.display_name()
            )),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: Some(status_before),
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        };
    }

    if let Err(e) = app.mark_merged() {
        return ActionResponse {
            action: "merged".to_string(),
            success: false,
            error: Some(format!("Failed to mark as merged: {}", e)),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: Some(status_before),
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        };
    }

    ActionResponse {
        action: "merged".to_string(),
        success: true,
        error: None,
        task: Some(TaskInfo {
            name: task_name.to_string(),
            status: None,
            status_before: Some(status_before),
            status_after: Some(TaskStatus::Merged),
            tmux_alive: None,
        }),
        available_actions: None,
        unavailable_actions: None,
        command: None,
    }
}

fn action_archive(app: &mut App, task_name: &str) -> ActionResponse {
    let task = app.selected_task().unwrap();
    let status_before = task.status.clone();

    if !app.can_archive() {
        return ActionResponse {
            action: "archive".to_string(),
            success: false,
            error: Some(format!(
                "Cannot archive: task is {} (need merged)",
                status_before.display_name()
            )),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: Some(status_before),
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        };
    }

    if let Err(e) = app.archive() {
        return ActionResponse {
            action: "archive".to_string(),
            success: false,
            error: Some(format!("Failed to archive: {}", e)),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: Some(status_before),
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        };
    }

    ActionResponse {
        action: "archive".to_string(),
        success: true,
        error: None,
        task: Some(TaskInfo {
            name: task_name.to_string(),
            status: None,
            status_before: Some(status_before),
            status_after: Some(TaskStatus::Archived),
            tmux_alive: None,
        }),
        available_actions: None,
        unavailable_actions: None,
        command: None,
    }
}

fn action_enter(app: &App, task_name: &str) -> ActionResponse {
    let task = app.selected_task().unwrap();

    match app.enter_action() {
        Some(TuiAction::SwitchTmuxWindow { session, window })
        | Some(TuiAction::AttachTmux { session, window }) => ActionResponse {
            action: "enter".to_string(),
            success: true,
            error: None,
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: None,
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: Some(CommandInfo {
                cmd_type: "tmux_switch".to_string(),
                session: Some(session),
                window: Some(window),
                ..Default::default()
            }),
        },
        Some(TuiAction::ShowResume {
            worktree,
            session_id,
            claude_command,
        }) => ActionResponse {
            action: "enter".to_string(),
            success: true,
            error: None,
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: None,
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: Some(CommandInfo {
                cmd_type: "resume".to_string(),
                worktree: Some(worktree.clone()),
                session_id: Some(session_id.clone()),
                shell_command: Some(format!(
                    "cd {} && {} -r {}",
                    worktree, claude_command, session_id
                )),
                ..Default::default()
            }),
        },
        _ => ActionResponse {
            action: "enter".to_string(),
            success: false,
            error: Some("Cannot enter: no tmux info available".to_string()),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: Some(task.status.clone()),
                status_before: None,
                status_after: None,
                tmux_alive: Some(task.tmux_alive),
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        },
    }
}

fn action_tail(task_name: &str) -> ActionResponse {
    // Execute tail command directly - it outputs JSON
    match crate::commands::tail::execute(task_name.to_string(), 1) {
        Ok(_) => {
            // tail::execute already printed output, return empty success
            // Actually, we need to return a proper response
            // But tail already outputs JSON, so this is a bit awkward
            // Let's just indicate success without printing additional JSON
            std::process::exit(0);
        }
        Err(e) => ActionResponse {
            action: "tail".to_string(),
            success: false,
            error: Some(format!("Failed to tail: {}", e)),
            task: Some(TaskInfo {
                name: task_name.to_string(),
                status: None,
                status_before: None,
                status_after: None,
                tmux_alive: None,
            }),
            available_actions: None,
            unavailable_actions: None,
            command: None,
        },
    }
}

