//! Centralized constants for path and naming conventions.

/// Task markdown files directory
pub const TASKS_DIR: &str = ".wt/tasks";

/// Configuration file name
pub const CONFIG_FILE: &str = ".wt.yaml";

/// Default worktree directory
pub const DEFAULT_WORKTREE_DIR: &str = ".wt-worktrees";

/// Default tmux session name
pub const DEFAULT_TMUX_SESSION: &str = "wt";

/// Branch name prefix for worktree tasks
pub const BRANCH_PREFIX: &str = "wt/";

/// Status file for runtime state
pub const STATUS_FILE: &str = ".wt/status.json";

/// Logs directory for agent output
pub const LOGS_DIR: &str = ".wt/logs";

/// Idle threshold in seconds (for status command)
pub const IDLE_THRESHOLD_SECS: u64 = 120;

/// Watch interval in seconds (for status --watch)
pub const WATCH_INTERVAL_SECS: u64 = 2;

/// Generate log file path from task name
pub fn log_path(task_name: &str) -> String {
    format!("{}/{}.jsonl", LOGS_DIR, task_name)
}

/// Generate branch name from task name
pub fn branch_name(task_name: &str) -> String {
    format!("{}{}", BRANCH_PREFIX, task_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_branch_name() {
        assert_eq!(branch_name("auth"), "wt/auth");
        assert_eq!(branch_name("feature-x"), "wt/feature-x");
    }
}
