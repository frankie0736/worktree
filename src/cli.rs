use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wt")]
#[command(about = "Worktree Task Manager - manage multi-agent parallel development tasks")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize wt in current directory
    Init,

    /// Create a new task from JSON
    Create {
        /// JSON input: {"name": "...", "depends": [...], "description": "..."}
        #[arg(long)]
        json: String,
    },

    /// Validate all task files
    Validate {
        /// Specific task name to validate (optional)
        name: Option<String>,
    },

    /// List all tasks (grouped by status)
    List {
        /// Show tree view instead of grouped view
        #[arg(long)]
        tree: bool,

        /// Output as JSON for programmatic use
        #[arg(long)]
        json: bool,
    },

    /// Start a task (creates worktree and tmux window)
    Start {
        /// Task name to start
        name: String,
    },

    /// Mark a task as done (ready for review)
    Done {
        /// Task name to mark as done
        name: String,
    },

    /// Mark a task as merged (cleanup and unblock dependents)
    Merged {
        /// Task name to mark as merged
        name: String,
    },

    /// Cleanup worktrees and tmux windows
    Cleanup {
        /// Clean all tasks (not just merged ones)
        #[arg(long)]
        all: bool,
    },

    /// Show tasks that are ready to start (all dependencies merged)
    Next {
        /// Output as JSON for programmatic use
        #[arg(long)]
        json: bool,
    },

    /// Reset a task to pending state (cleanup resources)
    Reset {
        /// Task name to reset
        name: String,
    },

    /// Show status of running/done tasks (TUI by default, --json for programmatic use)
    Status {
        /// Output as JSON for programmatic use (non-interactive)
        #[arg(long)]
        json: bool,
    },

    /// View last assistant messages from task transcript (JSON output)
    Tail {
        /// Task name
        name: String,

        /// Number of turns to show (default: 1)
        #[arg(short = 'n', default_value = "1")]
        count: usize,
    },

    /// Generate filtered logs for all tasks
    Logs,
}
