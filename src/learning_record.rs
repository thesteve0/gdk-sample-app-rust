// Lesson 12
use crate::{chooser::read_line, REPOSITORY_ROOT};
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
pub(super) const RECORD_TOOL: &str = "propose_learning_record";
const MAX_RECORD_BYTES: usize = 32_768;
const RECORD_FILE: &str = "learner-record.md";
// The record is purposefully smaller than the transcript and remains editable Markdown.
pub(super) fn record_path() -> PathBuf {
    PathBuf::from(REPOSITORY_ROOT).join(RECORD_FILE)
}

pub(super) fn read_record(path: &PathBuf) -> anyhow::Result<String> {
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
pub(super) enum RecordCategory {
    DiscussedTopics,
    StatedGoals,
    LearnerEvidence,
    UnresolvedQuestions,
    AgreedActivities,
}

impl RecordCategory {
    pub(super) fn heading(self) -> &'static str {
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
pub(super) struct RecordProposal {
    pub(super) category: RecordCategory,
    pub(super) note: String,
    pub(super) learner_quote: String,
}

pub(super) fn record_tool_definition() -> Tool {
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

pub(super) fn confirmation(prompt: &str) -> io::Result<bool> {
    let input = read_line(prompt)?;
    Ok(approval_value(input.as_deref()))
}

fn approval_value(input: Option<&str>) -> bool {
    match input {
        Some(input) => {
            input.trim().eq_ignore_ascii_case("y") || input.trim().eq_ignore_ascii_case("yes")
        }
        None => false,
    }
}

fn markdown_note(text: &str) -> String {
    let mut escaped = String::new();
    for character in text.chars() {
        if "\\`*_{}[]<>#!|".contains(character) {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

pub(super) fn append_approved_note(
    path: &PathBuf,
    proposal: &RecordProposal,
) -> anyhow::Result<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    #[test]
    fn consent_and_approval_default_to_no() {
        for input in [None, Some(""), Some("n"), Some("/quit"), Some("save it")] {
            assert!(!approval_value(input));
        }
        assert!(approval_value(Some(" Y ")));
        assert!(approval_value(Some("Yes")));
    }

    #[test]
    fn markdown_record_preserves_manual_edits_and_bounds_permissions() -> anyhow::Result<()> {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let directory = env::temp_dir().join(format!("gdk-lesson11-record-{}", std::process::id()));
        fs::create_dir(&directory)?;
        let path = directory.join("record.md");
        assert_eq!(read_record(&path)?, "");
        let proposal = RecordProposal {
            category: RecordCategory::AgreedActivities,
            note: "Compare *orders*".to_string(),
            learner_quote: "I agree <yes>".to_string(),
        };
        let saved = append_approved_note(&path, &proposal)?;
        assert!(saved.contains("## Learner-agreed next activities"));
        assert!(saved.contains("Compare \\*orders\\*"));
        assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
        let hand_edit = "# My hand-edited notes\n\nKeep these bytes exactly.\n";
        fs::write(&path, hand_edit)?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o640))?;
        let updated = append_approved_note(&path, &proposal)?;
        assert!(updated.starts_with(hand_edit));
        assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o640);
        fs::write(&path, "x".repeat(MAX_RECORD_BYTES))?;
        assert!(append_approved_note(&path, &proposal).is_err());
        assert_eq!(fs::metadata(&path)?.len(), MAX_RECORD_BYTES as u64);
        fs::write(&path, "x".repeat(MAX_RECORD_BYTES + 1))?;
        assert!(read_record(&path).is_err());
        fs::write(&path, [0xff])?;
        assert!(read_record(&path).is_err());
        let link = directory.join("link.md");
        symlink(&path, &link)?;
        assert!(read_record(&link).is_err());
        assert!(append_approved_note(&link, &proposal).is_err());
        assert!(read_record(&directory).is_err());
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
