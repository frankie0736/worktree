use std::collections::HashMap;

use crate::models::TaskStatus;
use crate::tui::{App, TuiAction};

use super::types::{ActionResponse, CommandInfo, TaskInfo};

/// Execute an action via the --action API
pub fn execute_action(action: &str, task_name: Option<String>) {
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

    let mut app = match App::new() {
        Ok(app) => app,
        Err(e) => {
            let response = ActionResponse {
                action: action.to_string(),
                success: false,
                error: Some(format!("Failed to initialize: {}", e)),
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
        "list" => handle_list_action(&app, &task_name),
        "done" => handle_done_action(&mut app, &task_name),
        "merged" => handle_merged_action(&mut app, &task_name),
        "archive" => handle_archive_action(&mut app, &task_name),
        "enter" => handle_enter_action(&app, &task_name),
        "tail" => handle_tail_action(&task_name),
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

    if !response.success {
        std::process::exit(1);
    }
}

fn handle_list_action(app: &App, task_name: &str) -> ActionResponse {
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

fn handle_done_action(app: &mut App, task_name: &str) -> ActionResponse {
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

fn handle_merged_action(app: &mut App, task_name: &str) -> ActionResponse {
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

fn handle_archive_action(app: &mut App, task_name: &str) -> ActionResponse {
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

fn handle_enter_action(app: &App, task_name: &str) -> ActionResponse {
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

fn handle_tail_action(task_name: &str) -> ActionResponse {
    // Execute tail command directly - it outputs JSON
    match crate::commands::tail::execute(task_name.to_string(), 1) {
        Ok(_) => {
            // tail::execute already printed output, exit without additional JSON
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
