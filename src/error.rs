use thiserror::Error;

#[derive(Error, Debug)]
pub enum WtError {
    #[error("Config file .wt.yaml not found in current directory")]
    ConfigNotFound,

    #[error("Failed to read config: {0}")]
    ConfigRead(String),

    #[error("Failed to parse config: {0}")]
    ConfigParse(#[from] serde_yaml::Error),

    #[error("Task '{0}' not found")]
    TaskNotFound(String),

    #[error("Task '{0}' already exists")]
    TaskExists(String),

    #[error("Dependency '{0}' not found")]
    DependencyNotFound(String),

    #[error("Cannot start task '{task}': dependency '{dep}' is not merged")]
    DependencyNotMerged { task: String, dep: String },

    #[error("Git command failed: {0}")]
    Git(String),

    #[error("Tmux command failed: {0}")]
    Tmux(String),

    #[error("Task '{0}' is already running")]
    AlreadyRunning(String),

    #[error("Branch '{0}' already exists.\nHint: Run `git branch -D {0}` to delete it, then retry.")]
    BranchExists(String),

    #[error("Invalid task file: {0}")]
    InvalidTaskFile(String),

    #[error("Invalid JSON: {0}")]
    InvalidJson(String),

    #[error("IO error during {operation} on '{path}': {message}")]
    Io {
        operation: String,
        path: String,
        message: String,
    },

    #[error("Failed to execute script '{script}': {message}")]
    Script { script: String, message: String },

    #[error("Script '{script}' failed with exit code: {exit_code:?}")]
    ScriptFailed {
        script: String,
        exit_code: Option<i32>,
    },

    #[error("Invalid state transition: cannot change task from {from} to {to}")]
    InvalidStateTransition { from: String, to: String },

    #[error("Cannot reset '{task}': task '{dependent}' depends on it and is {status}")]
    HasDependents {
        task: String,
        dependent: String,
        status: String,
    },

    #[error("Cannot review task '{0}': task is still running")]
    CannotReviewRunning(String),

    #[error("Cannot review task '{0}': worktree no longer exists")]
    WorktreeNotFound(String),

    #[error("Cannot review task '{0}': session transcript not found")]
    TranscriptNotFound(String),

    #[error("Cannot review task '{0}': task has no session ID (started with older wt version)")]
    NoSessionId(String),

    #[error("Task '{0}' is not in Done state")]
    TaskNotDone(String),
}

pub type Result<T> = std::result::Result<T, WtError>;
