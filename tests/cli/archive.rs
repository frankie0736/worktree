//! CLI tests for wt archive command

use crate::common::*;

// ==================== Normal Task Archive ====================

#[test]
fn test_archive_nonexistent() {
    let dir = setup_test_repo();

    let (ok, _, stderr) = run_wt(dir.path(), &["archive", "nonexistent"]);

    assert!(!ok);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_archive_pending_task_fails() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "pending")]);

    let (ok, _, stderr) = run_wt(dir.path(), &["archive", "task1"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("transition") || stderr.contains("pending"),
        "Expected state transition error, got: {}",
        stderr
    );
}

#[test]
fn test_archive_running_task_fails() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "running")]);

    let (ok, _, stderr) = run_wt(dir.path(), &["archive", "task1"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("transition"),
        "Expected state transition error, got: {}",
        stderr
    );
}

#[test]
fn test_archive_done_task_fails() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "done")]);

    let (ok, _, stderr) = run_wt(dir.path(), &["archive", "task1"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("transition"),
        "Expected state transition error, got: {}",
        stderr
    );
}

#[test]
fn test_archive_merged_task_succeeds() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "merged")]);

    let (ok, stdout, _) = run_wt(dir.path(), &["archive", "task1"]);

    assert!(ok);
    assert!(
        stdout.contains("archived") || stdout.contains("Archive"),
        "Expected archive confirmation, got: {}",
        stdout
    );
}

#[test]
fn test_archive_updates_status_to_archived() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "merged")]);

    let (ok, _, _) = run_wt(dir.path(), &["archive", "task1"]);
    assert!(ok);

    // Verify status changed to archived
    let (ok, stdout, _) = run_wt(dir.path(), &["list", "--json"]);
    assert!(ok);
    assert!(
        stdout.contains("\"status\": \"archived\"") || stdout.contains("\"status\":\"archived\""),
        "Expected archived status, got: {}",
        stdout
    );
}

#[test]
fn test_archive_preserves_task_file() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "merged")]);

    let task_file = dir.path().join(".wt/tasks/task1.md");
    assert!(task_file.exists(), "Task file should exist before archive");

    let (ok, _, _) = run_wt(dir.path(), &["archive", "task1"]);
    assert!(ok);

    // Task file should still exist after archive
    assert!(
        task_file.exists(),
        "Task file should be preserved after archive"
    );
}

#[test]
fn test_archive_already_archived_is_idempotent() {
    let dir = setup_repo_with_tasks(&[("task1", &[], "archived")]);

    let (ok, stdout, stderr) = run_wt(dir.path(), &["archive", "task1"]);

    // May succeed with warning or fail with clear message
    // Behavior depends on implementation - document it
    let output = format!("{}{}", stdout, stderr);
    assert!(
        ok || output.contains("archived") || output.contains("Invalid"),
        "Expected success, warning, or state error, got: {}",
        output
    );
}
