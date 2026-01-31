use std::path::Path;
use std::time::SystemTime;

use crate::error::{Result, WtError};
use crate::services::command::CommandRunner;

pub fn create_worktree(branch: &str, path: &str) -> Result<()> {
    let worktree_path = Path::new(path);
    if let Some(parent) = worktree_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| WtError::Git(e.to_string()))?;
        }
    }

    CommandRunner::git().run(&["worktree", "add", "-b", branch, path])
}

pub fn remove_worktree(path: &str) -> Result<()> {
    CommandRunner::git().run(&["worktree", "remove", "--force", path])
}

pub fn delete_branch(branch: &str) -> Result<()> {
    CommandRunner::git().run(&["branch", "-D", branch])
}

pub fn branch_exists(branch: &str) -> bool {
    CommandRunner::git().success(&["show-ref", "--verify", "--quiet", &format!("refs/heads/{}", branch)])
}

/// Get diff stats (additions, deletions) for a worktree compared to its base branch.
pub fn get_diff_stats(worktree_path: &str) -> Option<(i32, i32)> {
    let output = CommandRunner::new("git")
        .current_dir(worktree_path)
        .output(&["diff", "--shortstat", "HEAD"]);

    if let Ok(stdout) = output {
        parse_diff_stats(&stdout)
    } else {
        None
    }
}

/// Parse git diff --shortstat output like "3 files changed, 10 insertions(+), 5 deletions(-)"
fn parse_diff_stats(output: &str) -> Option<(i32, i32)> {
    let output = output.trim();
    if output.is_empty() {
        return Some((0, 0));
    }

    let mut insertions = 0;
    let mut deletions = 0;

    for part in output.split(',') {
        let part = part.trim();
        if part.contains("insertion") {
            if let Some(num) = part.split_whitespace().next() {
                insertions = num.parse().unwrap_or(0);
            }
        } else if part.contains("deletion") {
            if let Some(num) = part.split_whitespace().next() {
                deletions = num.parse().unwrap_or(0);
            }
        }
    }

    Some((insertions, deletions))
}

/// Get the number of commits ahead of the base branch.
pub fn get_commit_count(worktree_path: &str, base_branch: &str) -> Option<i32> {
    let range = format!("{}..HEAD", base_branch);
    let output = CommandRunner::new("git")
        .current_dir(worktree_path)
        .output(&["rev-list", "--count", &range]);

    if let Ok(stdout) = output {
        stdout.trim().parse().ok()
    } else {
        None
    }
}

/// Get the last modification time of any file in the worktree.
pub fn get_last_activity(worktree_path: &str) -> Option<SystemTime> {
    let path = Path::new(worktree_path);
    if !path.exists() {
        return None;
    }

    path.metadata().ok()?.modified().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_diff_stats_empty() {
        assert_eq!(parse_diff_stats(""), Some((0, 0)));
        assert_eq!(parse_diff_stats("  "), Some((0, 0)));
    }

    #[test]
    fn test_parse_diff_stats_insertions_only() {
        assert_eq!(
            parse_diff_stats("1 file changed, 10 insertions(+)"),
            Some((10, 0))
        );
    }

    #[test]
    fn test_parse_diff_stats_deletions_only() {
        assert_eq!(
            parse_diff_stats("1 file changed, 5 deletions(-)"),
            Some((0, 5))
        );
    }

    #[test]
    fn test_parse_diff_stats_both() {
        assert_eq!(
            parse_diff_stats("3 files changed, 10 insertions(+), 5 deletions(-)"),
            Some((10, 5))
        );
    }

    #[test]
    fn test_parse_diff_stats_singular() {
        assert_eq!(
            parse_diff_stats("1 file changed, 1 insertion(+), 1 deletion(-)"),
            Some((1, 1))
        );
    }
}
