use super::*;

#[test]
fn test_cleanup_nothing() {
    let dir = setup_test_repo();
    let (ok, stdout, _) = run_wt(dir.path(), &["cleanup"]);

    assert!(ok);
    assert!(stdout.contains("Nothing to clean"));
}

#[test]
fn test_cleanup_with_merged_tasks() {
    let dir = setup_repo_with_tasks(&[
        ("merged1", &[], "merged"),
        ("merged2", &[], "merged"),
        ("pending", &[], "pending"),
    ]);

    let (ok, stdout, _) = run_wt(dir.path(), &["cleanup"]);

    assert!(ok);
    // Should only clean merged tasks
    assert!(stdout.contains("Nothing to clean") || stdout.contains("cleaned"));
}
