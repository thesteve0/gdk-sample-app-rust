// Lesson 14
use super::*;
#[test]
fn storage_names_cannot_escape_the_disposable_directory() {
    for name in ["", "../learner-record", "/tmp/a", "a/b", ".", "a\\b"] {
        assert!(validate_storage_name(name).is_err());
    }
    assert!(validate_storage_name("baseline-case08").is_ok());
}
#[test]
fn observable_projection_keeps_dialogue_without_message_metadata() {
    let message =
        goose_providers::conversation::message::Message::user().with_text("actual answer");
    let text = observable(&[message]);
    assert!(text.contains("actual answer"));
    assert!(!text.contains("metadata"));
}
