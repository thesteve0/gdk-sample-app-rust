// Lesson 13
use super::*;
use crate::provider_setup::DEFAULT_PROVIDER_CONFIG;

async fn history(count: usize, width: usize) -> anyhow::Result<ChatSession> {
    let runtime = ChatRuntime::seeded();
    let mut state = runtime.load(SESSION_ID).await?;
    state.conversation.push(
        Message::assistant()
            .with_text("Welcome.")
            .with_generated_id_if_missing(),
    );
    for index in 0..count {
        let input = Message::user()
            .with_text(format!("Genuine reply {}", index))
            .with_generated_id_if_missing();
        state
            .genuine_reply_ids
            .push(input.id.clone().expect("fixture ID"));
        state.conversation.push(input);
        state.conversation.push(
            Message::assistant()
                .with_text(format!("Explanation {} {}", index, "x".repeat(width)))
                .with_generated_id_if_missing(),
        );
    }
    Ok(state)
}

fn report() -> Report {
    Report {
        reason: "fixture".to_string(),
        counting_method: "bytes".to_string(),
        before_bytes: 20000,
        after_bytes: 10000,
        covered_message_ids: Vec::new(),
        summary: Some("Fallible fixture".to_string()),
        summary_usage: Vec::new(),
        summary_partial_messages: Vec::new(),
        disposition: "applied: fixture".to_string(),
    }
}

#[test]
fn projection_separates_speakers_and_requires_genuine_identity() {
    let learner = Message::user()
        .with_text("I am unsure.")
        .with_generated_id_if_missing();
    let tutor = Message::assistant()
        .with_text("That is resolved.")
        .with_generated_id_if_missing();
    let opening = Message::user()
        .with_text("Begin lesson.")
        .with_generated_id_if_missing();
    let mut prior = Message::assistant()
        .with_text("Earlier uncertain account.")
        .with_generated_id_if_missing();
    prior
        .metadata
        .set_operation_note("compaction", "derived_summary", json!(true));
    let ids = vec![
        learner.id.clone().expect("fixture"),
        tutor.id.clone().expect("fixture"),
        prior.id.clone().expect("fixture"),
    ];
    let source = vec![learner, tutor, opening, prior];
    let projected = summary_source(&source, &ids);
    assert_eq!(projected.len(), source.len());
    for (record, original) in projected.iter().zip(&source) {
        assert_eq!(record["original_message_id"], json!(original.id));
        assert_eq!(record["speaker"], json!(original.role));
        assert_eq!(record["content"][0]["text"], original.as_concat_text());
    }
    assert_eq!(projected[0]["provenance"], "genuine learner reply");
    assert_eq!(
        projected[1]["provenance"],
        "tutor ordinary text (NOT learner evidence)"
    );
    assert_eq!(
        projected[2]["provenance"],
        "application or tool history (NOT learner evidence)"
    );
    assert_eq!(
        projected[3]["provenance"],
        "prior derived summary (fallible; NOT direct learner evidence)"
    );
    assert_eq!(
        summary_source(&source[..1], &[])[0]["provenance"],
        projected[2]["provenance"]
    );
}

#[test]
fn projection_omits_reasoning_and_metadata_without_changing_originals() -> anyhow::Result<()> {
    use goose_providers::conversation::message::{MessageContent, RedactedThinkingContentBlock};
    let mut tutor = Message::assistant()
        .with_thinking("private reasoning", "signature")
        .with_text("Ordinary explanation.");
    tutor.content.push(MessageContent::RedactedThinking(
        RedactedThinkingContentBlock {
            data: "opaque reasoning".to_string(),
        },
    ));
    tutor
        .metadata
        .set_operation_note("fixture", "internal", json!("private metadata"));
    let source = vec![tutor];
    let before = serde_json::to_value(&source)?;
    let projected = summary_source(&source, &[]);
    assert_eq!(
        projected[0],
        json!({
            "original_message_id": source[0].id,
            "speaker": source[0].role,
            "provenance": "tutor ordinary text (NOT learner evidence)",
            "content": [{"provenance": "tutor ordinary text (NOT learner evidence)", "text": "Ordinary explanation."}],
        })
    );
    assert_eq!(serde_json::to_value(&source)?, before);
    assert_eq!(source[0].content.len(), 3);
    Ok(())
}

#[test]
fn projection_preserves_actual_correlated_tool_data_and_qualifications() -> anyhow::Result<()> {
    let call = CallToolRequestParams::new(TOOL_NAME).with_arguments(serde_json::from_value(
        json!({"entry_price":"51.20", "stop_price":"50.70", "share_count":200}),
    )?);
    let result = CallToolResult::success(vec![ContentBlock::text(
        "100.00; excludes fees, slippage and gaps. Draft source, dated, clipped; partial coverage.",
    )]);
    let source = vec![
        Message::assistant()
            .with_text("Historical calculation.")
            .with_tool_request("actual-call", Ok(call.clone())),
        Message::user()
            .with_tool_response("actual-call", Ok(result.clone()))
            .with_generated_id_if_missing(),
    ];
    let before = serde_json::to_value(&source)?;
    // Even an erroneously listed user-role tool response cannot become learner evidence.
    let ids = vec![source[1].id.clone().expect("fixture")];
    let projected = summary_source(&source, &ids);
    assert_eq!(projected[0]["content"][1]["correlation_id"], "actual-call");
    assert_eq!(projected[1]["content"][0]["correlation_id"], "actual-call");
    assert_eq!(
        projected[0]["content"][1]["tool_call"],
        serde_json::to_value(Ok::<_, ErrorData>(call))?
    );
    assert_eq!(
        projected[1]["content"][0]["tool_result"],
        serde_json::to_value(Ok::<_, ErrorData>(result))?
    );
    assert_eq!(
        projected[1]["provenance"],
        "application or tool history (NOT learner evidence)"
    );
    assert_eq!(serde_json::to_value(&source)?, before);
    Ok(())
}

#[test]
fn summary_guard_counts_projected_request_and_boundary() -> anyhow::Result<()> {
    let source = vec![Message::assistant()
        .with_thinking("x".repeat(SUMMARY_INPUT_BYTES * 2), "")
        .with_text("Ordinary.")];
    let (prompt, request) = summary_request(&source, &[], "editable guidance")?;
    assert_eq!(
        prompt,
        format!("{}{}", SUMMARY_SOURCE_BOUNDARY, "editable guidance")
    );
    let text = request.messages()[0].as_concat_text();
    let (_, data) = text.split_once('\n').expect("source envelope");
    assert_eq!(
        serde_json::from_str::<Value>(data)?,
        json!(summary_source(&source, &[]))
    );
    let measured = payload_bytes(&prompt, &[], &request)?;
    let mut model = ModelConfig::new("local-tutor");
    model.max_tokens = Some(4_096);
    let wire = goose_providers::formats::openai::create_request_with_options(
        &model,
        &prompt,
        request.messages(),
        &[],
        &goose_providers::images::ImageFormat::OpenAi,
        true,
        goose_providers::formats::openai::OpenAiFormatOptions::default(),
    )?;
    assert!(measured >= serde_json::to_vec(&wire)?.len());
    let boundary_guidance = "g".repeat(SUMMARY_INPUT_BYTES - measured + "editable guidance".len());
    let (at_prompt, at_request) = summary_request(&source, &[], &boundary_guidance)?;
    assert_eq!(
        payload_bytes(&at_prompt, &[], &at_request)?,
        SUMMARY_INPUT_BYTES
    );
    assert!(summary_request(&source, &[], &format!("{}g", boundary_guidance)).is_err());
    assert!(summary_request(
        &[Message::user().with_text("x".repeat(SUMMARY_INPUT_BYTES))],
        &[],
        ""
    )
    .is_err());
    Ok(())
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
    assert_eq!(reply_action("/learned Stop orders"), ReplyAction::Learned);
    assert_eq!(reply_action("/learned"), ReplyAction::Learned);
}

#[test]
fn summary_token_truncation_and_byte_boundary_are_invalid() {
    let mut message = Message::assistant().with_text("valid text");
    assert!(valid_summary_output(&[message.clone()], "valid text"));
    message.metadata.output_token_limit_reached = true;
    assert!(!valid_summary_output(&[message], "valid text"));
    assert!(valid_summary_output(
        &[Message::assistant().with_text("x".repeat(SUMMARY_BYTES))],
        &"x".repeat(SUMMARY_BYTES)
    ));
    assert!(!valid_summary_output(
        &[Message::assistant().with_text("x".repeat(SUMMARY_BYTES + 1))],
        &"x".repeat(SUMMARY_BYTES + 1)
    ));
}

#[test]
fn provider_thinking_is_metadata_not_summary_data_or_a_tool() {
    let message = Message::assistant()
        .with_thinking("provider reasoning", "")
        .with_text("Derived summary.");
    assert_eq!(message.as_concat_text(), "Derived summary.");
    assert!(valid_summary_output(&[message], "Derived summary."));
    let tool = Message::assistant()
        .with_tool_request("unexpected", Ok(CallToolRequestParams::new(RECORD_TOOL)));
    assert!(!valid_summary_output(&[tool], "Derived summary."));
    assert!(!valid_summary_output(
        &[Message::assistant().with_thinking("reasoning only", "")],
        ""
    ));
}

#[test]
fn conservative_estimate_covers_pinned_openai_arguments_without_reserializing_all_text(
) -> anyhow::Result<()> {
    use goose_providers::formats::openai::{create_request_with_options, OpenAiFormatOptions};
    use goose_providers::images::ImageFormat;
    let args = json!({"note": "\"\\\n".repeat(4000)});
    let messages = vec![
        Message::user().with_text("{\"already_json\":\"\\\\\"}".repeat(2000)),
        Message::assistant().with_tool_request(
            "call",
            Ok(CallToolRequestParams::new(RECORD_TOOL)
                .with_arguments(args.as_object().expect("object").clone())),
        ),
    ];
    let conversation = Conversation::new_unvalidated(messages.clone());
    let estimate = payload_bytes("fixed prompt", &[record_tool_definition()], &conversation)?;
    let mut model = ModelConfig::new("local-tutor");
    model.max_tokens = Some(TUTOR_MAX_TOKENS);
    let wire = create_request_with_options(
        &model,
        "fixed prompt",
        &messages,
        &[record_tool_definition()],
        &ImageFormat::OpenAi,
        true,
        OpenAiFormatOptions::default(),
    )?;
    assert!(estimate >= serde_json::to_vec(&wire)?.len());
    let raw = Conversation::new_unvalidated(vec![messages[0].clone()]);
    let native = serde_json::to_vec(
        &json!({"system_prompt":"fixed prompt", "tools":[], "messages":raw.messages()}),
    )?
    .len();
    assert_eq!(
        payload_bytes("fixed prompt", &[], &raw)?,
        native + 4096 + 512 + 512
    );
    Ok(())
}

#[tokio::test]
async fn prospective_candidate_and_preparer_share_the_same_estimate_boundary() -> anyhow::Result<()>
{
    let mut state = history(2, 100).await?;
    let prospective = prospective_bytes(&state, "same input")?;
    let new = Message::user()
        .with_text("same input")
        .with_generated_id_if_missing();
    state.conversation.push(new);
    assert_eq!(prospective, tutor_payload(&state)?);
    let measured = tutor_payload(&state)?;
    let input = || InferenceInput {
        prompt_parts: state.prompt_parts(),
        tools: vec![tool_definition(), retrieval::tool_definition()],
        moim_parts: Vec::new(),
    };
    let mut at_boundary = state.clone();
    at_boundary.policy.total_bytes = measured + at_boundary.policy.headroom_bytes;
    assert!(check_budget(&at_boundary).is_ok());
    assert!(BudgetPreparer
        .prepare(&at_boundary, &at_boundary.conversation, input())
        .await
        .is_ok());
    at_boundary.policy.total_bytes -= 1;
    assert!(check_budget(&at_boundary).is_err());
    assert!(BudgetPreparer
        .prepare(&at_boundary, &at_boundary.conversation, input())
        .await
        .is_err());
    Ok(())
}

#[tokio::test]
async fn local_declaration_saves_once_without_tutor_turn_and_survives_maintenance_failure(
) -> anyhow::Result<()> {
    let directory = env::temp_dir().join(format!("l13-completion-{}", std::process::id()));
    fs::create_dir_all(&directory)?;
    let path = directory.join("record.md");
    fs::write(&path, "Hand edited original bytes\n")?;
    let mut log = DiagnosticLog::create_at(directory.join("diagnostics.md"))?;
    let runtime = ChatRuntime::seeded();
    {
        let mut sessions = runtime.sessions.lock().await;
        sessions
            .get_mut(SESSION_ID)
            .expect("fixture")
            .lesson
            .persistence_enabled = true;
    }
    let before = runtime.load(SESSION_ID).await?;
    declare(&runtime, SESSION_ID, "Stop orders", &mut log, &path).await?;
    let once = fs::read_to_string(&path)?;
    assert!(once.starts_with("Hand edited original bytes\n"));
    assert!(once.contains("## Learner-declared completed concepts"));
    assert!(once.contains("not assessed mastery"));
    let mut failed = Outcome {
        revision: runtime.load(SESSION_ID).await?.revision,
        replacement: None,
        report: report(),
    };
    failed.report.disposition = "summary timeout; history retained".to_string();
    {
        let mut sessions = runtime.sessions.lock().await;
        apply_outcome(sessions.get_mut(SESSION_ID).expect("fixture"), &mut failed)?;
    }
    declare(&runtime, SESSION_ID, "STOP  orders", &mut log, &path).await?;
    assert_eq!(fs::read_to_string(&path)?, once);
    let after = runtime.load(SESSION_ID).await?;
    assert_eq!(before.id, after.id);
    assert_eq!(after.applied_passes, before.applied_passes);
    assert_eq!(after.genuine_reply_ids, before.genuine_reply_ids);
    assert!(after.usage.is_empty());
    assert_eq!(after.completed_concepts.len(), 1);
    let diagnostics = fs::read_to_string(directory.join("diagnostics.md"))?;
    let intent = diagnostics.find("intent BEFORE").expect("intent");
    let saved = diagnostics.find("disposition AFTER").expect("success");
    assert!(intent < saved);
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[tokio::test]
async fn declaration_without_consent_is_ephemeral_and_never_quote_evidence() -> anyhow::Result<()> {
    let directory = env::temp_dir().join(format!("l13-ephemeral-{}", std::process::id()));
    fs::create_dir_all(&directory)?;
    let path = directory.join("never-written.md");
    let mut log = DiagnosticLog::create_at(directory.join("diagnostics.md"))?;
    let runtime = ChatRuntime::seeded();
    declare(&runtime, SESSION_ID, "Orders", &mut log, &path).await?;
    assert!(!path.exists());
    let state = runtime.load(SESSION_ID).await?;
    assert_eq!(
        state.completed_concepts["orders"],
        "ephemeral (memory consent disabled)"
    );
    let call = CallToolRequestParams::new(RECORD_TOOL).with_arguments(serde_json::from_value(
        json!({"category":"learner_evidence","note":"Orders completed", "learner_quote":"Orders"}),
    )?);
    assert!(parse_record_proposal(&call, &state).is_err());
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[tokio::test]
async fn storage_failure_does_not_mark_saved_and_is_fatal() -> anyhow::Result<()> {
    let directory = env::temp_dir().join(format!("l13-failure-{}", std::process::id()));
    fs::create_dir_all(&directory)?;
    let mut log = DiagnosticLog::create_at(directory.join("diagnostics.md"))?;
    let runtime = ChatRuntime::seeded();
    {
        let mut sessions = runtime.sessions.lock().await;
        sessions
            .get_mut(SESSION_ID)
            .expect("fixture")
            .lesson
            .persistence_enabled = true;
    }
    assert!(
        declare(&runtime, SESSION_ID, "Orders", &mut log, &directory)
            .await
            .is_err()
    );
    assert!(runtime
        .load(SESSION_ID)
        .await?
        .completed_concepts
        .is_empty());
    fs::remove_dir_all(directory)?;
    Ok(())
}

#[tokio::test]
async fn partition_keeps_opening_two_genuine_exchanges_pairs_and_metadata() -> anyhow::Result<()> {
    let mut state = history(4, 5000).await?;
    let request = Message::assistant()
        .with_tool_request("pair-id", Ok(CallToolRequestParams::new(TOOL_NAME)))
        .with_generated_id_if_missing();
    let mut response = Message::user()
        .with_tool_response(
            "pair-id",
            Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
        )
        .with_generated_id_if_missing();
    response
        .metadata
        .set_operation_note("fixture", "untouched", json!({"source":"retained"}));
    // Insert the tool pair into the last exchange before its final answer.
    let mut messages = state.conversation.messages().clone();
    let end = messages.len() - 1;
    messages.insert(end, request);
    messages.insert(end + 1, response.clone());
    state.conversation = Conversation::new_unvalidated(messages);
    let parts = partition(&state)?.expect("eligible");
    assert_eq!(parts.opening.len(), 2);
    assert_eq!(parts.opening[0].id, state.conversation.messages()[0].id);
    assert_eq!(parts.eligible.len(), 4);
    assert_eq!(parts.recent.len(), 6);
    assert_eq!(parts.recent[0].as_concat_text(), "Genuine reply 2");
    assert!(complete_exchange(&parts.recent[2..]));
    let replacement = candidate(parts, "Learner discussed orders; not mastery.");
    assert_eq!(
        replacement.messages()[0].id,
        state.conversation.messages()[0].id
    );
    let kept = replacement
        .messages()
        .iter()
        .find(|message| message.id == response.id)
        .expect("response retained");
    assert_eq!(serde_json::to_value(kept)?, serde_json::to_value(response)?);
    assert_eq!(
        messages_since_kickoff(&replacement)?[0].as_concat_text(),
        "Genuine reply 3"
    );
    assert_eq!(
        replacement
            .messages()
            .iter()
            .filter(|message| is_summary(message))
            .count(),
        1
    );
    Ok(())
}

#[tokio::test]
async fn unsafe_pair_active_exchange_or_pending_approval_never_compacts() -> anyhow::Result<()> {
    let mut state = history(4, 5000).await?;
    state.conversation.push(
        Message::assistant()
            .with_tool_request("pending", Ok(CallToolRequestParams::new(TOOL_NAME))),
    );
    assert!(partition(&state).is_err());
    let mut state = history(4, 5000).await?;
    let mut messages = state.conversation.messages().clone();
    messages.insert(
        3,
        Message::assistant()
            .with_tool_request("approval", Ok(CallToolRequestParams::new(RECORD_TOOL))),
    );
    messages.insert(
        4,
        Message::user().with_tool_response(
            "approval",
            Ok(CallToolResult::success(vec![ContentBlock::text(
                "Proposal prepared",
            )])),
        ),
    );
    state.conversation = Conversation::new_unvalidated(messages);
    assert!(partition(&state).is_err());
    state
        .settled_approvals
        .insert("approval".to_string(), false);
    assert!(partition(&state)?.is_some());
    state.settled_approvals.insert("approval".to_string(), true);
    assert!(partition(&state)?.is_some());
    Ok(())
}

#[tokio::test]
async fn replacement_checks_revision_retains_authoritative_state_and_real_quotes_only(
) -> anyhow::Result<()> {
    let mut state = history(4, 5000).await?;
    state
        .completed_concepts
        .insert("orders".to_string(), "saved".to_string());
    state.settled_approvals.insert("accepted".to_string(), true);
    state
        .settled_approvals
        .insert("rejected".to_string(), false);
    state.lesson.persistence_enabled = true;
    state.lesson.learner_record = "Hand-edited approved memory".to_string();
    state.applied_passes = 7;
    let original = serde_json::to_value(state.conversation.messages())?;
    let replacement = candidate(
        partition(&state)?.expect("eligible"),
        "Genuine reply 0. Invented quote.",
    );
    let mut stale = Outcome {
        revision: state.revision + 1,
        replacement: Some(replacement.clone()),
        report: report(),
    };
    apply_outcome(&mut state, &mut stale)?;
    assert_eq!(
        serde_json::to_value(state.conversation.messages())?,
        original
    );
    let mut outcome = Outcome {
        revision: state.revision,
        replacement: Some(replacement),
        report: report(),
    };
    apply_outcome(&mut state, &mut outcome)?;
    assert_eq!(state.id, SESSION_ID);
    assert_eq!(state.applied_passes, 7);
    assert_eq!(state.genuine_reply_ids.len(), 2);
    assert_eq!(state.completed_concepts["orders"], "saved");
    assert!(state.settled_approvals["accepted"]);
    assert!(!state.settled_approvals["rejected"]);
    assert_eq!(state.lesson.learner_record, "Hand-edited approved memory");
    let proposal = |quote: &str| {
        CallToolRequestParams::new(RECORD_TOOL).with_arguments(serde_json::from_value(json!({"category":"learner_evidence","note":"limited evidence", "learner_quote":quote})).expect("fixture"))
    };
    assert!(parse_record_proposal(&proposal("Genuine reply 0"), &state).is_err());
    assert!(parse_record_proposal(&proposal("Invented quote"), &state).is_err());
    assert!(parse_record_proposal(&proposal("Genuine reply 3"), &state).is_ok());
    assert!(parse_record_proposal(&proposal(OPENING_INPUT), &state).is_err());
    let parts = partition(&state)?;
    assert!(parts.is_none());
    Ok(())
}

#[tokio::test]
async fn prior_summary_is_merged_once_not_promoted_into_prompt_or_quote_provenance(
) -> anyhow::Result<()> {
    let mut state = history(6, 5000).await?;
    let mut old = Message::assistant()
        .with_text("Old derived data")
        .with_generated_id_if_missing();
    old.metadata
        .set_operation_note("compaction", "derived_summary", json!(true));
    let mut messages = state.conversation.messages().clone();
    messages.insert(2, old.clone());
    state.conversation = Conversation::new_unvalidated(messages);
    let parts = partition(&state)?.expect("eligible");
    assert_eq!(
        parts
            .eligible
            .iter()
            .filter(|message| is_summary(message))
            .count(),
        1
    );
    let new = candidate(parts, "Merged fallible summary");
    assert_eq!(
        new.messages()
            .iter()
            .filter(|message| is_summary(message))
            .count(),
        1
    );
    assert!(!new.messages().iter().any(|message| message.id == old.id));
    assert!(!state
        .lesson
        .prompt_parts()
        .iter()
        .any(|(_name, part)| part.contains("Old derived data")));
    Ok(())
}

#[tokio::test]
async fn full_byte_budget_counts_static_tools_metadata_and_tool_results() -> anyhow::Result<()> {
    let mut state = history(2, 100).await?;
    let base = tutor_payload(&state)?;
    state.lesson.material.push_str(&"x".repeat(90_000));
    assert!(check_budget(&state).is_err());
    let mut state = history(2, 100).await?;
    assert!(prospective_bytes(&state, "ordinary input")? > base);
    state.conversation.push(Message::user().with_tool_response(
        "oversized",
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "x".repeat(90_000),
        )])),
    ));
    let input = InferenceInput {
        prompt_parts: state.lesson.prompt_parts(),
        tools: vec![tool_definition(), retrieval::tool_definition()],
        moim_parts: Vec::new(),
    };
    assert!(BudgetPreparer
        .prepare(&state, &state.conversation, input)
        .await
        .is_err());
    assert!(partition(&state).is_err());
    assert_eq!(Policy::default().trigger_bytes, 60000);
    assert_eq!(Policy::default().total_bytes, 100000);
    assert_eq!(Policy::default().headroom_bytes, 16000);
    Ok(())
}

#[tokio::test]
async fn short_history_is_noop_and_append_continues_same_session() -> anyhow::Result<()> {
    let state = history(2, 5000).await?;
    assert!(partition(&state)?.is_none());
    let runtime = ChatRuntime {
        sessions: Mutex::new(HashMap::from([(SESSION_ID.to_string(), state)])),
    };
    runtime
        .append_user(SESSION_ID, "Follow-up about stop orders")
        .await?;
    let after = runtime.load(SESSION_ID).await?;
    assert_eq!(after.id, SESSION_ID);
    assert_eq!(
        messages_since_kickoff(&after.conversation)?[0].as_concat_text(),
        "Follow-up about stop orders"
    );
    assert_eq!(after.genuine_reply_ids.len(), 3);
    assert_eq!(after.applied_passes, 0);
    Ok(())
}

#[tokio::test]
async fn summary_machine_yields_after_single_effect_batch_without_tools() -> anyhow::Result<()> {
    let operation = SummaryStop {
        guidance: "Summarize.".to_string(),
    };
    let mut session = SummarySession {
        conversation: Conversation::new_unvalidated(vec![
            Message::user().with_text("untrusted source")
        ]),
        usage: Vec::new(),
        batches: 0,
    };
    let (sender, _receiver) = mpsc::channel(4);
    let emit = Emitter::new(sender, CancellationToken::new());
    assert!(operation.inference_tools(&session).await?.is_empty());
    assert!(matches!(
        operation
            .run(&session, &session.conversation, &emit)
            .await?,
        OperationResult::NotApplicable
    ));
    session.batches = 1;
    let result = operation
        .run(&session, &session.conversation, &emit)
        .await?;
    let result = match result {
        OperationResult::Applied(result) => result,
        _ => panic!("yield"),
    };
    assert!(result.yield_to_client);
    assert!(result.effects.is_empty());
    Ok(())
}
#[tokio::test]
async fn cancelled_maintenance_retains_every_message_and_yields_without_summary_inference(
) -> anyhow::Result<()> {
    let mut state = history(4, 5000).await?;
    state.maintenance = Some("fixture cancellation".to_string());
    let original = serde_json::to_value(state.conversation.messages())?;
    let config = fs::read_to_string(PathBuf::from(REPOSITORY_ROOT).join(DEFAULT_PROVIDER_CONFIG))?;
    let parsed: Value = serde_json::from_str(&config)?;
    let provider = from_json(&config, None, EnvKeyResolver {})?;
    let operation = Compaction::new(
        Arc::from(provider),
        ModelConfig::new(first_configured_model(&parsed)?),
        Policy::default(),
    )?;
    let token = CancellationToken::new();
    token.cancel();
    let (sender, _receiver) = mpsc::channel(4);
    let emit = Emitter::new(sender, token);
    let outcome = operation.run(&state, &state.conversation, &emit).await?;
    let result = match outcome {
        OperationResult::Applied(result) => result,
        _ => panic!("yield"),
    };
    assert!(result.yield_to_client);
    assert_eq!(result.effects.len(), 1);
    let mut outcome = match result.effects.into_iter().next().expect("effect") {
        ChatEffect::Maintenance(outcome) => outcome,
        _ => panic!("maintenance effect"),
    };
    assert!(outcome.report.disposition.contains("cancelled"));
    assert!(outcome.report.summary.is_none());
    assert!(outcome.report.summary_usage.is_empty());
    apply_outcome(&mut state, &mut outcome)?;
    assert_eq!(
        serde_json::to_value(state.conversation.messages())?,
        original
    );
    assert!(state.maintenance.is_none());
    Ok(())
}

#[tokio::test]
async fn accepted_and_rejected_proposal_ids_cannot_replay_local_approval() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    {
        let mut sessions = runtime.sessions.lock().await;
        let stored = sessions.get_mut(SESSION_ID).expect("fixture");
        stored.lesson.persistence_enabled = true;
        for (id, approved) in [("accepted", true), ("rejected", false)] {
            stored.settled_approvals.insert(id.to_string(), approved);
            stored.conversation.push(
                Message::assistant()
                    .with_tool_request(id, Ok(CallToolRequestParams::new(RECORD_TOOL))),
            );
            stored.conversation.push(Message::user().with_tool_response(
                id,
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "Proposal prepared",
                )])),
            ));
        }
    }
    let path = env::temp_dir().join(format!("l13-replay-{}.md", std::process::id()));
    let mut diagnostics = DiagnosticLog::create_at(path.clone())?;
    // Invalid/missing proposal arguments would fail if the replay guard were absent.
    approve_record_proposals(&runtime, SESSION_ID, &mut diagnostics).await?;
    let saved = runtime.load(SESSION_ID).await?;
    assert_eq!(saved.settled_approvals.len(), 2);
    assert!(saved.settled_approvals["accepted"]);
    assert!(!saved.settled_approvals["rejected"]);
    fs::remove_file(path)?;
    Ok(())
}
#[test]
fn material_reduction_requires_both_absolute_relative_and_total_allowances() {
    let policy = Policy::default();
    assert!(material_reduction(60000, 50000, &policy));
    assert!(!material_reduction(60000, 59000, &policy));
    assert!(!material_reduction(60000, 55000, &policy));
    assert!(!material_reduction(200000, 150000, &policy));
    assert!(!material_reduction(900, 1, &policy));
    assert!(!material_reduction(60000, 60000, &policy));
}
