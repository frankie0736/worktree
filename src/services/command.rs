//! Command execution utilities for git and tmux operations.

use std::process::{Command, Output};

use crate::error::{Result, WtError};

/// A helper for running external commands with consistent error handling.
pub struct CommandRunner {
    program: &'static str,
    error_mapper: fn(String) -> WtError,
}

impl CommandRunner {
    /// Create a runner for git commands.
    pub fn git() -> Self {
        Self {
            program: "git",
            error_mapper: WtError::Git,
        }
    }

    /// Create a runner for tmux commands.
    pub fn tmux() -> Self {
        Self {
            program: "tmux",
            error_mapper: WtError::Tmux,
        }
    }

    /// Run a command and check for success.
    pub fn run(&self, args: &[&str]) -> Result<()> {
        let output = self.execute(args)?;
        self.check_status(output)
    }

    /// Check if a command succeeds without returning an error.
    pub fn success(&self, args: &[&str]) -> bool {
        Command::new(self.program)
            .args(args)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    fn execute(&self, args: &[&str]) -> Result<Output> {
        Command::new(self.program)
            .args(args)
            .output()
            .map_err(|e| (self.error_mapper)(e.to_string()))
    }

    fn check_status(&self, output: Output) -> Result<()> {
        if output.status.success() {
            Ok(())
        } else {
            Err((self.error_mapper)(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_runner_success() {
        let runner = CommandRunner::git();
        assert!(runner.success(&["--version"]));
    }

    #[test]
    fn test_git_runner_failure() {
        let runner = CommandRunner::git();
        let result = runner.run(&["invalid-command-that-does-not-exist"]);
        assert!(result.is_err());
    }
}
