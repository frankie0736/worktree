//! CLI tests for wt status command

use crate::common::*;
use serde_json::json;

#[test]
fn test_status_no_tasks() {
    let dir = setup_test_repo();

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status"]);

    assert!(ok);
    assert!(stdout.contains("No running or done tasks"));
}

#[test]
fn test_status_with_running_task() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "running")]);

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status"]);

    assert!(ok);
    assert!(stdout.contains("task1"));
    assert!(stdout.contains("running"));
}

#[test]
fn test_status_with_done_task() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "done")]);

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status"]);

    assert!(ok);
    assert!(stdout.contains("task1"));
    assert!(stdout.contains("done"));
}

#[test]
fn test_status_ignores_pending_tasks() {
    let dir = setup_repo_with_tasks(&[
        ("task1", &[], "pending"),
        ("task2", &[], "running"),
    ]);

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status"]);

    assert!(ok);
    assert!(!stdout.contains("task1"));
    assert!(stdout.contains("task2"));
}

#[test]
fn test_status_ignores_merged_tasks() {
    let dir = setup_repo_with_tasks(&[
        ("task1", &[], "merged"),
        ("task2", &[], "running"),
    ]);

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status"]);

    assert!(ok);
    assert!(!stdout.contains("task1"));
    assert!(stdout.contains("task2"));
}

#[test]
fn test_status_json_output() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "running")]);

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status", "--json"]);

    assert!(ok);
    // Should be valid JSON
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&stdout);
    assert!(parsed.is_ok(), "Output should be valid JSON: {}", stdout);

    let json = parsed.unwrap();
    assert!(json.get("tasks").is_some());
    assert!(json.get("summary").is_some());
}

#[test]
fn test_status_json_structure() {
    let dir = setup_repo_with_tasks(&[
        ("task1", &[], "running"),
        ("task2", &[], "done"),
    ]);

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status", "--json"]);

    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    // Check tasks array
    let tasks = json.get("tasks").unwrap().as_array().unwrap();
    assert_eq!(tasks.len(), 2);

    // Check summary
    let summary = json.get("summary").unwrap();
    assert_eq!(summary.get("running").unwrap().as_i64().unwrap(), 1);
    assert_eq!(summary.get("done").unwrap().as_i64().unwrap(), 1);
}

#[test]
fn test_status_summary_line() {
    let dir = setup_repo_with_tasks(&[
        ("task1", &[], "running"),
        ("task2", &[], "done"),
    ]);

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status"]);

    assert!(ok);
    assert!(stdout.contains("Summary:"));
    assert!(stdout.contains("1 running"));
    assert!(stdout.contains("1 done"));
}

#[test]
fn test_status_shows_warning_when_tmux_window_closed() {
    let dir = setup_test_repo();

    // Create task file
    create_task_file(dir.path(), "task1", &[]);

    // Set running status with instance pointing to non-existent tmux window
    set_task_status_with_instance(
        dir.path(),
        "task1",
        "running",
        Some(json!({
            "branch": "wt/task1",
            "worktree_path": "/tmp/nonexistent",
            "tmux_session": "nonexistent-session-12345",
            "tmux_window": "task1"
        })),
    );

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status"]);

    assert!(ok);
    assert!(stdout.contains("task1"));
    assert!(stdout.contains("tmux window closed"), "Should warn about closed tmux window: {}", stdout);
}

#[test]
fn test_status_json_includes_tmux_alive_field() {
    let dir = setup_test_repo();

    // Create task file
    create_task_file(dir.path(), "task1", &[]);

    // Set running status with instance pointing to non-existent tmux window
    set_task_status_with_instance(
        dir.path(),
        "task1",
        "running",
        Some(json!({
            "branch": "wt/task1",
            "worktree_path": "/tmp/nonexistent",
            "tmux_session": "nonexistent-session-12345",
            "tmux_window": "task1"
        })),
    );

    let (ok, stdout, _stderr) = run_wt(dir.path(), &["status", "--json"]);

    assert!(ok);
    let json: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let tasks = json.get("tasks").unwrap().as_array().unwrap();
    let task = &tasks[0];

    assert_eq!(task.get("tmux_alive").unwrap().as_bool().unwrap(), false);
}
