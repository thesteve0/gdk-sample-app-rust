// Lesson 14
use rmcp::model::{JsonObject, Tool};
use serde::Deserialize;
use serde_json::json;
use std::{
    fs,
    io::{self, Write},
    os::unix::fs::OpenOptionsExt,
    path::PathBuf,
    sync::Arc,
};
pub const RECORD_TOOL: &str = "propose_learning_record";
pub const MAX_RECORD_BYTES: usize = 32_768;
pub fn read_record(path: &PathBuf) -> anyhow::Result<String> {
    let metadata_result = fs::symlink_metadata(path);
    let metadata = match metadata_result {
        Ok(metadata) => metadata,
        Err(error) => {
            if error.kind() == io::ErrorKind::NotFound {
                return Ok(String::new());
            }
            return Err(error.into());
        }
    };
    if !metadata.file_type().is_file() || metadata.len() > MAX_RECORD_BYTES as u64 {
        return Err(anyhow::anyhow!(
            "learning record must be a regular file of at most {} bytes",
            MAX_RECORD_BYTES
        ));
    }
    let text = fs::read_to_string(path)?;
    if text.len() > MAX_RECORD_BYTES {
        return Err(anyhow::anyhow!("learning record is too large"));
    }
    Ok(text)
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecordCategory {
    DiscussedTopics,
    StatedGoals,
    LearnerEvidence,
    UnresolvedQuestions,
    AgreedActivities,
}

impl RecordCategory {
    pub fn heading(self) -> &'static str {
        match self {
            Self::DiscussedTopics => "Discussed topics",
            Self::StatedGoals => "Learner-stated goals",
            Self::LearnerEvidence => "Evidence from learner answers (not mastery)",
            Self::UnresolvedQuestions => "Unresolved questions",
            Self::AgreedActivities => "Learner-agreed next activities",
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordProposal {
    pub category: RecordCategory,
    pub note: String,
    pub learner_quote: String,
}

pub fn record_tool_definition() -> Tool {
    let schema: JsonObject = serde_json::from_value(json!({
        "type": "object",
        "properties": {
            "category": {"type": "string", "enum": ["discussed_topics", "stated_goals", "learner_evidence", "unresolved_questions", "agreed_activities"]},
            "note": {"type": "string", "minLength": 1, "maxLength": 600},
            "learner_quote": {"type": "string", "minLength": 1, "maxLength": 600}
        },
        "required": ["category", "note", "learner_quote"],
        "additionalProperties": false
    })).expect("static record schema is an object");
    Tool::new(RECORD_TOOL, "Propose one concise learning note for approval after your final answer. This does NOT save it. Quote an actual learner reply exactly. Distinguish discussion, goals, limited evidence, questions, and next activities; never infer mastery. No file/path argument.", Arc::new(schema))
}

pub fn approval_value(input: Option<&str>) -> bool {
    match input {
        Some(input) => {
            input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes")
        }
        None => false,
    }
}

pub fn markdown_note(text: &str) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        if "\\`*_{}[]<>#!|".contains(character) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

pub fn append_approved_note(path: &PathBuf, proposal: &RecordProposal) -> anyhow::Result<String> {
    // Read current bytes, preserving hand edits; never rewrite existing content.
    let existing = read_record(path)?;
    let header = if existing.is_empty() {
        "# Learner learning record\n\nApproved notes, not a mastery certificate. Edit or delete between launches. Repeated category headings group chronological additions; remove obsolete next activities by hand.\n"
    } else {
        ""
    };
    let addition = format!(
        "{}\n## {}\n\n- {}\n  - Learner said: {}\n",
        header,
        proposal.category.heading(),
        markdown_note(&proposal.note),
        markdown_note(&proposal.learner_quote)
    );
    if existing.len() + addition.len() > MAX_RECORD_BYTES {
        return Err(anyhow::anyhow!(
            "learning record is full; edit it between launches before saving more notes"
        ));
    }
    // New files are owner-only on Linux. Existing permissions are not silently changed.
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(addition.as_bytes())?;
    file.sync_all()?;
    let updated = read_record(path)?;
    Ok(updated)
}

pub fn parse_learned(input: &str) -> Result<String, String> {
    if input.chars().any(char::is_control) {
        return Err("The command must be a single line without control characters.".to_string());
    }
    let command = input.trim();
    let tail = match command.strip_prefix("/learned ") {
        Some(tail) => tail.trim(),
        None => {
            return Err(
                "Use /learned <concept> to declare your own completion, not assessed mastery."
                    .to_string(),
            )
        }
    };
    if tail.is_empty() || tail.len() > 200 || tail.chars().any(char::is_control) {
        return Err("The concept must be one nonempty line of at most 200 UTF-8 bytes, without control characters.".to_string());
    }
    // No markup, terminal controls, path or hidden multiline command interpretation.
    if tail
        .chars()
        .any(|character| !character.is_alphanumeric() && !" -'.,():/".contains(character))
    {
        return Err(
            "Use plain words for the concept (letters, numbers, spaces, and simple punctuation)."
                .to_string(),
        );
    }
    Ok(tail.to_string())
}

pub fn completion_key(concept: &str) -> String {
    concept
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

pub fn append_completion(path: &PathBuf, concept: &str) -> anyhow::Result<String> {
    let existing = read_record(path)?;
    let header = if existing.is_empty() {
        "# Learner learning record\n\nLearner-approved notes and explicit declarations, not a mastery certificate.\n"
    } else {
        ""
    };
    let addition = format!("{}\n## Learner-declared completed concepts\n\n- {} (learner declaration; not assessed mastery)\n", header, markdown_note(concept));
    if existing.len() + addition.len() > MAX_RECORD_BYTES {
        return Err(anyhow::anyhow!(
            "learning record is full; edit it between launches"
        ));
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(addition.as_bytes())?;
    file.sync_all()?;
    read_record(path)
}

#[cfg(test)]
mod tests;
