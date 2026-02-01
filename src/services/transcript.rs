//! Transcript service for reading Claude Code session transcripts.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Metrics extracted from transcript
#[derive(Debug, Default, Clone)]
pub struct TranscriptMetrics {
    /// Input tokens (context size from last assistant message)
    pub input_tokens: u64,
    /// Output tokens (cumulative)
    pub output_tokens: u64,
    /// Number of conversation turns
    pub num_turns: u32,
    /// Context window size
    pub context_window: u64,
    /// Final summary/result from last assistant message
    pub summary: Option<String>,
    /// Whether the session completed normally
    pub completed: bool,
    /// Session start timestamp (from first entry)
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    /// Session end timestamp (from last entry)
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl TranscriptMetrics {
    /// Calculate context usage percentage
    pub fn context_percent(&self) -> u8 {
        if self.context_window == 0 {
            return 0;
        }
        let used = self.input_tokens + self.output_tokens;
        ((used * 100) / self.context_window).min(100) as u8
    }

    /// Calculate session duration in seconds
    pub fn duration_secs(&self) -> Option<i64> {
        match (self.started_at, self.finished_at) {
            (Some(start), Some(end)) => Some(end.signed_duration_since(start).num_seconds()),
            _ => None,
        }
    }
}

/// Convert a filesystem path to Claude Code's project directory name.
///
/// Claude Code escapes paths by replacing `/` and `.` with `-`.
/// Example: `/Users/foo/project/.wt` -> `-Users-foo-project--wt`
pub fn project_dir_name(path: &str) -> String {
    path.replace('/', "-").replace('.', "-")
}

/// Get the Claude Code projects directory.
pub fn claude_projects_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude/projects"))
}

/// Get the transcript file path for a given worktree path and session ID.
pub fn transcript_path(worktree_path: &str, session_id: &str) -> Option<PathBuf> {
    let projects_dir = claude_projects_dir()?;
    let dir_name = project_dir_name(worktree_path);
    Some(projects_dir.join(dir_name).join(format!("{}.jsonl", session_id)))
}

/// Parse a transcript file and extract metrics.
pub fn parse_transcript(path: &Path) -> Option<TranscriptMetrics> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut metrics = TranscriptMetrics::default();
    metrics.context_window = 200_000; // Default

    let mut last_cache_read: u64 = 0;
    let mut last_input: u64 = 0;
    let mut total_output: u64 = 0;
    let mut turn_count: u32 = 0;
    let mut last_assistant_text: Option<String> = None;
    let mut first_timestamp: Option<chrono::DateTime<chrono::Utc>> = None;
    let mut last_timestamp: Option<chrono::DateTime<chrono::Utc>> = None;

    for line in reader.lines() {
        let line = line.ok()?;
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(&line) {
            // Extract timestamp from every entry
            if let Some(ts) = &entry.timestamp {
                if let Ok(parsed) = chrono::DateTime::parse_from_rfc3339(ts) {
                    let utc = parsed.with_timezone(&chrono::Utc);
                    if first_timestamp.is_none() {
                        first_timestamp = Some(utc);
                    }
                    last_timestamp = Some(utc);
                }
            }

            match entry.r#type.as_str() {
                "assistant" => {
                    if let Some(msg) = entry.message {
                        // Extract usage info
                        if let Some(usage) = msg.usage {
                            last_cache_read = usage.cache_read_input_tokens.unwrap_or(0);
                            last_input = usage.input_tokens.unwrap_or(0);
                            total_output += usage.output_tokens.unwrap_or(0);
                        }

                        // Extract text content for summary
                        if let Some(content) = msg.content {
                            for item in content {
                                if item.r#type == "text" {
                                    if let Some(text) = item.text {
                                        last_assistant_text = Some(text);
                                    }
                                }
                            }
                        }

                        turn_count += 1;
                    }
                }
                "system" => {
                    // Check for init entry to get context window
                    if entry.subtype.as_deref() == Some("init") {
                        // Could extract model info here if needed
                    }
                }
                _ => {}
            }
        }
    }

    // Context = cache_read (history) + input (new tokens)
    metrics.input_tokens = last_cache_read + last_input;
    metrics.output_tokens = total_output;
    metrics.num_turns = turn_count;
    metrics.summary = last_assistant_text;
    metrics.completed = turn_count > 0; // Consider completed if there's at least one turn
    metrics.started_at = first_timestamp;
    metrics.finished_at = last_timestamp;

    Some(metrics)
}

// Deserialization structs

#[derive(Debug, Deserialize)]
struct TranscriptEntry {
    r#type: String,
    #[serde(default)]
    subtype: Option<String>,
    #[serde(default)]
    message: Option<TranscriptMessage>,
    #[serde(default)]
    timestamp: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TranscriptMessage {
    #[serde(default)]
    usage: Option<Usage>,
    #[serde(default)]
    content: Option<Vec<ContentItem>>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ContentItem {
    r#type: String,
    #[serde(default)]
    text: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_dir_name() {
        assert_eq!(
            project_dir_name("/Users/foo/project"),
            "-Users-foo-project"
        );
        // . is also replaced with -
        assert_eq!(
            project_dir_name("/Users/foo/project/.wt-worktrees/task"),
            "-Users-foo-project--wt-worktrees-task"
        );
    }

    #[test]
    fn test_context_percent() {
        let metrics = TranscriptMetrics {
            input_tokens: 50_000,
            output_tokens: 10_000,
            context_window: 200_000,
            ..Default::default()
        };
        assert_eq!(metrics.context_percent(), 30);
    }

    #[test]
    fn test_context_percent_zero_window() {
        let metrics = TranscriptMetrics {
            input_tokens: 50_000,
            output_tokens: 10_000,
            context_window: 0,
            ..Default::default()
        };
        assert_eq!(metrics.context_percent(), 0);
    }
}
