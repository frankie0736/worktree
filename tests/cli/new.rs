//! CLI tests for wt new command
//!
//! Note: wt new uses positional argument for name, e.g., `wt new my-scratch`

use crate::common::*;

// ==================== Name Generation ====================

#[test]
fn test_new_auto_generates_name_s1_s2() {
    let dir = setup_test_repo();

    // wt new without name should fail in test environment (no tmux)
    // but we can check the error message shows a generated name
    let (_ok, stdout, stderr) = run_wt(dir.path(), &["new"]);

    // It will fail because tmux is not available in test, but should show the generated name
    // or succeed in creating status entry before tmux failure
    let output = format!("{}{}", stdout, stderr);

    // The command should attempt to create a scratch env with auto-generated name (s1, s2, ...)
    // Either succeeds partially or fails on tmux - both indicate name was generated
    assert!(
        output.contains(": s1") || output.contains(": s2") || output.contains("'s1'") || output.contains("'s2'") || output.contains("Created scratch"),
        "Expected auto-generated name pattern 's1/s2' or creation message, got: {}",
        output
    );
}

// ==================== Explicit Name ====================

#[test]
fn test_new_with_explicit_name_validates() {
    let dir = setup_test_repo();

    // Invalid name (starts with dash) should fail validation
    // Use positional argument, not --name
    let (ok, _, stderr) = run_wt(dir.path(), &["new", "--", "-invalid"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("branch") || stderr.contains("cannot start with"),
        "Expected validation error, got: {}",
        stderr
    );
}

#[test]
fn test_new_with_valid_explicit_name() {
    let dir = setup_test_repo();

    // Valid name - will fail on git/tmux but should pass validation
    let (_ok, stdout, stderr) = run_wt(dir.path(), &["new", "my-scratch"]);

    let output = format!("{}{}", stdout, stderr);

    // Should either succeed or fail on git/tmux (not validation)
    // If it fails on validation, that's a bug
    assert!(
        !output.contains("Invalid task name"),
        "Name 'my-scratch' should be valid"
    );
}

// ==================== Name Conflicts ====================

#[test]
fn test_new_name_conflict_with_task_file() {
    let dir = setup_test_repo();

    // Create existing task file
    create_task_file(dir.path(), "existing", &[]);

    // Try to create scratch with same name (positional arg)
    let (ok, _, stderr) = run_wt(dir.path(), &["new", "existing"]);

    assert!(!ok);
    assert!(
        stderr.contains("already exists") || stderr.contains("TaskExists"),
        "Expected conflict error, got: {}",
        stderr
    );
}

#[test]
fn test_new_name_conflict_with_status_entry() {
    let dir = setup_test_repo();

    // Create status entry without task file (scratch scenario)
    set_scratch_status(dir.path(), "scratch-env", "running");

    // Try to create another scratch with same name
    let (ok, _, stderr) = run_wt(dir.path(), &["new", "scratch-env"]);

    assert!(!ok);
    assert!(
        stderr.contains("already exists") || stderr.contains("status.json"),
        "Expected conflict error, got: {}",
        stderr
    );
}

// ==================== Scratch Flag ====================

#[test]
fn test_new_does_not_create_task_file() {
    let dir = setup_test_repo();

    // Run new command (may fail on tmux but should not create task file)
    let _ = run_wt(dir.path(), &["new", "no-file-scratch"]);

    // Verify no task file was created
    let task_file = dir.path().join(".wt/tasks/no-file-scratch.md");
    assert!(
        !task_file.exists(),
        "wt new should not create task file"
    );
}

// ==================== Validation Rules ====================

#[test]
fn test_new_rejects_names_starting_with_dash() {
    let dir = setup_test_repo();

    // Use -- to pass argument starting with dash
    let (ok, _, stderr) = run_wt(dir.path(), &["new", "--", "-badname"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("branch") || stderr.contains("cannot start with"),
        "Expected validation error, got: {}",
        stderr
    );
}

#[test]
fn test_new_rejects_names_with_spaces() {
    let dir = setup_test_repo();

    let (ok, _, stderr) = run_wt(dir.path(), &["new", "bad name"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("space") || stderr.contains("whitespace"),
        "Expected validation error, got: {}",
        stderr
    );
}

#[test]
fn test_new_rejects_names_ending_with_dot() {
    let dir = setup_test_repo();

    let (ok, _, stderr) = run_wt(dir.path(), &["new", "badname."]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("branch") || stderr.contains("cannot end with"),
        "Expected validation error, got: {}",
        stderr
    );
}

#[test]
fn test_new_rejects_names_ending_with_lock() {
    let dir = setup_test_repo();

    let (ok, _, stderr) = run_wt(dir.path(), &["new", "badname.lock"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("branch") || stderr.contains(".lock"),
        "Expected validation error, got: {}",
        stderr
    );
}

#[test]
fn test_new_rejects_names_with_double_dots() {
    let dir = setup_test_repo();

    let (ok, _, stderr) = run_wt(dir.path(), &["new", "bad..name"]);

    assert!(!ok);
    assert!(
        stderr.contains("Invalid") || stderr.contains("branch") || stderr.contains(".."),
        "Expected validation error, got: {}",
        stderr
    );
}
