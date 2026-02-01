use std::env;

use uuid::Uuid;

use crate::constants::branch_name;
use crate::error::{Result, WtError};
use crate::models::{Instance, TaskStatus, TaskStore, WtConfig};
use crate::services::{dependency, git, tmux, workspace::WorkspaceInitializer};

pub fn execute(name: String) -> Result<()> {
    let config = WtConfig::load()?;
    let mut store = TaskStore::load()?;

    // Check task exists
    let _task = store
        .get(&name)
        .ok_or_else(|| WtError::TaskNotFound(name.clone()))?;

    // Check status from StatusStore
    if store.get_status(&name) == TaskStatus::Running {
        return Err(WtError::AlreadyRunning(name.clone()));
    }

    dependency::check_dependencies_merged(&store, &name)?;

    // Generate session ID for branch naming and Claude Code tracking
    let session_id = Uuid::new_v4().to_string();
    let branch = branch_name(&name, &session_id);
    let cwd = env::current_dir().map_err(|e| WtError::Git(e.to_string()))?;
    let worktree_path = cwd
        .join(&config.worktree_dir)
        .join(&name)
        .to_string_lossy()
        .to_string();

    // Check if branch already exists (e.g., from a previous failed cleanup)
    if git::branch_exists(&branch) {
        return Err(WtError::BranchExists(branch));
    }

    git::create_worktree(&branch, &worktree_path)?;

    // Initialize workspace
    let initializer = WorkspaceInitializer::new(&worktree_path, &cwd);

    // Copy files from main project to worktree
    let copied = initializer.copy_files(&config.copy_files)?;
    for file in &copied {
        println!("  Copied: {}", file);
    }

    // Run init script if configured
    if let Some(ref script) = config.init_script {
        println!("  Running init script...");
        initializer.run_init_script(script)?;
    }

    if !tmux::session_exists(&config.tmux_session) {
        tmux::create_session(&config.tmux_session)?;
    }

    // Build agent command: claude_command + start_args
    let expanded_args = config
        .start_args
        .replace("${task}", &name)
        .replace("${branch}", &branch)
        .replace("${worktree}", &worktree_path);

    // Add --session-id to the command for session tracking
    let agent_cmd = format!(
        "{} {} --session-id {}",
        config.claude_command, expanded_args, session_id
    );

    tmux::create_window(&config.tmux_session, &name, &worktree_path, &agent_cmd)?;

    // Update status in StatusStore
    store.set_status(&name, TaskStatus::Running);
    store.set_instance(
        &name,
        Some(Instance {
            branch: branch.clone(),
            worktree_path: worktree_path.clone(),
            tmux_session: config.tmux_session.clone(),
            tmux_window: name.clone(),
            session_id: Some(session_id),
        }),
    );
    store.save_status()?;

    let relative_path = format!("{}/{}", config.worktree_dir, name);

    println!("Task '{}' started.", name);
    println!();
    println!("  Worktree: {}", relative_path);
    println!("  Branch:   {}", branch);
    println!();
    println!("进入工作区:");
    println!("  wt status          # TUI 中按 Enter 进入 tmux 窗口");
    println!("  cd {}    # 或直接进入目录", relative_path);
    Ok(())
}
