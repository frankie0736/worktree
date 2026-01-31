use super::*;

#[test]
fn test_enter_no_session() {
    let dir = setup_test_repo();

    let (ok, _, stderr) = run_wt(dir.path(), &["enter"]);

    assert!(!ok);
    assert!(stderr.contains("does not exist") || stderr.contains("Start a task first"));
}

#[test]
fn test_enter_task_not_found() {
    let dir = setup_test_repo();

    let (ok, _, stderr) = run_wt(dir.path(), &["enter", "nonexistent"]);

    // Either session doesn't exist or task not found
    assert!(!ok);
    assert!(stderr.contains("not found") || stderr.contains("does not exist"));
}

#[test]
fn test_enter_task_not_running() {
    let dir = setup_repo_with_tasks(&[("auth", &[], "pending")]);

    let (ok, _, stderr) = run_wt(dir.path(), &["enter", "auth"]);

    // Session doesn't exist (no task started) or task not running
    assert!(!ok);
    assert!(
        stderr.contains("not running")
            || stderr.contains("does not exist")
            || stderr.contains("Start")
    );
}

#[test]
fn test_enter_help() {
    let dir = setup_test_repo();

    let (ok, stdout, _) = run_wt(dir.path(), &["enter", "--help"]);

    assert!(ok);
    assert!(stdout.contains("Enter tmux session"));
    assert!(stdout.contains("view/interact"));
}
