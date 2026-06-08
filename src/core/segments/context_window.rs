use super::{Segment, SegmentData};
use crate::config::{ContextWindow, InputData, ModelConfig, SegmentId, TranscriptEntry};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

#[derive(Default)]
pub struct ContextWindowSegment;

#[derive(Debug, Clone, Copy)]
struct ContextUsageSnapshot {
    used_tokens: u64,
    context_limit: u64,
    percentage: f64,
}

impl ContextWindowSegment {
    pub fn new() -> Self {
        Self
    }

    /// Get context limit for the specified model
    fn get_context_limit_for_model(model_id: &str) -> u32 {
        let model_config = ModelConfig::load();
        model_config.get_context_limit(model_id)
    }
}

impl Segment for ContextWindowSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        // Dynamically determine context limit based on current model ID
        let model_context_limit = Self::get_context_limit_for_model(&input.model.id) as u64;
        let usage = official_context_usage(input.context_window.as_ref(), model_context_limit)
            .or_else(|| {
                parse_transcript_usage(&input.transcript_path).and_then(|used_tokens| {
                    build_usage_snapshot(used_tokens as u64, model_context_limit, None)
                })
            });

        let (percentage_display, tokens_display) = match usage {
            Some(snapshot) => (
                format_percentage(snapshot.percentage),
                format_tokens(snapshot.used_tokens),
            ),
            None => ("-".to_string(), "-".to_string()),
        };

        let mut metadata = HashMap::new();
        match usage {
            Some(snapshot) => {
                metadata.insert("tokens".to_string(), snapshot.used_tokens.to_string());
                metadata.insert("percentage".to_string(), snapshot.percentage.to_string());
                metadata.insert("limit".to_string(), snapshot.context_limit.to_string());
            }
            None => {
                metadata.insert("tokens".to_string(), "-".to_string());
                metadata.insert("percentage".to_string(), "-".to_string());
                metadata.insert("limit".to_string(), model_context_limit.to_string());
            }
        }
        metadata.insert("model".to_string(), input.model.id.clone());

        Some(SegmentData {
            primary: format!("{} · {} tokens", percentage_display, tokens_display),
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::ContextWindow
    }
}

fn official_context_usage(
    context_window: Option<&ContextWindow>,
    model_context_limit: u64,
) -> Option<ContextUsageSnapshot> {
    let context_window = context_window?;
    let context_limit = context_window
        .context_window_size
        .filter(|limit| *limit > 0)
        .unwrap_or(model_context_limit);
    let percentage = context_window
        .used_percentage
        .filter(|value| value.is_finite());
    let used_tokens = official_used_tokens(context_window).or_else(|| {
        percentage.map(|value| ((value / 100.0) * context_limit as f64).round() as u64)
    })?;

    build_usage_snapshot(used_tokens, context_limit, percentage)
}

fn official_used_tokens(context_window: &ContextWindow) -> Option<u64> {
    if let Some(total_input_tokens) = context_window.total_input_tokens {
        return Some(total_input_tokens);
    }

    context_window.current_usage.as_ref().and_then(|usage| {
        let has_counted_tokens = usage.input_tokens.is_some()
            || usage.cache_creation_input_tokens.is_some()
            || usage.cache_read_input_tokens.is_some();

        if has_counted_tokens {
            Some(
                usage.input_tokens.unwrap_or(0)
                    + usage.cache_creation_input_tokens.unwrap_or(0)
                    + usage.cache_read_input_tokens.unwrap_or(0),
            )
        } else {
            None
        }
    })
}

fn build_usage_snapshot(
    used_tokens: u64,
    context_limit: u64,
    percentage: Option<f64>,
) -> Option<ContextUsageSnapshot> {
    if context_limit == 0 {
        return None;
    }

    let percentage = percentage
        .filter(|value| value.is_finite())
        .unwrap_or_else(|| (used_tokens as f64 / context_limit as f64) * 100.0);

    Some(ContextUsageSnapshot {
        used_tokens,
        context_limit,
        percentage,
    })
}

fn format_percentage(percentage: f64) -> String {
    if percentage.fract() == 0.0 {
        format!("{:.0}%", percentage)
    } else {
        format!("{:.1}%", percentage)
    }
}

fn format_tokens(tokens: u64) -> String {
    if tokens >= 1000 {
        let k_value = tokens as f64 / 1000.0;
        if k_value.fract() == 0.0 {
            format!("{}k", k_value as u64)
        } else {
            format!("{:.1}k", k_value)
        }
    } else {
        tokens.to_string()
    }
}

fn parse_transcript_usage<P: AsRef<Path>>(transcript_path: P) -> Option<u32> {
    let path = transcript_path.as_ref();

    // Try to parse from current transcript file
    if let Some(usage) = try_parse_transcript_file(path) {
        return Some(usage);
    }

    // If file doesn't exist, try to find usage from project history
    if !path.exists() {
        if let Some(usage) = try_find_usage_from_project_history(path) {
            return Some(usage);
        }
    }

    None
}

fn try_parse_transcript_file(path: &Path) -> Option<u32> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();

    if lines.is_empty() {
        return None;
    }

    // Check if the last line is a summary
    let last_line = lines.last()?.trim();
    if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(last_line) {
        if entry.r#type.as_deref() == Some("summary") {
            // Handle summary case: find usage by leafUuid
            if let Some(leaf_uuid) = &entry.leaf_uuid {
                let project_dir = path.parent()?;
                return find_usage_by_leaf_uuid(leaf_uuid, project_dir);
            }
        }
    }

    // Normal case: find the last assistant message in current file
    for line in lines.iter().rev() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(line) {
            if entry.r#type.as_deref() == Some("assistant") {
                if let Some(message) = &entry.message {
                    if let Some(raw_usage) = &message.usage {
                        let normalized = raw_usage.clone().normalize();
                        return Some(normalized.display_tokens());
                    }
                }
            }
        }
    }

    None
}

fn find_usage_by_leaf_uuid(leaf_uuid: &str, project_dir: &Path) -> Option<u32> {
    // Search for the leafUuid across all session files in the project directory
    let entries = fs::read_dir(project_dir).ok()?;

    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }

        if let Some(usage) = search_uuid_in_file(&path, leaf_uuid) {
            return Some(usage);
        }
    }

    None
}

fn search_uuid_in_file(path: &Path, target_uuid: &str) -> Option<u32> {
    let file = fs::File::open(path).ok()?;
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_default();

    // Find the message with target_uuid
    for line in &lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(line) {
            if let Some(uuid) = &entry.uuid {
                if uuid == target_uuid {
                    // Found the target message, check its type
                    if entry.r#type.as_deref() == Some("assistant") {
                        // Direct assistant message with usage
                        if let Some(message) = &entry.message {
                            if let Some(raw_usage) = &message.usage {
                                let normalized = raw_usage.clone().normalize();
                                return Some(normalized.display_tokens());
                            }
                        }
                    } else if entry.r#type.as_deref() == Some("user") {
                        // User message, need to find the parent assistant message
                        if let Some(parent_uuid) = &entry.parent_uuid {
                            return find_assistant_message_by_uuid(&lines, parent_uuid);
                        }
                    }
                    break;
                }
            }
        }
    }

    None
}

fn find_assistant_message_by_uuid(lines: &[String], target_uuid: &str) -> Option<u32> {
    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Ok(entry) = serde_json::from_str::<TranscriptEntry>(line) {
            if let Some(uuid) = &entry.uuid {
                if uuid == target_uuid && entry.r#type.as_deref() == Some("assistant") {
                    if let Some(message) = &entry.message {
                        if let Some(raw_usage) = &message.usage {
                            let normalized = raw_usage.clone().normalize();
                            return Some(normalized.display_tokens());
                        }
                    }
                }
            }
        }
    }

    None
}

fn try_find_usage_from_project_history(transcript_path: &Path) -> Option<u32> {
    let project_dir = transcript_path.parent()?;

    // Find the most recent session file in the project directory
    let mut session_files: Vec<PathBuf> = Vec::new();
    let entries = fs::read_dir(project_dir).ok()?;

    for entry in entries {
        let entry = entry.ok()?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("jsonl") {
            session_files.push(path);
        }
    }

    if session_files.is_empty() {
        return None;
    }

    // Sort by modification time (most recent first)
    session_files.sort_by_key(|path| {
        fs::metadata(path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::UNIX_EPOCH)
    });
    session_files.reverse();

    // Try to find usage from the most recent session
    for session_path in &session_files {
        if let Some(usage) = try_parse_transcript_file(session_path) {
            return Some(usage);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InputData;
    use serde_json::{json, Value};
    use std::time::{SystemTime, UNIX_EPOCH};

    struct TempTranscript {
        path: PathBuf,
    }

    impl TempTranscript {
        fn new(lines: &[&str]) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after Unix epoch")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "ccline-context-window-test-{}-{unique}.jsonl",
                std::process::id()
            ));
            fs::write(&path, lines.join("\n")).expect("test transcript should be writable");
            Self { path }
        }

        fn path_str(&self) -> &str {
            self.path
                .to_str()
                .expect("test transcript path should be valid UTF-8")
        }
    }

    impl Drop for TempTranscript {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    fn base_input(transcript_path: &str) -> Value {
        json!({
            "model": {
                "id": "claude-sonnet-4",
                "display_name": "Claude Sonnet 4"
            },
            "workspace": {
                "current_dir": "/tmp"
            },
            "transcript_path": transcript_path
        })
    }

    fn collect_from_value(value: Value) -> SegmentData {
        let input: InputData =
            serde_json::from_value(value).expect("test input should deserialize");
        ContextWindowSegment::new()
            .collect(&input)
            .expect("context window segment should render")
    }

    #[test]
    fn official_context_window_overrides_zero_transcript_usage() {
        let transcript = TempTranscript::new(&[
            r#"{"type":"assistant","message":{"usage":{"input_tokens":0,"output_tokens":0}}}"#,
        ]);
        let mut input = base_input(transcript.path_str());
        input["context_window"] = json!({
            "context_window_size": 1_000_000u64,
            "used_percentage": 23.0f64,
            "current_usage": {
                "input_tokens": 229_900u64,
                "output_tokens": 9_999u64,
                "cache_creation_input_tokens": 0u64,
                "cache_read_input_tokens": 0u64
            }
        });

        let segment = collect_from_value(input);

        assert_eq!(segment.primary, "23% · 229.9k tokens");
        assert_eq!(segment.metadata.get("tokens"), Some(&"229900".to_string()));
        assert_eq!(
            segment.metadata.get("limit"),
            Some(&"1000000".to_string())
        );
    }

    #[test]
    fn official_context_window_size_sets_limit_metadata() {
        let mut input = base_input("/tmp/missing-transcript.jsonl");
        input["context_window"] = json!({
            "total_input_tokens": 10_000u64,
            "context_window_size": 1_000_000u64
        });

        let segment = collect_from_value(input);

        assert_eq!(
            segment.metadata.get("limit"),
            Some(&"1000000".to_string())
        );
        assert_eq!(segment.metadata.get("tokens"), Some(&"10000".to_string()));
        assert_eq!(segment.primary, "1% · 10k tokens");
    }

    #[test]
    fn official_current_usage_does_not_count_output_tokens() {
        let mut input = base_input("/tmp/missing-transcript.jsonl");
        input["context_window"] = json!({
            "context_window_size": 200_000u64,
            "current_usage": {
                "input_tokens": 1_000u64,
                "output_tokens": 200u64,
                "cache_creation_input_tokens": 50u64,
                "cache_read_input_tokens": 25u64
            }
        });

        let segment = collect_from_value(input);

        assert_eq!(segment.metadata.get("tokens"), Some(&"1075".to_string()));
        assert_eq!(segment.primary, "0.5% · 1.1k tokens");
    }

    #[test]
    fn incomplete_official_context_window_falls_back_to_transcript_usage() {
        let transcript = TempTranscript::new(&[
            r#"{"type":"assistant","message":{"usage":{"input_tokens":0,"output_tokens":0}}}"#,
        ]);
        let mut input = base_input(transcript.path_str());
        input["context_window"] = json!({});

        let segment = collect_from_value(input);

        assert_eq!(segment.primary, "0% · 0 tokens");
        assert_eq!(segment.metadata.get("tokens"), Some(&"0".to_string()));
    }

    #[test]
    fn camel_case_official_context_window_fields_deserialize() {
        let mut input = base_input("/tmp/missing-transcript.jsonl");
        input["contextWindow"] = json!({
            "contextWindowSize": 1_000_000u64,
            "usedPercentage": 23.0f64,
            "currentUsage": {
                "inputTokens": 229_900u64,
                "outputTokens": 9_999u64,
                "cacheCreationInputTokens": 0u64,
                "cacheReadInputTokens": 0u64
            }
        });

        let segment = collect_from_value(input);

        assert_eq!(segment.primary, "23% · 229.9k tokens");
        assert_eq!(segment.metadata.get("tokens"), Some(&"229900".to_string()));
        assert_eq!(
            segment.metadata.get("limit"),
            Some(&"1000000".to_string())
        );
    }
}
