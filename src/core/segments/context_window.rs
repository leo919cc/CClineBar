use super::{Segment, SegmentData};
use crate::config::{InputData, ModelConfig, SegmentId, TranscriptEntry};
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

/// Claude Code constants (from cli.js source)
const MAX_OUTPUT_CAP: u64 = 20_000; // a5Y — caps min(max_output, 20000)
const AUTOCOMPACT_BUFFER: u64 = 13_000; // wk8 — subtracted for autocompact threshold

#[derive(Default)]
pub struct ContextWindowSegment;

impl ContextWindowSegment {
    pub fn new() -> Self {
        Self
    }

    /// Get context limit for the specified model
    fn get_context_limit_for_model(model_id: &str) -> u32 {
        let model_config = ModelConfig::load();
        model_config.get_context_limit(model_id)
    }

    /// Get max output tokens for a model (used to compute effective context window).
    /// These match Claude Code's internal `uk8(model)` values.
    fn get_max_output_tokens(model_id: &str) -> u64 {
        let lower = model_id.to_lowercase();
        if lower.contains("opus") {
            32_000
        } else if lower.contains("sonnet") {
            16_384
        } else if lower.contains("haiku") {
            8_192
        } else {
            16_384 // conservative default
        }
    }

    /// Compute "context % left until auto-compaction" using Claude Code's UI formula.
    /// Formula from cli.js `Yl()`:
    ///   effective_window = context_window_size - min(max_output, 20000)
    ///   autocompact_threshold = effective_window - 13000
    ///   percent_left = max(0, round((threshold - tokens_used) / threshold * 100))
    fn compute_remaining_percent(
        context_window_size: u64,
        tokens_used: u64,
        model_id: &str,
    ) -> f64 {
        let max_output = Self::get_max_output_tokens(model_id);
        let output_reserve = max_output.min(MAX_OUTPUT_CAP);
        let effective_window = context_window_size.saturating_sub(output_reserve);
        let threshold = effective_window.saturating_sub(AUTOCOMPACT_BUFFER);

        if threshold == 0 {
            return 0.0;
        }

        let remaining = threshold.saturating_sub(tokens_used) as f64;
        (remaining / threshold as f64 * 100.0).max(0.0).round()
    }
}

impl Segment for ContextWindowSegment {
    fn collect(&self, input: &InputData) -> Option<SegmentData> {
        let mut metadata = HashMap::new();
        metadata.insert("model".to_string(), input.model.id.clone());

        // Primary path: use context_window JSON from Claude Code (matches UI formula exactly)
        if let Some(ref cw) = input.context_window {
            if let (Some(window_size), Some(ref usage)) =
                (cw.context_window_size, &cw.current_usage)
            {
                let tokens_used = usage.input_tokens.unwrap_or(0)
                    + usage.cache_creation_input_tokens.unwrap_or(0)
                    + usage.cache_read_input_tokens.unwrap_or(0)
                    + usage.output_tokens.unwrap_or(0);

                let percent_left = Self::compute_remaining_percent(
                    window_size,
                    tokens_used,
                    &input.model.id,
                );

                let display = if percent_left.fract() == 0.0 {
                    format!("{:.0}%", percent_left)
                } else {
                    format!("{:.1}%", percent_left)
                };

                metadata.insert("tokens".to_string(), tokens_used.to_string());
                metadata.insert("percentage".to_string(), percent_left.to_string());
                metadata.insert("limit".to_string(), window_size.to_string());

                return Some(SegmentData {
                    primary: display,
                    secondary: String::new(),
                    metadata,
                });
            }
        }

        // Fallback: parse transcript file (for older Claude Code versions without context_window)
        let context_limit = Self::get_context_limit_for_model(&input.model.id);
        let context_used_token_opt = parse_transcript_usage(&input.transcript_path);

        let percentage_display = match context_used_token_opt {
            Some(context_used_token) => {
                let percent_left = Self::compute_remaining_percent(
                    context_limit as u64,
                    context_used_token as u64,
                    &input.model.id,
                );

                metadata.insert("tokens".to_string(), context_used_token.to_string());
                metadata.insert("percentage".to_string(), percent_left.to_string());

                if percent_left.fract() == 0.0 {
                    format!("{:.0}%", percent_left)
                } else {
                    format!("{:.1}%", percent_left)
                }
            }
            None => {
                metadata.insert("tokens".to_string(), "-".to_string());
                metadata.insert("percentage".to_string(), "-".to_string());
                "-".to_string()
            }
        };
        metadata.insert("limit".to_string(), context_limit.to_string());

        Some(SegmentData {
            primary: percentage_display,
            secondary: String::new(),
            metadata,
        })
    }

    fn id(&self) -> SegmentId {
        SegmentId::ContextWindow
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
