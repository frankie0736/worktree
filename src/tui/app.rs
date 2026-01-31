//! Application state for TUI.

use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

use chrono::Utc;

use crate::constants::{log_path, IDLE_THRESHOLD_SECS};
use crate::display::format_duration;
use crate::error::{Result, WtError};
use crate::models::{TaskStatus, TaskStore, WtConfig};
use crate::services::{git, logs, tmux};

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
    pub tmux_session: Option<String>,
    pub tmux_window: Option<String>,
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
        let store = TaskStore::load()?;
        let mut tasks = Vec::new();

        for task in store.list() {
            let status = store.get_status(task.name());

            // Only show Running and Done tasks
            if status != TaskStatus::Running && status != TaskStatus::Done {
                continue;
            }

            let instance = store.get_instance(task.name());
            let worktree_path = instance.map(|i| i.worktree_path.as_str());
            let started_at = instance.and_then(|i| i.started_at);

            // Duration
            let duration = started_at.map(|start| {
                let now = Utc::now();
                let secs = now.signed_duration_since(start).num_seconds();
                format_duration(secs)
            });

            // Git changes
            let (additions, deletions) = worktree_path
                .and_then(git::get_diff_stats)
                .unwrap_or((0, 0));

            // Activity status
            let active = if let Some(path) = worktree_path {
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

            // Tmux status
            let (tmux_session, tmux_window, tmux_alive) = if let Some(inst) = instance {
                let alive = tmux::window_exists(&inst.tmux_session, &inst.tmux_window);
                (
                    Some(inst.tmux_session.clone()),
                    Some(inst.tmux_window.clone()),
                    alive,
                )
            } else {
                (None, None, false)
            };

            // Context from logs
            let context_percent = {
                let log_file = log_path(task.name());
                let path = Path::new(&log_file);
                logs::parse_log_file(path)
                    .map(|m| m.context_percent())
                    .unwrap_or(0)
            };

            tasks.push(TaskDisplay {
                name: task.name().to_string(),
                status,
                duration,
                context_percent,
                additions,
                deletions,
                active,
                tmux_alive,
                tmux_session,
                tmux_window,
            });
        }

        self.tasks = tasks;

        // Adjust selection if out of bounds
        if self.selected >= self.tasks.len() && !self.tasks.is_empty() {
            self.selected = self.tasks.len() - 1;
        }

        Ok(())
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

    /// Enter tmux window for selected task
    pub fn enter_selected(&self) -> Result<()> {
        if let Some(task) = self.tasks.get(self.selected) {
            if let (Some(session), Some(window)) = (&task.tmux_session, &task.tmux_window) {
                let target = format!("{}:{}", session, window);

                // Attach to tmux session and select window
                Command::new("tmux")
                    .args(["select-window", "-t", &target])
                    .status()
                    .ok();

                Command::new("tmux")
                    .args(["attach-session", "-t", session])
                    .status()
                    .map_err(|e| WtError::Tmux(format!("Failed to attach: {}", e)))?;

                Ok(())
            } else {
                Err(WtError::TaskNotRunning(task.name.clone()))
            }
        } else {
            Ok(())
        }
    }
}
