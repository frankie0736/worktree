//! Log parsing service for extracting metrics from agent JSONL logs.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use serde::Deserialize;

/// Metrics extracted from agent logs
#[derive(Debug, Default, Clone)]
pub struct LogMetrics {
    /// Total input tokens used
    pub input_tokens: u64,
    /// Total output tokens used
    pub output_tokens: u64,
    /// Cache creation tokens
    pub cache_creation_tokens: u64,
    /// Cache read tokens
    pub cache_read_tokens: u64,
    /// Number of conversation turns
    pub num_turns: u32,
    /// Context window size (from model info)
    pub context_window: u64,
}

impl LogMetrics {
    /// Calculate context usage percentage
    pub fn context_percent(&self) -> u8 {
        if self.context_window == 0 {
            return 0;
        }
        let used = self.input_tokens + self.output_tokens;
        ((used * 100) / self.context_window).min(100) as u8
    }
}

/// Parse JSONL log file and extract metrics
pub fn parse_log_file(path: &Path) -> Option<LogMetrics> {
    let file = File::open(path).ok()?;
    let reader = BufReader::new(file);

    let mut metrics = LogMetrics::default();
    // Default context window (will be overwritten if result entry found)
    metrics.context_window = 200_000;

    // Track the LAST assistant message's usage (not cumulative)
    // because cache_read_input_tokens represents current context size
    let mut last_cache_read: u64 = 0;
    let mut last_input: u64 = 0;
    let mut total_output: u64 = 0;
    let mut turn_count: u32 = 0;

    for line in reader.lines() {
        let line = line.ok()?;
        if line.is_empty() {
            continue;
        }

        // Try to parse as JSON
        if let Ok(entry) = serde_json::from_str::<LogEntry>(&line) {
            match entry {
                LogEntry::System {} | LogEntry::Init {} => {}
                LogEntry::Assistant(msg) => {
                    if let Some(usage) = msg.message.usage {
                        // cache_read = current context (conversation history from cache)
                        // input = new tokens this turn
                        // Together they represent current context size
                        last_cache_read = usage.cache_read_input_tokens.unwrap_or(0);
                        last_input = usage.input_tokens.unwrap_or(0);
                        total_output += usage.output_tokens.unwrap_or(0);
                        metrics.cache_creation_tokens +=
                            usage.cache_creation_input_tokens.unwrap_or(0);
                        turn_count += 1;
                    }
                }
                LogEntry::Result(result) => {
                    metrics.num_turns = result.num_turns.unwrap_or(turn_count);
                    if let Some(model_usage) = result.model_usage {
                        for (_, usage) in model_usage {
                            // Only get context_window from result
                            // Do NOT overwrite last_cache_read/last_input - those are cumulative in result
                            // We want the last assistant message's values for context calculation
                            if let Some(cw) = usage.context_window {
                                metrics.context_window = cw;
                            }
                            // Use cumulative output from result for total output
                            if let Some(output) = usage.output_tokens {
                                total_output = output;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Current context = cache_read (history) + input (new tokens)
    // Always use last assistant message's values for context calculation
    metrics.input_tokens = last_cache_read + last_input;
    metrics.output_tokens = total_output;
    metrics.cache_read_tokens = last_cache_read;
    metrics.num_turns = if metrics.num_turns > 0 {
        metrics.num_turns
    } else {
        turn_count
    };

    Some(metrics)
}

// Log entry types for deserialization

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum LogEntry {
    #[serde(rename = "system")]
    System {},
    Init {},
    Assistant(AssistantEntry),
    Result(ResultEntry),
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Deserialize)]
struct AssistantEntry {
    message: AssistantMessage,
}

#[derive(Debug, Deserialize)]
struct AssistantMessage {
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: Option<u64>,
    output_tokens: Option<u64>,
    cache_creation_input_tokens: Option<u64>,
    cache_read_input_tokens: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ResultEntry {
    num_turns: Option<u32>,
    #[serde(rename = "modelUsage")]
    model_usage: Option<std::collections::HashMap<String, ModelUsage>>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct ModelUsage {
    #[serde(rename = "inputTokens")]
    input_tokens: Option<u64>,
    #[serde(rename = "outputTokens")]
    output_tokens: Option<u64>,
    #[serde(rename = "cacheReadInputTokens")]
    cache_read_input_tokens: Option<u64>,
    #[serde(rename = "contextWindow")]
    context_window: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_empty_file() {
        let file = NamedTempFile::new().unwrap();
        let metrics = parse_log_file(file.path());
        assert!(metrics.is_some());
        let m = metrics.unwrap();
        assert_eq!(m.input_tokens, 0);
        assert_eq!(m.context_window, 200_000); // default
    }

    #[test]
    fn test_parse_with_assistant_and_result() {
        let mut file = NamedTempFile::new().unwrap();
        // Simulate real log: assistant messages followed by result
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"usage":{{"input_tokens":10,"output_tokens":100,"cache_read_input_tokens":5000}}}}}}"#
        ).unwrap();
        writeln!(
            file,
            r#"{{"type":"assistant","message":{{"usage":{{"input_tokens":20,"output_tokens":200,"cache_read_input_tokens":10000}}}}}}"#
        ).unwrap();
        writeln!(
            file,
            r#"{{"type":"result","num_turns":2,"modelUsage":{{"claude-sonnet":{{"inputTokens":30,"outputTokens":300,"contextWindow":200000}}}}}}"#
        ).unwrap();

        let metrics = parse_log_file(file.path()).unwrap();
        assert_eq!(metrics.num_turns, 2);
        // Context = last assistant's cache_read + input = 10000 + 20
        assert_eq!(metrics.input_tokens, 10020);
        // Output = cumulative from result
        assert_eq!(metrics.output_tokens, 300);
        assert_eq!(metrics.context_window, 200_000);
        // Context percent = 10020 / 200000 = 5%
        assert_eq!(metrics.context_percent(), 5);
    }

    #[test]
    fn test_context_percent() {
        let metrics = LogMetrics {
            input_tokens: 50_000,
            output_tokens: 10_000,
            context_window: 200_000,
            ..Default::default()
        };
        assert_eq!(metrics.context_percent(), 30);
    }
}
