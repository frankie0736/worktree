use super::*;

#[test]
fn test_start_nonexistent() {
    let dir = setup_test_repo();
    let (ok, _, stderr) = run_wt(dir.path(), &["start", "nonexistent"]);

    assert!(!ok);
    assert!(stderr.contains("not found"));
}

#[test]
fn test_start_unmerged_dependency() {
    let dir = setup_test_repo();

    run_wt(dir.path(), &["create", "--json", r#"{"name": "a", "depends": [], "description": "A"}"#]);
    run_wt(dir.path(), &["create", "--json", r#"{"name": "b", "depends": ["a"], "description": "B"}"#]);

    let (ok, _, stderr) = run_wt(dir.path(), &["start", "b"]);

    assert!(!ok);
    assert!(stderr.contains("not merged") || stderr.contains("dependency"));
}

#[test]
fn test_start_already_running() {
    let dir = setup_repo_with_tasks(&[("task", &[], "running")]);

    let (ok, _, stderr) = run_wt(dir.path(), &["start", "task"]);

    assert!(!ok);
    assert!(stderr.contains("already") || stderr.contains("running"));
}
