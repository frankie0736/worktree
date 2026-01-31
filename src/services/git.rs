use std::path::Path;

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
