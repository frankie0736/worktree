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

    // Track running totals from assistant messages
    let mut running_input: u64 = 0;
    let mut running_output: u64 = 0;
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
                    // Accumulate usage from each assistant message
                    if let Some(usage) = msg.message.usage {
                        // These are per-message values, accumulate them
                        running_input += usage.input_tokens.unwrap_or(0);
                        running_output += usage.output_tokens.unwrap_or(0);
                        metrics.cache_creation_tokens +=
                            usage.cache_creation_input_tokens.unwrap_or(0);
                        metrics.cache_read_tokens += usage.cache_read_input_tokens.unwrap_or(0);
                        turn_count += 1;
                    }
                }
                LogEntry::Result(result) => {
                    // Final result has accurate totals
                    metrics.num_turns = result.num_turns.unwrap_or(turn_count);
                    if let Some(model_usage) = result.model_usage {
                        for (_, usage) in model_usage {
                            if let Some(cw) = usage.context_window {
                                metrics.context_window = cw;
                            }
                            // Use result totals (more accurate than accumulation)
                            if let Some(input) = usage.input_tokens {
                                running_input = input;
                            }
                            if let Some(output) = usage.output_tokens {
                                running_output = output;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Use running totals
    metrics.input_tokens = running_input;
    metrics.output_tokens = running_output;
    if metrics.num_turns == 0 {
        metrics.num_turns = turn_count;
    }

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
struct ModelUsage {
    #[serde(rename = "inputTokens")]
    input_tokens: Option<u64>,
    #[serde(rename = "outputTokens")]
    output_tokens: Option<u64>,
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
    fn test_parse_result_entry() {
        let mut file = NamedTempFile::new().unwrap();
        writeln!(
            file,
            r#"{{"type":"result","num_turns":5,"modelUsage":{{"claude-sonnet":{{"inputTokens":1000,"outputTokens":500,"contextWindow":200000}}}}}}"#
        )
        .unwrap();

        let metrics = parse_log_file(file.path()).unwrap();
        assert_eq!(metrics.num_turns, 5);
        assert_eq!(metrics.input_tokens, 1000);
        assert_eq!(metrics.output_tokens, 500);
        assert_eq!(metrics.context_window, 200_000);
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
