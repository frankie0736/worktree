use super::*;

#[test]
fn test_merged_nonexistent() {
    let dir = setup_test_repo();
    let (ok, _, stderr) = run_wt(dir.path(), &["merged", "nonexistent"]);

    assert!(!ok);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_merged_already_merged() {
    let dir = setup_repo_with_tasks(&[("task", &[], "merged")]);

    let (ok, stdout, _) = run_wt(dir.path(), &["merged", "task"]);

    // Should warn but succeed (idempotent)
    assert!(ok);
    assert!(stdout.contains("Warning") || stdout.contains("merged"));
}

#[test]
fn test_merged_pending_task() {
    let dir = setup_repo_with_tasks(&[("task", &[], "pending")]);

    let (ok, stdout, _) = run_wt(dir.path(), &["merged", "task"]);

    // Should warn but succeed (allow force merge)
    assert!(ok);
    assert!(stdout.contains("Warning") || stdout.contains("pending"));
}
