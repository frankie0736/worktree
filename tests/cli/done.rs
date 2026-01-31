use super::*;

#[test]
fn test_done_nonexistent() {
    let dir = setup_test_repo();
    let (ok, _, stderr) = run_wt(dir.path(), &["done", "nonexistent"]);

    assert!(!ok);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_done_pending_task() {
    let dir = setup_repo_with_tasks(&[("task", &[], "pending")]);

    let (ok, _, stderr) = run_wt(dir.path(), &["done", "task"]);

    assert!(!ok);
    assert!(stderr.contains("no running") || stderr.contains("instance") || stderr.contains("Invalid state"));
}

#[test]
fn test_done_already_done() {
    let dir = setup_repo_with_tasks(&[("task", &[], "done")]);

    let (ok, _, stderr) = run_wt(dir.path(), &["done", "task"]);

    assert!(!ok);
    assert!(stderr.contains("no running") || stderr.contains("instance") || stderr.contains("Invalid state"));
}
