// Lesson 14
//! Application-owned storage selection, confirmation IO and append instrumentation.
use crate::{chooser::read_line, telemetry};
pub(super) use gdk_hello::learning::record::{
    read_record, record_tool_definition, RecordProposal, RECORD_TOOL,
};
use std::{io, path::PathBuf};

pub(super) fn record_path() -> PathBuf {
    telemetry::storage_path("learner-record.md")
}

pub(super) fn confirmation(prompt: &str) -> io::Result<bool> {
    let input = read_line(prompt)?;
    Ok(gdk_hello::learning::record::approval_value(
        input.as_deref(),
    ))
}

#[tracing::instrument(name = "memory_append", skip_all, fields(mlflow.spanType = "CHAIN"), err)]
pub(super) fn append_approved_note(
    path: &PathBuf,
    proposal: &RecordProposal,
) -> anyhow::Result<String> {
    let updated = gdk_hello::learning::record::append_approved_note(path, proposal)?;
    tracing::info!(
        "course.storage_outcome" = "approved note appended and synced",
        "course.memory_content" = updated.as_str()
    );
    Ok(updated)
}
