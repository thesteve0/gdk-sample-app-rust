// Lesson 14
use super::*;
use std::{
    env,
    sync::atomic::{AtomicUsize, Ordering},
};
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
    let directory = test_directory("record")?;
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

// Every storage test supplies its own path; never read the application's record.
fn test_directory(label: &str) -> anyhow::Result<PathBuf> {
    static NEXT_DIRECTORY: AtomicUsize = AtomicUsize::new(0);
    let sequence = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
    let directory = env::temp_dir().join(format!(
        "gdk-lesson14-learning-{}-{}-{}",
        label,
        std::process::id(),
        sequence
    ));
    fs::create_dir(&directory)?;
    Ok(directory)
}

#[test]
fn learned_parser_and_key_are_bounded_plain_and_not_mastery() {
    assert_eq!(
        parse_learned("/learned Stop orders").unwrap(),
        "Stop orders"
    );
    assert_eq!(completion_key(" STOP  Orders "), "stop orders");
    assert_eq!(
        parse_learned(&format!("/learned {}", "a".repeat(200)))
            .unwrap()
            .len(),
        200
    );
    for raw in [
        "/learned",
        "/learned ",
        "/learnedness abc",
        "/learned a\nb",
        "/learned a\t",
        "/learned `instructions`",
        "/learned \x1b[31m",
        "/learned # heading",
    ] {
        assert!(parse_learned(raw).is_err(), "{}", raw);
    }
    assert!(parse_learned(&format!("/learned {}", "é".repeat(101))).is_err());
}

#[test]
fn learned_parser_preserves_concept_text_and_uses_utf8_bytes() {
    assert_eq!(
        parse_learned("  /learned  Trader's stop/limit (orders): 1.0, 2-3  ").unwrap(),
        "Trader's stop/limit (orders): 1.0, 2-3"
    );
    let unicode_concept = "é".repeat(100);
    assert_eq!(
        parse_learned(&format!("/learned {}", unicode_concept)).unwrap(),
        unicode_concept
    );
    for raw in [
        "/Learned orders",
        "/learned orders\r",
        "/learned orders\0",
        "/learned a\u{202e}b",
        "/learned [orders]",
        "/learned 🪿",
    ] {
        assert!(parse_learned(raw).is_err(), "{:?}", raw);
    }
    assert_eq!(completion_key(" ÉTÉ  Orders "), "été orders");
}

#[test]
fn completion_append_preserves_manual_bytes_permissions_and_declaration_wording(
) -> anyhow::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let directory = test_directory("completion")?;
    let path = directory.join("explicit-record.md");
    let concept = parse_learned("/learned Stop orders").expect("valid declaration");
    let saved = append_completion(&path, &concept)?;
    assert!(saved.starts_with("# Learner learning record\n\n"));
    assert!(saved
        .contains("Learner-approved notes and explicit declarations, not a mastery certificate."));
    assert!(saved.contains("## Learner-declared completed concepts"));
    assert!(saved.contains("- Stop orders (learner declaration; not assessed mastery)\n"));
    assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o600);
    assert_eq!(read_record(&path)?, saved);

    let hand_edit = "# Hand-edited notes\n\nKeep every byte.\n";
    fs::write(&path, hand_edit)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640))?;
    let updated = append_completion(&path, &concept)?;
    assert!(updated.starts_with(hand_edit));
    assert!(!updated.contains("# Learner learning record"));
    assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o640);

    // Duplicate suppression belongs to the caller's Session ledger, not storage.
    let repeated = append_completion(&path, &concept)?;
    assert!(repeated.starts_with(&updated));
    assert_eq!(repeated.matches("- Stop orders (").count(), 2);
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn completion_append_enforces_byte_bound_without_rewriting_on_failure() -> anyhow::Result<()> {
    let directory = test_directory("completion-bound")?;
    let path = directory.join("explicit-record.md");
    let concept = "Stop orders";
    let addition = format!("\n## Learner-declared completed concepts\n\n- {} (learner declaration; not assessed mastery)\n", concept);
    let existing = "x".repeat(MAX_RECORD_BYTES - addition.len());
    fs::write(&path, &existing)?;
    let updated = append_completion(&path, concept)?;
    assert_eq!(updated.len(), MAX_RECORD_BYTES);
    assert!(updated.starts_with(&existing));
    assert!(append_completion(&path, concept).is_err());
    assert_eq!(fs::read_to_string(&path)?, updated);

    fs::write(&path, "x".repeat(MAX_RECORD_BYTES + 1))?;
    assert!(append_completion(&path, concept).is_err());
    assert_eq!(fs::metadata(&path)?.len(), (MAX_RECORD_BYTES + 1) as u64);
    fs::write(&path, [0xff])?;
    assert!(append_completion(&path, concept).is_err());
    assert_eq!(fs::read(&path)?, vec![0xff]);
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn completion_append_rejects_symlinks_nonregular_and_missing_parent_paths() -> anyhow::Result<()> {
    use std::os::unix::fs::symlink;
    let directory = test_directory("completion-path")?;
    let path = directory.join("explicit-record.md");
    let hand_edit = "Untouched manual notes.\n";
    fs::write(&path, hand_edit)?;
    let link = directory.join("link.md");
    symlink(&path, &link)?;
    assert!(append_completion(&link, "Stop orders").is_err());
    assert_eq!(fs::read_to_string(&path)?, hand_edit);
    assert!(append_completion(&directory, "Stop orders").is_err());
    let missing_parent = directory.join("missing").join("record.md");
    assert!(append_completion(&missing_parent, "Stop orders").is_err());
    assert!(!missing_parent.exists());
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[test]
fn schema_and_categories_preserve_proposal_only_contract() -> anyhow::Result<()> {
    let definition = record_tool_definition();
    let encoded = serde_json::to_value(&definition)?;
    assert_eq!(encoded["name"], RECORD_TOOL);
    let schema = &encoded["inputSchema"];
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(schema["properties"]["note"]["maxLength"], 600);
    assert_eq!(schema["properties"]["learner_quote"]["maxLength"], 600);
    assert_eq!(
        schema["required"],
        json!(["category", "note", "learner_quote"])
    );
    assert!(schema["properties"].get("path").is_none());
    for (category, heading) in [
        ("discussed_topics", "Discussed topics"),
        ("stated_goals", "Learner-stated goals"),
        (
            "learner_evidence",
            "Evidence from learner answers (not mastery)",
        ),
        ("unresolved_questions", "Unresolved questions"),
        ("agreed_activities", "Learner-agreed next activities"),
    ] {
        let proposal: RecordProposal = serde_json::from_value(json!({
            "category": category,
            "note": "My words",
            "learner_quote": "My words"
        }))?;
        assert_eq!(proposal.category.heading(), heading);
    }
    for invalid in [
        json!({"category": "mastery", "note": "A", "learner_quote": "A"}),
        json!({"category": "stated_goals", "note": "A"}),
        json!({"category": "stated_goals", "note": "A", "learner_quote": "A", "path": "other.md"}),
    ] {
        assert!(serde_json::from_value::<RecordProposal>(invalid).is_err());
    }
    Ok(())
}

#[test]
fn markdown_escaping_is_shared_by_note_and_declaration_storage() -> anyhow::Result<()> {
    let raw = "\\`*_{}[]<>#!|";
    let expected = "\\\\\\`\\*\\_\\{\\}\\[\\]\\<\\>\\#\\!\\|";
    assert_eq!(markdown_note(raw), expected);
    let directory = test_directory("markdown")?;
    let path = directory.join("explicit-record.md");
    // Parsing is a separate caller boundary; storage escapes supplied text verbatim.
    let saved = append_completion(&path, raw)?;
    assert!(saved.contains(&format!(
        "- {} (learner declaration; not assessed mastery)",
        expected
    )));
    fs::remove_dir_all(directory)?;
    Ok(())
}
