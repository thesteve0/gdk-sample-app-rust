// Supplied presentation helpers; they never change provider input or recorded state.
use std::{
    env,
    io::{self, IsTerminal},
};

use crate::ChatSession;

pub(super) fn display_session(label: &str, session: &ChatSession) {
    print_heading(&format!("\nSession {}", label.to_lowercase()));
    print!(
        "{}",
        session_fields_with_color(session, stdout_color_enabled())
    );
}

#[cfg(test)]
pub(super) fn session_fields(session: &ChatSession) -> String {
    session_fields_with_color(session, false)
}

// Color field labels gold, not Debug type names or string values containing colons.
fn gold_session_attributes(fields: &str, enabled: bool) -> String {
    if !enabled {
        return fields.to_string();
    }
    let mut output = String::new();
    for line in fields.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        let mut label_end = None;
        if trimmed.starts_with('"') {
            let mut escaped = false;
            for (index, character) in trimmed.char_indices().skip(1) {
                if escaped {
                    escaped = false;
                } else if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    if trimmed[index + 1..].starts_with(':') {
                        label_end = Some(index + 1);
                    }
                    break;
                }
            }
        } else if let Some(colon) = trimmed.find(':') {
            let name = &trimmed[..colon];
            if !name.is_empty()
                && name
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
                && !name.as_bytes()[0].is_ascii_digit()
                && !trimmed[colon + 1..].starts_with(':')
            {
                label_end = Some(colon);
            }
        }
        match label_end {
            Some(end) => {
                output.push_str(&line[..indentation]);
                output.push_str("\x1b[38;2;255;215;0m");
                output.push_str(&trimmed[..end]);
                // Restore the default foreground without resetting usage dimming.
                output.push_str("\x1b[39m");
                output.push_str(&trimmed[end..]);
            }
            None => output.push_str(line),
        }
    }
    output
}

fn session_fields_with_color(session: &ChatSession, color_enabled: bool) -> String {
    // Debug formatting includes absent/default fields that JSON serialization can omit.
    let conversation = format!(
        "id: {:?}\nconversation: {:#?}\n",
        session.id,
        session.conversation.messages()
    );
    let usage = format!("usage: {:#?}", session.usage);
    let displayed_conversation = gold_session_attributes(&conversation, color_enabled);
    let gold_usage = gold_session_attributes(&usage, color_enabled);
    let displayed_usage = styled(&gold_usage, TextStyle::Dim, color_enabled);
    format!("{}{}\n\n\n", displayed_conversation, displayed_usage)
}

// Terminal styling is presentation only; redirected transcripts contain no ANSI codes.
#[derive(Clone, Copy)]
enum TextStyle {
    Heading,
    Error,
    Dim,
}

fn color_enabled(is_terminal: bool, no_color: bool, dumb_terminal: bool) -> bool {
    is_terminal && !no_color && !dumb_terminal
}

fn terminal_color_enabled(is_terminal: bool) -> bool {
    let no_color = env::var_os("NO_COLOR").is_some();
    let dumb_terminal = env::var_os("TERM") == Some("dumb".into());
    color_enabled(is_terminal, no_color, dumb_terminal)
}

fn stdout_color_enabled() -> bool {
    terminal_color_enabled(io::stdout().is_terminal())
}

fn styled(text: &str, style: TextStyle, enabled: bool) -> String {
    if !enabled {
        return text.to_string();
    }
    let code = match style {
        TextStyle::Heading => "1;36",
        TextStyle::Error => "1;31",
        TextStyle::Dim => "2",
    };
    format!("\x1b[{}m{}\x1b[0m", code, text)
}

pub(super) fn print_heading(text: &str) {
    println!(
        "{}",
        styled(text, TextStyle::Heading, stdout_color_enabled())
    );
}

pub(super) fn print_error(text: &str) {
    let enabled = terminal_color_enabled(io::stderr().is_terminal());
    eprintln!("{}", styled(text, TextStyle::Error, enabled));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ChatRuntime, SESSION_ID};
    use goose_agent::machine::SessionLoader;
    use goose_providers::conversation::{
        message::Message,
        token_usage::{ProviderUsage, Usage},
    };

    #[test]
    fn terminal_colors_respect_redirection_no_color_and_dumb_terminals() {
        assert!(color_enabled(true, false, false));
        assert!(!color_enabled(false, false, false));
        assert!(!color_enabled(true, true, false));
        assert!(!color_enabled(true, false, true));
        assert_eq!(styled("payload", TextStyle::Heading, false), "payload");
        assert_eq!(
            styled("payload", TextStyle::Heading, true),
            "\x1b[1;36mpayload\x1b[0m"
        );
        assert_eq!(
            styled("failure", TextStyle::Error, true),
            "\x1b[1;31mfailure\x1b[0m"
        );
        assert_eq!(styled("usage", TextStyle::Dim, true), "\x1b[2musage\x1b[0m");
    }

    #[test]
    fn session_attribute_gold_leaves_values_and_type_names_unchanged() {
        let fields = "id: \"value: unchanged\"\n    text: \"role: User\",\n    \"entry_price\": String(\"51.20\"),\n    TextContent {\n        \"not a key: just text\",\n    }\n";
        let gold = gold_session_attributes(fields, true);
        assert!(gold.contains("\x1b[38;2;255;215;0mid\x1b[39m: \"value: unchanged\""));
        assert!(gold.contains("\x1b[38;2;255;215;0mtext\x1b[39m: \"role: User\""));
        assert!(gold.contains("\x1b[38;2;255;215;0m\"entry_price\"\x1b[39m: String(\"51.20\")"));
        assert_eq!(gold.matches("\x1b[38;2;255;215;0m").count(), 3);
        assert_eq!(
            gold.replace("\x1b[38;2;255;215;0m", "")
                .replace("\x1b[39m", ""),
            fields
        );
        assert_eq!(gold_session_attributes(fields, false), fields);
    }

    #[tokio::test]
    async fn session_styling_dims_only_usage_and_preserves_complete_plain_text(
    ) -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let mut session = runtime.load(SESSION_ID).await?;
        session.usage.push(ProviderUsage::new(
            "test-model".to_string(),
            Usage::default(),
        ));
        let mut message = Message::assistant().with_text("Paris.");
        message.metadata.usage = Some(Box::new(
            goose_providers::conversation::message::MessageUsage::default(),
        ));
        session.conversation.push(message);
        let colored = session_fields_with_color(&session, true);
        let usage = format!("usage: {:#?}", session.usage);
        let gold_usage = gold_session_attributes(&usage, true);
        let expected_dimmed = format!("\x1b[2m{}\x1b[0m", gold_usage);
        assert!(colored.contains(&expected_dimmed));
        assert_eq!(colored.matches("\x1b[2m").count(), 1);
        let conversation_end = colored.find("\x1b[2m").expect("top-level usage style");
        let displayed_conversation = &colored[..conversation_end];
        assert!(displayed_conversation.contains("\x1b[38;2;255;215;0musage\x1b[39m: Some("));
        assert!(
            displayed_conversation.contains("\x1b[38;2;255;215;0mcache_read_tokens\x1b[39m: None")
        );
        assert_eq!(colored.matches("\x1b[0m").count(), 1);
        let plain = colored
            .replace("\x1b[2m", "")
            .replace("\x1b[0m", "")
            .replace("\x1b[38;2;255;215;0m", "")
            .replace("\x1b[39m", "");
        assert_eq!(plain, session_fields(&session));
        let original_fields = format!(
            "id: {:?}\nconversation: {:#?}\nusage: {:#?}\n\n\n",
            session.id,
            session.conversation.messages(),
            session.usage
        );
        assert_eq!(plain, original_fields);
        assert!(!session_fields_with_color(&session, false).contains('\x1b'));
        assert!(colored.ends_with("\x1b[0m\n\n\n"));
        assert!(colored.contains("\x1b[38;2;255;215;0mrole\x1b[39m: User"));
        assert!(colored.contains("\x1b[38;2;255;215;0minput_tokens\x1b[39m:"));
        Ok(())
    }

    #[tokio::test]
    async fn session_view_includes_all_message_and_metadata_fields() -> anyhow::Result<()> {
        let runtime = ChatRuntime::seeded();
        let session = runtime.load(SESSION_ID).await?;
        let fields = session_fields(&session);
        for name in [
            "id:",
            "conversation:",
            "usage:",
            "role:",
            "created:",
            "content:",
            "metadata:",
            "user_visible:",
            "agent_visible:",
            "inference:",
            "output_token_limit_reached:",
            "steer:",
            "turn_context:",
            "operations:",
        ] {
            assert!(fields.contains(name), "missing field {}", name);
        }
        assert!(fields.contains("inference: None"));
        assert!(fields.contains("output_token_limit_reached: false"));
        assert!(
            fields.ends_with("\n\n\n"),
            "two blank lines after raw fields"
        );
        Ok(())
    }
}
