// Lesson 14
use crate::{
    calculator::tool_definition, learning_record::record_tool_definition, retrieval, telemetry,
    ChatSession, LessonContext,
};
use goose_agent::operation::messages_since_kickoff;
use std::{fs, io::Write, path::PathBuf};
// Markdown diagnostics are presentation, never an input to the tutor or learner memory.
pub(super) struct DiagnosticLog {
    file: fs::File,
}

impl DiagnosticLog {
    pub(super) fn create() -> anyhow::Result<Self> {
        let path = telemetry::storage_path("session-output.md");
        Self::create_at(path)
    }

    pub(super) fn create_at(path: PathBuf) -> anyhow::Result<Self> {
        let mut file = fs::File::create(path)?;
        writeln!(file, "# Learning partner — launch diagnostics\n\nOverwritten at every launch. May contain personal conversation; ignored by Git, not access-controlled. Not learner memory. No provider configuration or credentials are deliberately recorded.\n")?;
        file.flush()?;
        Ok(Self { file })
    }

    pub(super) fn block(
        &mut self,
        heading: &str,
        language: &str,
        text: &str,
    ) -> anyhow::Result<()> {
        // Diagnostics are already complete locally; export only ordinary evaluation boundaries.
        if matches!(
            heading,
            "Learning note decision"
                | "Learner completion disposition AFTER local append"
                | "Warning"
                | "Session ended"
        ) {
            tracing::info!("course.diagnostic" = heading, "course.evidence" = text);
        }
        let block = markdown_block(heading, language, text);
        self.file.write_all(block.as_bytes())?;
        // Surface a logging failure now rather than hiding it until process exit.
        self.file.flush()?;
        Ok(())
    }

    pub(super) fn lesson(
        &mut self,
        session_id: &str,
        context: &LessonContext,
    ) -> anyhow::Result<()> {
        writeln!(
            self.file,
            "\n## Lesson: {} — Session {}\n",
            context.title, session_id
        )?;
        self.block(
            "Application → provider: selected context (prompt parts, not an exact request dump)",
            "rust",
            &format!("{:#?}", context.prompt_parts()),
        )?;
        self.block(
            "Available calculator definition",
            "json",
            &serde_json::to_string_pretty(&tool_definition())?,
        )?;
        self.block(
            "Available educational retrieval definition",
            "json",
            &serde_json::to_string_pretty(&retrieval::tool_definition())?,
        )?;
        if context.persistence_enabled {
            self.block(
                "Available proposal definition",
                "json",
                &serde_json::to_string_pretty(&record_tool_definition())?,
            )?;
        }
        Ok(())
    }

    pub(super) fn turn(&mut self, turn_number: usize, session: &ChatSession) -> anyhow::Result<()> {
        writeln!(self.file, "\n### Turn {}\n", turn_number)?;
        let turn = messages_since_kickoff(&session.conversation)?;
        let input = match turn.first() {
            Some(input) => input,
            None => return Err(anyhow::anyhow!("missing turn input")),
        };
        let heading = if turn_number == 1 {
            "Application → provider: opening input (not a learner reply)"
        } else {
            "Learner input → application → provider"
        };
        self.block(heading, "text", &input.as_concat_text())?;
        self.block(
            "Complete Session before run",
            "rust",
            &session_fields(session),
        )
    }

    pub(super) fn after(&mut self, session: &ChatSession) -> anyhow::Result<()> {
        self.block(
            "Complete Session after run (possibly partial on failure)",
            "rust",
            &session_fields(session),
        )?;
        let turn = messages_since_kickoff(&session.conversation)?;
        for message in turn {
            for block in &message.content {
                if let Some(request) = block.as_tool_request() {
                    self.block(
                        &format!("Provider → application: saved request {}", request.id),
                        "rust",
                        &format!("{:#?}", request),
                    )?;
                    if let Ok(call) = &request.tool_call {
                        self.block(
                            "Tool name and arguments",
                            "json",
                            &serde_json::to_string_pretty(call)?,
                        )?;
                    }
                }
                if let Some(response) = block.as_tool_response() {
                    self.block(
                        &format!(
                            "Application, as the tool → provider: saved result {}",
                            response.id
                        ),
                        "rust",
                        &format!("{:#?}", response),
                    )?;
                }
            }
            if message.role == rmcp::model::Role::Assistant {
                let text = message.as_concat_text();
                if !text.is_empty() {
                    self.block(
                        "Saved assembled tutor response (not streaming deltas)",
                        "text",
                        &text,
                    )?;
                }
            }
        }
        Ok(())
    }
}

// Remove terminal control sequences from file presentation without changing recorded state.
pub(super) fn plain_output(text: &str) -> String {
    let mut output = String::new();
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character == '\x1b' {
            match characters.next() {
                Some('[') => {
                    for next in characters.by_ref() {
                        if ('@'..='~').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    let mut escaped = false;
                    for next in characters.by_ref() {
                        if next == '\x07' || (escaped && next == '\\') {
                            break;
                        }
                        escaped = next == '\x1b';
                    }
                }
                Some(_) | None => {}
            }
        } else if !character.is_control() || character == '\n' || character == '\t' {
            output.push(character);
        }
    }
    output
}

fn markdown_block(heading: &str, language: &str, text: &str) -> String {
    let plain = plain_output(text);
    // A learner/model can include Markdown fences; choose a longer enclosing fence.
    let mut longest_run = 0;
    for run in plain.split(|character| character != '`') {
        longest_run = std::cmp::max(longest_run, run.len());
    }
    let fence = "`".repeat(std::cmp::max(3, longest_run + 1));
    let safe_heading = plain_output(heading).replace('\n', " ");
    format!(
        "\n#### {}\n\n{}{}\n{}\n{}\n",
        safe_heading, fence, language, plain, fence
    )
}

// Debug formatting includes fields that JSON serialization can omit.
pub(super) fn session_fields(session: &ChatSession) -> String {
    let existing = format!(
        "id: {:?}\nconversation: {:#?}\nusage: {:#?}\napplied_passes: {:?}\nhalt_reason: {:?}\nlesson: {:#?}\n",
        session.id, session.conversation.messages(), session.usage,
        session.applied_passes, session.halt_reason, session.lesson,
    );
    format!("{}revision: {:?}\ngenuine_reply_ids: {:#?}\nsettled_approvals: {:#?}\ncompleted_concepts: {:#?}\npolicy: {:#?}\nmaintenance: {:#?}\nmaintenance_report: {:#?}\n", existing, session.revision, session.genuine_reply_ids, session.settled_approvals, session.completed_concepts, session.policy, session.maintenance, session.maintenance_report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        calculator::TOOL_NAME,
        presentation::{styled, TextStyle},
        tests::request,
        ChatRuntime, SESSION_ID,
    };
    use goose_agent::machine::SessionLoader;
    use goose_providers::conversation::message::Message;
    use rmcp::model::{CallToolResult, ContentBlock};
    use std::env;
    #[test]
    fn markdown_fences_and_control_sequences_are_presentation_only() {
        let payload = "text\n```rust\ncode\n```\n\x1b[35mrequest\x1b[0m\x1b]0;title\x07";
        let block = markdown_block("Saved response", "text", payload);
        assert!(block.contains("````text\n"));
        assert!(block.contains("```rust\ncode\n```\nrequest"));
        assert!(!block.contains('\x1b'));
        assert!(payload.contains('\x1b')); // No mutation of protocol data.
        for style in [
            TextStyle::Heading,
            TextStyle::Blue,
            TextStyle::Magenta,
            TextStyle::Green,
            TextStyle::Yellow,
            TextStyle::Error,
        ] {
            let colored = styled("value", style, true);
            assert!(colored.ends_with("\x1b[0m"));
            assert_eq!(plain_output(&colored), "value");
            assert_eq!(styled("value", style, false), "value");
        }
    }

    #[tokio::test]
    async fn diagnostic_log_truncates_launch_and_retains_complete_turns_and_partial_state(
    ) -> anyhow::Result<()> {
        let path = env::temp_dir().join(format!("gdk-lesson10-test-{}.md", std::process::id()));
        fs::write(&path, "obsolete launch")?;
        let mut log = DiagnosticLog::create_at(path.clone())?;
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        log.lesson(SESSION_ID, &session.lesson)?;
        log.turn(1, &session)?;
        session
            .conversation
            .push(request("diagnostic-call", TOOL_NAME, 200));
        session
            .conversation
            .push(Message::user().with_tool_response(
                "diagnostic-call",
                Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
            ));
        session
            .conversation
            .push(Message::assistant().with_text("One assembled answer, not fragments."));
        session.halt_reason = Some("partial failure".to_string());
        log.after(&session)?;
        log.block("Failure", "text", "partial failure")?;
        log.lesson("another-session", &session.lesson)?;
        let text = fs::read_to_string(&path)?;
        assert!(!text.contains("obsolete launch"));
        assert!(!text.contains('\x1b'));
        assert!(text.contains(&session_fields(&session)));
        assert!(text.contains("user_visible: true"));
        assert!(text.contains("inference: None"));
        assert!(text.contains("halt_reason: Some("));
        assert!(text.contains("saved request diagnostic-call"));
        assert!(text.contains("saved result diagnostic-call"));
        assert!(text.contains("One assembled answer, not fragments."));
        assert!(text.contains("Session another-session"));
        let mut next_launch = DiagnosticLog::create_at(path.clone())?;
        next_launch.block("New launch", "text", "fresh")?;
        let fresh = fs::read_to_string(&path)?;
        assert!(!fresh.contains("diagnostic-call"));
        assert!(fresh.contains("New launch"));
        drop(log);
        drop(next_launch);
        fs::remove_file(path)?;
        // A bad destination is not silently ignored.
        assert!(DiagnosticLog::create_at(env::temp_dir()).is_err());
        // Linux's /dev/full accepts opening but fails every write, covering append failures.
        let file = fs::OpenOptions::new().write(true).open("/dev/full")?;
        let mut failed_log = DiagnosticLog { file };
        assert!(failed_log.block("Failure", "text", "cannot save").is_err());
        Ok(())
    }
}
