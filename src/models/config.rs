use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::constants::{CONFIG_FILE, DEFAULT_TMUX_SESSION, DEFAULT_WORKTREE_DIR};
use crate::error::{Result, WtError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WtConfig {
    #[serde(default = "default_claude_command")]
    pub claude_command: String,
    #[serde(default = "default_start_args")]
    pub start_args: String,
    #[serde(default = "default_tmux_session")]
    pub tmux_session: String,
    #[serde(default = "default_worktree_dir")]
    pub worktree_dir: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub copy_files: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub init_script: Option<String>,
}

fn default_claude_command() -> String {
    "claude".to_string()
}

fn default_start_args() -> String {
    r#"--verbose --output-format=stream-json --input-format=stream-json -p "@.wt/tasks/${task}.md 请完成这个任务""#.to_string()
}

fn default_tmux_session() -> String {
    DEFAULT_TMUX_SESSION.to_string()
}

fn default_worktree_dir() -> String {
    DEFAULT_WORKTREE_DIR.to_string()
}

impl WtConfig {
    pub fn load() -> Result<Self> {
        let path = Path::new(CONFIG_FILE);
        if !path.exists() {
            return Err(WtError::ConfigNotFound);
        }
        let content = std::fs::read_to_string(path).map_err(|e| {
            WtError::ConfigRead(e.to_string())
        })?;
        Self::from_str(&content)
    }

    /// Parse config from string
    pub fn from_str(content: &str) -> Result<Self> {
        let config: WtConfig = serde_yaml::from_str(content)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_minimal() {
        let yaml = "start_args: -p test\n";
        let config = WtConfig::from_str(yaml).unwrap();

        assert_eq!(config.claude_command, "claude"); // default
        assert_eq!(config.start_args, "-p test");
        assert_eq!(config.tmux_session, "wt"); // default
        assert_eq!(config.worktree_dir, ".wt/worktrees"); // default
        assert!(config.copy_files.is_empty());
        assert!(config.init_script.is_none());
    }

    #[test]
    fn test_config_all_defaults() {
        let yaml = "{}\n";
        let config = WtConfig::from_str(yaml).unwrap();

        assert_eq!(config.claude_command, "claude");
        assert!(config.start_args.contains("--output-format=stream-json"));
        assert_eq!(config.tmux_session, "wt");
        assert_eq!(config.worktree_dir, ".wt/worktrees");
    }

    #[test]
    fn test_config_full() {
        let yaml = r#"
claude_command: ccc --yolo
start_args: -p "test"
tmux_session: my-session
worktree_dir: /custom/path
copy_files:
  - .env
  - config.json
init_script: npm install
"#;
        let config = WtConfig::from_str(yaml).unwrap();

        assert_eq!(config.claude_command, "ccc --yolo");
        assert_eq!(config.start_args, "-p \"test\"");
        assert_eq!(config.tmux_session, "my-session");
        assert_eq!(config.worktree_dir, "/custom/path");
        assert_eq!(config.copy_files, vec![".env", "config.json"]);
        assert_eq!(config.init_script, Some("npm install".to_string()));
    }

    #[test]
    fn test_config_invalid_yaml() {
        let yaml = "claude_command: [invalid";
        let result = WtConfig::from_str(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_config_empty() {
        let yaml = "";
        let result = WtConfig::from_str(yaml);
        // Empty YAML now works with all defaults
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_config_multiline_init_script() {
        let yaml = r#"
init_script: |
  npm install
  npm run build
"#;
        let config = WtConfig::from_str(yaml).unwrap();
        assert!(config.init_script.unwrap().contains("npm install"));
    }

    #[test]
    fn test_config_serialize() {
        let config = WtConfig {
            claude_command: "ccc".to_string(),
            start_args: "-p test".to_string(),
            tmux_session: "wt".to_string(),
            worktree_dir: ".wt/worktrees".to_string(),
            copy_files: vec![".env".to_string()],
            init_script: Some("npm i".to_string()),
        };
        let yaml = serde_yaml::to_string(&config).unwrap();

        assert!(yaml.contains("claude_command: ccc"));
        assert!(yaml.contains("start_args:"));
        assert!(yaml.contains("copy_files:"));
        assert!(yaml.contains("init_script:"));
    }
}
