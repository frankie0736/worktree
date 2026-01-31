use std::env;
use std::fs;
use std::path::Path;

use crate::constants::{CONFIG_FILE, TASKS_DIR};
use crate::error::{Result, WtError};

const GITIGNORE_MARKER: &str = "# wt - Worktree Task Manager";

const GITIGNORE_ENTRIES: &str = r#"# wt - Worktree Task Manager
# https://github.com/anthropics/wt
.wt-worktrees/    # git worktree directories (auto-generated)
.wt/status.json   # runtime task status (auto-generated)
"#;

fn get_project_name() -> String {
    env::current_dir()
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .unwrap_or_else(|| "wt".to_string())
}

fn generate_config(project_name: &str) -> String {
    format!(
        r#"# wt configuration
# See: https://github.com/anthropics/wt

# Agent command (uses @file syntax to read prompt)
agent_command: claude --verbose --output-format=stream-json --input-format=stream-json -p

# Tmux session name (unique per project to avoid conflicts)
tmux_session: {}

# Directory for git worktrees
worktree_dir: .wt-worktrees

# Files to copy to each worktree (optional)
copy_files:
  - .env

# Script to run after creating worktree (optional)
# init_script: npm install
"#,
        project_name
    )
}

fn update_gitignore() -> Result<bool> {
    let gitignore_path = Path::new(".gitignore");

    if gitignore_path.exists() {
        let content = fs::read_to_string(gitignore_path).map_err(|e| WtError::Io {
            operation: "read".to_string(),
            path: ".gitignore".to_string(),
            message: e.to_string(),
        })?;

        // Check if already has wt entries
        if content.contains(GITIGNORE_MARKER) {
            return Ok(false);
        }

        // Append to existing .gitignore
        let new_content = if content.ends_with('\n') {
            format!("{}\n{}", content, GITIGNORE_ENTRIES)
        } else {
            format!("{}\n\n{}", content, GITIGNORE_ENTRIES)
        };

        fs::write(gitignore_path, new_content).map_err(|e| WtError::Io {
            operation: "write".to_string(),
            path: ".gitignore".to_string(),
            message: e.to_string(),
        })?;
    } else {
        // Create new .gitignore
        fs::write(gitignore_path, GITIGNORE_ENTRIES).map_err(|e| WtError::Io {
            operation: "create".to_string(),
            path: ".gitignore".to_string(),
            message: e.to_string(),
        })?;
    }

    Ok(true)
}

pub fn execute() -> Result<()> {
    let config_path = Path::new(CONFIG_FILE);
    let tasks_dir = Path::new(TASKS_DIR);

    // Check if already initialized
    if config_path.exists() {
        return Err(WtError::Io {
            operation: "init".to_string(),
            path: CONFIG_FILE.to_string(),
            message: "already exists. Remove it first if you want to reinitialize.".to_string(),
        });
    }

    let project_name = get_project_name();

    // Create .wt.yaml
    let config_content = generate_config(&project_name);
    fs::write(config_path, &config_content).map_err(|e| WtError::Io {
        operation: "create".to_string(),
        path: CONFIG_FILE.to_string(),
        message: e.to_string(),
    })?;
    println!("Created {}", CONFIG_FILE);

    // Create .wt/tasks/ directory
    if !tasks_dir.exists() {
        fs::create_dir_all(tasks_dir).map_err(|e| WtError::Io {
            operation: "create".to_string(),
            path: TASKS_DIR.to_string(),
            message: e.to_string(),
        })?;
        println!("Created {}/", TASKS_DIR);
    }

    // Update .gitignore
    if update_gitignore()? {
        println!("Updated .gitignore");
    } else {
        println!(".gitignore already has wt entries");
    }

    // Summary
    println!();
    println!("Initialized wt for project '{}'", project_name);
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to customize settings", CONFIG_FILE);
    println!("  2. Create tasks: wt create --json '{{\"name\": \"...\", \"description\": \"...\"}}'");
    println!("  3. Start working: wt start <task>");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_config_contains_project_name() {
        let config = generate_config("my-project");
        assert!(config.contains("tmux_session: my-project"));
    }

    #[test]
    fn test_generate_config_has_required_fields() {
        let config = generate_config("test");
        assert!(config.contains("agent_command:"));
        assert!(config.contains("tmux_session:"));
        assert!(config.contains("worktree_dir:"));
        assert!(config.contains("copy_files:"));
        assert!(config.contains(".env"));
    }

    #[test]
    fn test_gitignore_entries_has_marker() {
        assert!(GITIGNORE_ENTRIES.contains(GITIGNORE_MARKER));
    }

    #[test]
    fn test_gitignore_entries_has_worktree_dir() {
        assert!(GITIGNORE_ENTRIES.contains(".wt-worktrees/"));
    }

    #[test]
    fn test_gitignore_entries_has_status_file() {
        assert!(GITIGNORE_ENTRIES.contains(".wt/status.json"));
    }
}
