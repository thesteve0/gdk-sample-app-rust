// Lesson 13
use super::*;
use goose_providers::{
    base::ThinkingPreservationFormat,
    conversation::message::{InferenceMetadata, Message, MessageContent, MessageUsage},
};
use rmcp::model::{
    Annotations, CallToolRequestParams, CallToolResult, ContentBlock, JsonObject, Role, TextContent,
};
use std::sync::Arc;

fn config(preserves_thinking: bool) -> anyhow::Result<DeclarativeProviderConfig> {
    let raw = json!({
        "name": "local", "engine": "openai", "display_name": "Local",
        "base_url": "http://127.0.0.1:8080", "requires_auth": false,
        "preserves_thinking": preserves_thinking,
        "models": [{"name": "other"}, {"name": "selected"}]
    });
    let config = serde_json::from_value(raw)?;
    Ok(config)
}

fn estimator() -> anyhow::Result<ContextSize> {
    let model = ModelConfig::new("selected");
    let config = config(false)?;
    Ok(ContextSize::new(&model, &config))
}

fn tool() -> Tool {
    let schema = JsonObject::from_iter([
        ("type".to_string(), json!("object")),
        (
            "properties".to_string(),
            json!({"note": {"type": "string"}}),
        ),
        ("required".to_string(), json!(["note"])),
        ("additionalProperties".to_string(), json!(false)),
    ]);
    Tool::new(
        "remember",
        "Propose a note, without saving it.",
        Arc::new(schema),
    )
}

#[test]
fn byte_ceiling_counts_utf8_and_json_escapes() -> anyhow::Result<()> {
    let estimator = estimator()?;
    for text in [
        "",
        "a",
        "aa",
        "aaa",
        "aaaa",
        "é",
        "日本語",
        "🪿",
        "\"\\\n\t",
    ] {
        let conversation = Conversation::new_unvalidated(vec![Message::user().with_text(text)]);
        // An independent expected content projection includes the static instruction.
        let expected = json!({"messages": [
            {"role": "system", "content": "fixed"},
            {"role": "user", "content": if text.is_empty() { "Hello" } else { text }}
        ], "tools": []});
        let bytes = serde_json::to_vec(&expected)?;
        assert_eq!(estimator.projection("fixed", &[], &conversation)?, expected);
        assert_eq!(
            estimator.estimate("fixed", &[], &conversation)?,
            bytes.len().div_ceil(4)
        );
    }
    let conversation = Conversation::new_unvalidated(vec![Message::user().with_text("same")]);
    let mut remainders = Vec::new();
    for width in 0..4 {
        let prompt = "p".repeat(width);
        let bytes = serde_json::to_vec(&estimator.projection(&prompt, &[], &conversation)?)?;
        remainders.push(bytes.len() % 4);
        assert_eq!(
            estimator.estimate(&prompt, &[], &conversation)?,
            bytes.len().div_ceil(4)
        );
    }
    remainders.sort();
    assert_eq!(remainders, vec![0, 1, 2, 3]);
    Ok(())
}

#[test]
fn selected_model_and_provider_control_thinking_replay() -> anyhow::Result<()> {
    let model = ModelConfig::new("selected");
    let conversation = Conversation::new_unvalidated(vec![
        Message::user().with_text("Question"),
        Message::assistant()
            .with_thinking("private reasoning".repeat(20), "")
            .with_text("Answer"),
        Message::user().with_text("Follow-up"),
    ]);
    let disabled_config = config(false)?;
    let disabled = ContextSize::new(&model, &disabled_config);
    let disabled_projection = disabled.projection("fixed", &[], &conversation)?;
    assert!(disabled_projection["messages"][2]
        .get("reasoning_content")
        .is_none());
    assert!(!serde_json::to_string(&disabled_projection)?.contains("private reasoning"));

    let enabled_config = config(true)?;
    let enabled = ContextSize::new(&model, &enabled_config);
    let enabled_projection = enabled.projection("fixed", &[], &conversation)?;
    assert_eq!(
        enabled_projection["messages"][2]["reasoning_content"],
        "private reasoning".repeat(20)
    );
    assert!(
        enabled.estimate("fixed", &[], &conversation)?
            > disabled.estimate("fixed", &[], &conversation)?
    );

    let mut format_config = config(false)?;
    format_config.models[0].thinking_preservation_format =
        Some(ThinkingPreservationFormat::ContentPrepend);
    // The first configured model is NOT the selected model.
    let other_format = ContextSize::new(&model, &format_config);
    assert_eq!(
        other_format.projection("fixed", &[], &conversation)?,
        disabled_projection
    );
    format_config.models[1].thinking_preservation_format =
        Some(ThinkingPreservationFormat::ContentXml);
    let explicit = ContextSize::new(&model, &format_config);
    let explicit_projection = explicit.projection("fixed", &[], &conversation)?;
    assert!(explicit.options.preserve_thinking_context);
    let text = match explicit_projection["messages"][2]["content"].as_str() {
        Some(text) => text,
        None => return Err(anyhow::anyhow!("expected assistant text")),
    };
    assert!(text.starts_with("<think>\nprivate reasoning"));
    assert!(explicit_projection["messages"][2]
        .get("reasoning_content")
        .is_none());
    let unknown = ContextSize::new(&ModelConfig::new("unlisted"), &format_config);
    assert!(!unknown.options.preserve_thinking_context);
    assert!(unknown.options.thinking_preservation_format.is_none());
    Ok(())
}

#[test]
fn application_metadata_ids_and_usage_do_not_inflate_estimate() -> anyhow::Result<()> {
    let estimator = estimator()?;
    let message = Message::user().with_text("Same actual content.");
    let plain = Conversation::new_unvalidated(vec![message.clone()]);
    let mut annotated = message.with_id("application-id".repeat(1000));
    annotated.created = i64::MAX;
    annotated.metadata.steer = true;
    annotated.metadata.turn_context = true;
    annotated.metadata.user_visible = false;
    annotated.metadata.inference = Some(InferenceMetadata {
        provider: "provider metadata".repeat(1000),
        requested_model: "requested".to_string(),
        resolved_model: Some("resolved".to_string()),
        provider_session_id: Some("session".repeat(1000)),
    });
    annotated.metadata.usage = Some(Box::new(MessageUsage {
        total_tokens: Some(i32::MAX),
        input_tokens: Some(i32::MAX),
        ..Default::default()
    }));
    annotated
        .metadata
        .set_operation_note("local", "data", json!("notes".repeat(1000)));
    let with_metadata = Conversation::new_unvalidated(vec![annotated]);
    assert_eq!(
        estimator.projection("fixed", &[], &plain)?,
        estimator.projection("fixed", &[], &with_metadata)?
    );
    assert_eq!(
        estimator.estimate("fixed", &[], &plain)?,
        estimator.estimate("fixed", &[], &with_metadata)?
    );
    Ok(())
}

#[test]
fn static_instructions_content_and_tool_definitions_all_count() -> anyhow::Result<()> {
    let estimator = estimator()?;
    let short = Conversation::new_unvalidated(vec![Message::user().with_text("Question")]);
    let long =
        Conversation::new_unvalidated(vec![Message::user().with_text("Question".repeat(100))]);
    let baseline = estimator.estimate("fixed", &[], &short)?;
    assert!(estimator.estimate(&"instructions".repeat(100), &[], &short)? > baseline);
    assert!(estimator.estimate("fixed", &[], &long)? > baseline);
    assert!(estimator.estimate("fixed", &[tool()], &short)? > baseline);
    let projection = estimator.projection("fixed", &[tool()], &short)?;
    assert_eq!(projection["tools"][0]["function"]["name"], "remember");
    assert_eq!(
        projection["tools"][0]["function"]["parameters"]["properties"]["note"]["type"],
        "string"
    );
    Ok(())
}

#[test]
fn request_options_and_model_identity_are_not_counted() -> anyhow::Result<()> {
    let config = config(false)?;
    let mut model = ModelConfig::new("selected");
    let ordinary = ContextSize::new(&model, &config);
    model.model_name = "long-model-name".repeat(1000);
    model.max_tokens = Some(999_999);
    model.temperature = Some(0.9);
    model.context_limit = Some(999_999);
    model.request_params = Some(std::collections::HashMap::from([(
        "custom_option".to_string(),
        json!("not context".repeat(1000)),
    )]));
    let modified = ContextSize::new(&model, &config);
    let conversation = Conversation::new_unvalidated(vec![Message::user().with_text("Question")]);
    let projection = modified.projection("fixed", &[], &conversation)?;
    assert_eq!(projection.as_object().expect("object").len(), 2);
    assert_eq!(
        ordinary.estimate("fixed", &[], &conversation)?,
        modified.estimate("fixed", &[], &conversation)?
    );
    Ok(())
}

#[test]
fn visible_projection_fixes_and_merges_without_mutating_inputs() -> anyhow::Result<()> {
    let estimator = estimator()?;
    let user_only = TextContent::new("user-only block".repeat(1000))
        .with_annotations(Annotations::default().with_audience(vec![Role::User]));
    let conversation = Conversation::new_unvalidated(vec![
        Message::user()
            .with_text("First")
            .with_content(MessageContent::Text(user_only)),
        Message::assistant()
            .with_text("hidden".repeat(1000))
            .with_visibility(true, false),
        Message::user().with_text("Second"),
        Message::assistant().with_text("Answer"),
        Message::user().with_text("Next"),
        // The request repair pass removes an orphan tool result from this clone.
        Message::user().with_tool_response(
            "orphan",
            Ok(CallToolResult::success(vec![ContentBlock::text(
                "orphan data",
            )])),
        ),
    ]);
    let tools = vec![tool()];
    let conversation_before = serde_json::to_value(&conversation)?;
    let tools_before = serde_json::to_value(&tools)?;
    let prompt = String::from("fixed instructions");
    let projected = estimator.projection(&prompt, &tools, &conversation)?;
    let serialized = serde_json::to_string(&projected)?;
    assert!(!serialized.contains("user-only block"));
    assert!(!serialized.contains("hidden"));
    assert!(!serialized.contains("orphan data"));
    assert_eq!(projected["messages"].as_array().expect("messages").len(), 4);
    assert_eq!(projected["messages"][1]["content"], "First\nSecond");
    let first = estimator.estimate(&prompt, &tools, &conversation)?;
    assert_eq!(estimator.estimate(&prompt, &tools, &conversation)?, first);
    assert_eq!(serde_json::to_value(&conversation)?, conversation_before);
    assert_eq!(serde_json::to_value(&tools)?, tools_before);
    assert_eq!(prompt, "fixed instructions");
    Ok(())
}

#[test]
fn tool_argument_json_escaping_is_measured_after_provider_formatting() -> anyhow::Result<()> {
    let estimator = estimator()?;
    let arguments =
        JsonObject::from_iter([("note".to_string(), json!("\"\\\n日本語".repeat(100)))]);
    let call = CallToolRequestParams::new("remember").with_arguments(arguments.clone());
    let conversation = Conversation::new_unvalidated(vec![
        Message::user().with_text("Question"),
        Message::assistant().with_tool_request("call-1", Ok(call)),
        Message::user().with_tool_response(
            "call-1",
            Ok(CallToolResult::success(vec![ContentBlock::text(
                "Proposed only.",
            )])),
        ),
        Message::assistant().with_text("Answer"),
    ]);
    let projection = estimator.projection("fixed", &[tool()], &conversation)?;
    assert_eq!(
        projection["messages"][2]["tool_calls"][0]["function"]["arguments"],
        serde_json::to_string(&arguments)?
    );
    let bytes = serde_json::to_vec(&projection)?;
    assert_eq!(
        estimator.estimate("fixed", &[tool()], &conversation)?,
        bytes.len().div_ceil(4)
    );
    Ok(())
}

#[test]
fn vision_uses_actual_model_setting_and_model_is_cloned() -> anyhow::Result<()> {
    let config = config(false)?;
    let mut model = ModelConfig::new("selected");
    model.supports_vision = Some(true);
    let estimator = ContextSize::new(&model, &config);
    model.supports_vision = Some(false);
    model.model_name = "changed".to_string();
    assert!(estimator.options.supports_vision);
    assert_eq!(estimator.model.model_name, "selected");
    assert_eq!(estimator.model.supports_vision, Some(true));
    let disabled = ContextSize::new(&model, &config);
    assert!(!disabled.options.supports_vision);
    let default = ContextSize::new(&ModelConfig::new("selected"), &config);
    assert!(!default.options.supports_vision);
    Ok(())
}

#[test]
fn status_is_informational_and_formatter_errors_are_nonfatal() -> anyhow::Result<()> {
    let estimator = estimator()?;
    let conversation = Conversation::new_unvalidated(vec![Message::user().with_text("Question")]);
    let estimate = estimator.estimate("fixed", &[], &conversation)?;
    assert_eq!(
        estimator.status("fixed", &[], &conversation),
        format!("Context: ~{} tokens (estimate)", estimate)
    );
    let unsupported = ContextSize::new(&ModelConfig::new("o1-mini"), &config(false)?);
    assert!(unsupported.estimate("fixed", &[], &conversation).is_err());
    assert_eq!(
        unsupported.status("fixed", &[], &conversation),
        "Context: token estimate unavailable"
    );
    Ok(())
}

#[test]
fn completed_answer_is_counted_without_inventing_the_next_question() -> anyhow::Result<()> {
    let estimator = estimator()?;
    let short = Conversation::new_unvalidated(vec![
        Message::user().with_text("Question"),
        Message::assistant().with_text("Answer"),
    ]);
    let long = Conversation::new_unvalidated(vec![
        Message::user().with_text("Question"),
        Message::assistant().with_text("Answer".repeat(1000)),
    ]);
    let projection = estimator.projection("fixed", &[], &short)?;
    assert_eq!(
        projection["messages"].as_array().expect("messages").len(),
        3
    );
    assert_eq!(projection["messages"][2]["content"], "Answer");
    assert!(!serde_json::to_string(&projection)?.contains("context sizing boundary"));
    assert!(estimator.estimate("fixed", &[], &long)? > estimator.estimate("fixed", &[], &short)?);
    Ok(())
}
