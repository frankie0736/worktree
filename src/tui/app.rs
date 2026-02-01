//! Application state for TUI.

use std::time::SystemTime;

use crate::constants::IDLE_THRESHOLD_SECS;
use crate::display::format_duration;
use crate::error::Result;
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::{git, tmux, transcript};

/// Action to perform after TUI exits or during TUI
#[derive(Debug, Clone)]
pub enum TuiAction {
    /// Just quit, no action
    Quit,
    /// Switch to tmux window (inside tmux, window exists)
    SwitchTmuxWindow { session: String, window: String },
    /// Attach to tmux session (outside tmux, window exists)
    AttachTmux { session: String, window: String },
    /// Show resume command (tmux window closed, need to copy command)
    ShowResume {
        worktree: String,
        session_id: String,
        claude_command: String,
    },
    /// Review a done task
    Review { name: String },
}

/// Task with computed metrics for display
#[derive(Debug, Clone)]
pub struct TaskDisplay {
    pub name: String,
    pub status: TaskStatus,
    pub duration: Option<String>,
    pub context_percent: u8,
    pub additions: i32,
    pub deletions: i32,
    pub active: bool,
    pub tmux_alive: bool,
    pub worktree_path: Option<String>,
    pub tmux_session: Option<String>,
    pub tmux_window: Option<String>,
    pub session_id: Option<String>,
}

/// Application state
pub struct App {
    pub tasks: Vec<TaskDisplay>,
    pub selected: usize,
    #[allow(dead_code)]
    config: Option<WtConfig>,
}

impl App {
    /// Create new app and load initial data
    pub fn new() -> Result<Self> {
        let config = WtConfig::load().ok();
        let mut app = Self {
            tasks: Vec::new(),
            selected: 0,
            config,
        };
        app.refresh()?;
        Ok(app)
    }

    /// Refresh task data from disk
    pub fn refresh(&mut self) -> Result<()> {
        let mut store = TaskStore::load()?;
        let mut tasks = Vec::new();
        let mut tasks_to_mark_done: Vec<String> = Vec::new();

        for task in store.list() {
            let status = store.get_status(task.name());

            // Only show Running and Done tasks
            if status != TaskStatus::Running && status != TaskStatus::Done {
                continue;
            }

            let instance = store.get_instance(task.name());
            let worktree_path = instance.map(|i| i.worktree_path.clone());

            // Tmux status - check first for auto-done detection
            let tmux_alive = if let Some(inst) = instance {
                tmux::window_exists(&inst.tmux_session, &inst.tmux_window)
            } else {
                false
            };

            // Auto-mark as Done if Running but tmux window is gone
            let final_status = if status == TaskStatus::Running && !tmux_alive {
                tasks_to_mark_done.push(task.name().to_string());
                TaskStatus::Done
            } else {
                status
            };

            // Parse transcript for metrics (duration, context, etc.)
            let transcript_metrics = instance.and_then(|inst| {
                inst.session_id
                    .as_ref()
                    .and_then(|sid| transcript::transcript_path(&inst.worktree_path, sid))
                    .and_then(|path| transcript::parse_transcript(&path))
            });

            // Duration from transcript timestamps
            let duration = transcript_metrics
                .as_ref()
                .and_then(|m| m.duration_secs())
                .map(format_duration);

            // Git changes
            let (additions, deletions) = worktree_path
                .as_deref()
                .and_then(git::get_diff_stats)
                .unwrap_or((0, 0));

            // Activity status
            let active = if let Some(ref path) = worktree_path {
                git::get_last_activity(path)
                    .and_then(|last| {
                        SystemTime::now()
                            .duration_since(last)
                            .ok()
                            .map(|d| d.as_secs() < IDLE_THRESHOLD_SECS)
                    })
                    .unwrap_or(false)
            } else {
                false
            };

            // Context from transcript
            let context_percent = transcript_metrics
                .as_ref()
                .map(|m| m.context_percent())
                .unwrap_or(0);

            // Get tmux and session info
            let (tmux_session, tmux_window, session_id) = instance
                .map(|i| {
                    (
                        Some(i.tmux_session.clone()),
                        Some(i.tmux_window.clone()),
                        i.session_id.clone(),
                    )
                })
                .unwrap_or((None, None, None));

            tasks.push(TaskDisplay {
                name: task.name().to_string(),
                status: final_status,
                duration,
                context_percent,
                additions,
                deletions,
                active,
                tmux_alive,
                worktree_path,
                tmux_session,
                tmux_window,
                session_id,
            });
        }

        // Auto-mark tasks as Done (Running but tmux window gone)
        if !tasks_to_mark_done.is_empty() {
            for name in &tasks_to_mark_done {
                store.set_status(name, TaskStatus::Done);
            }
            store.save_status()?;
        }

        self.tasks = tasks;

        // Adjust selection if out of bounds
        if self.selected >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected = self.tasks.len() - 1;
        }

        Ok(())
    }

    /// Get currently selected task
    pub fn selected_task(&self) -> Option<&TaskDisplay> {
        self.tasks.get(self.selected)
    }

    /// Select next task
    pub fn next(&mut self) {
        if !self.tasks.is_empty() {
            self.selected = (self.selected + 1) % self.tasks.len();
        }
    }

    /// Select previous task
    pub fn previous(&mut self) {
        if !self.tasks.is_empty() {
            self.selected = self.selected.checked_sub(1).unwrap_or(self.tasks.len() - 1);
        }
    }

    /// Check if selected task can be marked as done (Running + tmux exited)
    pub fn can_mark_done(&self) -> bool {
        self.selected_task()
            .map(|t| t.status == TaskStatus::Running && !t.tmux_alive)
            .unwrap_or(false)
    }

    /// Check if selected task can be marked as merged (Done status)
    pub fn can_mark_merged(&self) -> bool {
        self.selected_task()
            .map(|t| t.status == TaskStatus::Done)
            .unwrap_or(false)
    }

    /// Mark selected task as done
    pub fn mark_done(&mut self) -> Result<()> {
        if let Some(task) = self.selected_task() {
            if task.status == TaskStatus::Running && !task.tmux_alive {
                let name = task.name.clone();
                let mut store = TaskStore::load()?;
                store.set_status(&name, TaskStatus::Done);
                store.save_status()?;
                self.refresh()?;
            }
        }
        Ok(())
    }

    /// Mark selected task as merged
    pub fn mark_merged(&mut self) -> Result<()> {
        if let Some(task) = self.selected_task() {
            if task.status == TaskStatus::Done {
                let name = task.name.clone();
                let mut store = TaskStore::load()?;
                store.set_status(&name, TaskStatus::Merged);
                store.save_status()?;
                self.refresh()?;
            }
        }
        Ok(())
    }

    /// Check if running inside tmux
    pub fn is_in_tmux(&self) -> bool {
        std::env::var("TMUX").is_ok()
    }

    /// Get action for Enter key on selected task
    /// - Inside tmux + window exists: attach to it
    /// - Inside tmux + window closed: show resume command
    /// - Outside tmux: show tmux attach command
    pub fn enter_action(&self) -> Option<TuiAction> {
        let task = self.selected_task()?;

        // Need tmux session and window info
        let session = task.tmux_session.as_ref()?;
        let window = task.tmux_window.as_ref()?;

        let claude_command = self
            .config
            .as_ref()
            .map(|c| c.claude_command.clone())
            .unwrap_or_else(|| "claude".to_string());

        if task.tmux_alive {
            if self.is_in_tmux() {
                // Inside tmux: switch to target window
                Some(TuiAction::SwitchTmuxWindow {
                    session: session.clone(),
                    window: window.clone(),
                })
            } else {
                // Outside tmux: attach to session
                Some(TuiAction::AttachTmux {
                    session: session.clone(),
                    window: window.clone(),
                })
            }
        } else {
            // Tmux window closed, show resume command
            let worktree = task.worktree_path.as_ref()?;
            let session_id = task.session_id.as_ref()?;
            Some(TuiAction::ShowResume {
                worktree: worktree.clone(),
                session_id: session_id.clone(),
                claude_command,
            })
        }
    }

    /// Get action to review selected task
    pub fn review_action(&self) -> Option<TuiAction> {
        self.selected_task().and_then(|task| {
            if task.status == TaskStatus::Done {
                Some(TuiAction::Review {
                    name: task.name.clone(),
                })
            } else {
                None
            }
        })
    }
}
