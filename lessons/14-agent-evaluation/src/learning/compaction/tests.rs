// Lesson 14
use super::*;
use goose_agent::operation::messages_since_kickoff;
use goose_providers::model::ModelConfig;
use rmcp::model::{CallToolRequestParams, ContentBlock, ErrorData};
use std::{process::Command, sync::Arc};

const TOOL_NAME: &str = "calculate_maximum_planned_loss";
const GROUNDING_ESCALATION_MESSAGE: &str =
    "Application-authored reminder: answer the same learner question again.";

// Authored ordinary histories, independent of application runtime and private data.
struct History {
    conversation: Conversation,
    genuine_reply_ids: Vec<String>,
    settled_approvals: HashMap<String, bool>,
}

fn history(count: usize, width: usize) -> History {
    let mut messages = vec![
        identified(Message::user().with_text("Begin lesson."), "opening-input"),
        identified(Message::assistant().with_text("Welcome."), "opening-answer"),
    ];
    let mut genuine_reply_ids = Vec::new();
    for index in 0..count {
        let id = format!("genuine-{}", index);
        let input = identified(
            Message::user().with_text(format!("Genuine reply {}", index)),
            &id,
        );
        genuine_reply_ids.push(id);
        messages.push(input);
        messages.push(identified(
            Message::assistant().with_text(format!("Explanation {} {}", index, "x".repeat(width))),
            &format!("answer-{}", index),
        ));
    }
    History {
        conversation: Conversation::new_unvalidated(messages),
        genuine_reply_ids,
        settled_approvals: HashMap::new(),
    }
}

fn identified(mut message: Message, id: &str) -> Message {
    message.id = Some(id.to_string());
    message
}

fn record_tool_definition() -> Tool {
    let schema = serde_json::from_value(json!({
        "type": "object", "properties": {"note": {"type": "string"}},
        "required": ["note"], "additionalProperties": false
    }))
    .expect("authored schema");
    Tool::new(
        RECORD_TOOL,
        "Authored proposal fixture; never executed.",
        Arc::new(schema),
    )
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
    model.max_tokens = Some(4096);
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

#[test]
fn partition_keeps_opening_two_genuine_exchanges_pairs_and_metadata() -> anyhow::Result<()> {
    let mut state = history(4, 5000);
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
    let parts = partition(
        &state.conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals,
    )?
    .expect("eligible");
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

#[test]
fn unsafe_pair_active_exchange_or_pending_approval_never_compacts() -> anyhow::Result<()> {
    let mut state = history(4, 5000);
    state.conversation.push(
        Message::assistant()
            .with_tool_request("pending", Ok(CallToolRequestParams::new(TOOL_NAME))),
    );
    assert!(partition(
        &state.conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals
    )
    .is_err());
    let mut state = history(4, 5000);
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
    let key = approval_key(&messages[2], "approval")?;
    state.conversation = Conversation::new_unvalidated(messages);
    assert!(partition(
        &state.conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals
    )
    .is_err());
    state.settled_approvals.insert(key.clone(), false);
    assert!(partition(
        &state.conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals
    )?
    .is_some());
    state.settled_approvals.insert(key, true);
    assert!(partition(
        &state.conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals
    )?
    .is_some());
    Ok(())
}

#[test]
fn escalated_attempts_reuse_tool_ids_without_splitting_genuine_partition() -> anyhow::Result<()> {
    let mut state = history(4, 5000);
    let original = state.conversation.messages();
    let mut attempts = Vec::new();
    for kickoff in [
        original[2].clone(),
        Message::user()
            .with_text(GROUNDING_ESCALATION_MESSAGE)
            .with_generated_id_if_missing(),
    ] {
        attempts.push(kickoff);
        attempts.push(
            Message::assistant()
                .with_tool_request(
                    "reused-calculator-id",
                    Ok(CallToolRequestParams::new(TOOL_NAME)),
                )
                .with_generated_id_if_missing(),
        );
        attempts.push(
            Message::user()
                .with_tool_response(
                    "reused-calculator-id",
                    Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
                )
                .with_generated_id_if_missing(),
        );
        attempts.push(
            Message::assistant()
                .with_text(format!("Complete calculation answer {}", "x".repeat(5000)))
                .with_generated_id_if_missing(),
        );
    }
    assert!(complete_exchange(&attempts));
    let mut messages = original[..2].to_vec();
    messages.extend(attempts.clone());
    messages.extend_from_slice(&original[4..]);
    state.conversation = Conversation::new_unvalidated(messages);
    let parts = partition(
        &state.conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals,
    )?
    .expect("complete genuine exchanges remain eligible");
    assert_eq!(parts.eligible.len(), attempts.len() + 2);
    assert_eq!(parts.recent.len(), 4);
    assert_eq!(parts.recent[0].as_concat_text(), "Genuine reply 2");
    assert_eq!(state.genuine_reply_ids.len(), 4);
    assert!(complete_exchange(&parts.eligible[..attempts.len()]));
    Ok(())
}

#[test]
fn duplicate_tool_ids_within_one_attempt_are_rejected() {
    let request = Message::assistant()
        .with_tool_request("duplicate-id", Ok(CallToolRequestParams::new(TOOL_NAME)));
    let response = Message::user().with_tool_response(
        "duplicate-id",
        Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
    );
    let kickoff = Message::user().with_text("Calculate planned loss.");
    let final_answer = Message::assistant().with_text("The result is $100.00.");
    assert!(!complete_exchange(&[
        kickoff.clone(),
        request.clone(),
        response.clone(),
        request.clone(),
        response.clone(),
        final_answer.clone(),
    ]));
    assert!(!complete_exchange(&[
        kickoff,
        request,
        response.clone(),
        response,
        final_answer,
    ]));
}

#[test]
fn application_kickoff_cannot_resolve_a_previous_attempts_unfinished_tool_pair() {
    let kickoff = Message::user().with_text("Calculate planned loss.");
    let request = Message::assistant().with_tool_request(
        "cross-boundary-id",
        Ok(CallToolRequestParams::new(TOOL_NAME)),
    );
    let response = Message::user().with_tool_response(
        "cross-boundary-id",
        Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
    );
    let escalation = Message::user().with_text(GROUNDING_ESCALATION_MESSAGE);
    let final_answer = Message::assistant().with_text("An ordinary final answer.");
    // An ordinary answer alone cannot settle an unanswered tool request.
    assert!(!complete_exchange(&[
        kickoff.clone(),
        request.clone(),
        final_answer.clone(),
        escalation.clone(),
        response.clone(),
        final_answer.clone(),
    ]));
    // A paired request/result still needs an ordinary answer before a new kickoff.
    assert!(!complete_exchange(&[
        kickoff.clone(),
        request.clone(),
        response.clone(),
        escalation.clone(),
        final_answer.clone(),
    ]));
    // A result from a completed prior attempt is orphaned in the new window.
    assert!(!complete_exchange(&[
        kickoff,
        request,
        response.clone(),
        final_answer.clone(),
        escalation,
        response,
        final_answer,
    ]));
}

#[test]
fn hidden_user_messages_do_not_reset_attempt_correlation() {
    let request = Message::assistant()
        .with_tool_request("same-attempt-id", Ok(CallToolRequestParams::new(TOOL_NAME)));
    let response = Message::user().with_tool_response(
        "same-attempt-id",
        Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
    );
    assert!(!complete_exchange(&[
        Message::user().with_text("Calculate planned loss."),
        request.clone(),
        response.clone(),
        Message::assistant().with_text("First answer."),
        Message::user()
            .with_text("Hidden application data.")
            .agent_only(),
        request,
        response,
        Message::assistant().with_text("Second answer."),
    ]));
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

#[test]
fn reused_record_ids_require_separate_settlements_for_genuine_and_escalated_attempts(
) -> anyhow::Result<()> {
    for approved in [false, true] {
        for escalation in [false, true] {
            let mut state = history(4, 5000);
            let mut messages = state.conversation.messages().clone();
            let first_key = approval_key(&messages[2], "reused-record")?;
            messages.insert(
                3,
                Message::assistant().with_tool_request(
                    "reused-record",
                    Ok(CallToolRequestParams::new(RECORD_TOOL)),
                ),
            );
            messages.insert(
                4,
                Message::user().with_tool_response(
                    "reused-record",
                    Ok(CallToolResult::success(vec![ContentBlock::text(
                        "Prepared",
                    )])),
                ),
            );
            if escalation {
                messages.insert(
                    6,
                    Message::user()
                        .with_text(GROUNDING_ESCALATION_MESSAGE)
                        .with_generated_id_if_missing(),
                );
                messages.insert(
                    7,
                    Message::assistant().with_text("Application retry answer."),
                );
            }
            // Index 6 is either the next genuine reply or the application escalation.
            let second_key = approval_key(&messages[6], "reused-record")?;
            messages.insert(
                7,
                Message::assistant().with_tool_request(
                    "reused-record",
                    Ok(CallToolRequestParams::new(RECORD_TOOL)),
                ),
            );
            messages.insert(
                8,
                Message::user().with_tool_response(
                    "reused-record",
                    Ok(CallToolResult::success(vec![ContentBlock::text(
                        "Prepared",
                    )])),
                ),
            );
            state.conversation = Conversation::new_unvalidated(messages);
            state.settled_approvals.insert(first_key.clone(), approved);
            assert_ne!(first_key, second_key);
            assert!(state.settled_approvals.contains_key(&first_key));
            assert!(!state.settled_approvals.contains_key(&second_key));
            let error = match partition(
                &state.conversation,
                &state.genuine_reply_ids,
                &state.settled_approvals,
            ) {
                Err(error) => error,
                Ok(_) => panic!("the second successful proposal still needs its own decision"),
            };
            assert!(error.to_string().contains("approval is still pending"));
            state
                .settled_approvals
                .insert(second_key.clone(), !approved);
            assert!(state.settled_approvals.contains_key(&second_key));
            assert!(partition(
                &state.conversation,
                &state.genuine_reply_ids,
                &state.settled_approvals
            )?
            .is_some());
            assert_eq!(state.settled_approvals[&first_key], approved);
        }
    }
    Ok(())
}

#[test]
fn pending_scan_never_matches_a_prior_attempt_success_to_a_rejected_or_missing_result(
) -> anyhow::Result<()> {
    for response in ["rejected", "missing", "error"] {
        let mut state = history(4, 5000);
        let mut messages = state.conversation.messages().clone();
        let first_key = approval_key(&messages[2], "reused-record")?;
        messages.insert(
            3,
            Message::assistant()
                .with_tool_request("reused-record", Ok(CallToolRequestParams::new(RECORD_TOOL))),
        );
        messages.insert(
            4,
            Message::user().with_tool_response(
                "reused-record",
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "Prepared",
                )])),
            ),
        );
        messages.insert(
            7,
            Message::assistant()
                .with_tool_request("reused-record", Ok(CallToolRequestParams::new(RECORD_TOOL))),
        );
        if response != "missing" {
            let result = if response == "rejected" {
                Ok(CallToolResult::error(vec![ContentBlock::text("Rejected")]))
            } else {
                Err(ErrorData::invalid_params("Rejected", None))
            };
            messages.insert(
                8,
                Message::user().with_tool_response("reused-record", result),
            );
        }
        state.conversation = Conversation::new_unvalidated(messages);
        state.settled_approvals.insert(first_key, true);
        if response == "missing" {
            let error = match partition(
                &state.conversation,
                &state.genuine_reply_ids,
                &state.settled_approvals,
            ) {
                Err(error) => error,
                Ok(_) => panic!("missing result must fail whole-exchange validation"),
            };
            assert!(error
                .to_string()
                .contains("complete opening and learner exchanges"));
        } else {
            assert!(partition(
                &state.conversation,
                &state.genuine_reply_ids,
                &state.settled_approvals
            )?
            .is_some());
        }
    }
    Ok(())
}

#[test]
fn partition_minimum_history_has_exact_4000_byte_boundary() -> anyhow::Result<()> {
    assert_eq!(COMPACTION_MIN_HISTORY_BYTES, 4000);
    for target in [3999, 4000, 4001] {
        let baseline = history(3, 0);
        let base_size = serde_json::to_vec(&baseline.conversation.messages()[2..4])?.len();
        assert!(base_size < target);
        let state = history(3, target - base_size);
        assert_eq!(
            serde_json::to_vec(&state.conversation.messages()[2..4])?.len(),
            target
        );
        let before = serde_json::to_value(&state.conversation)?;
        let result = partition(
            &state.conversation,
            &state.genuine_reply_ids,
            &state.settled_approvals,
        )?;
        assert_eq!(result.is_some(), target >= 4000);
        assert_eq!(serde_json::to_value(&state.conversation)?, before);
    }
    Ok(())
}

#[test]
fn latest_two_and_opening_are_protected_regardless_of_size() -> anyhow::Result<()> {
    for count in 0..=2 {
        let state = history(count, 9000);
        assert!(partition(
            &state.conversation,
            &state.genuine_reply_ids,
            &state.settled_approvals
        )?
        .is_none());
    }
    let state = history(5, 5000);
    let parts = partition(
        &state.conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals,
    )?
    .expect("eligible");
    assert_eq!(
        serde_json::to_value(&parts.opening)?,
        serde_json::to_value(&state.conversation.messages()[..2])?
    );
    assert_eq!(
        serde_json::to_value(&parts.eligible)?,
        serde_json::to_value(&state.conversation.messages()[2..8])?
    );
    assert_eq!(
        serde_json::to_value(&parts.recent)?,
        serde_json::to_value(&state.conversation.messages()[8..])?
    );
    Ok(())
}

#[test]
fn prior_summary_is_merged_once_and_marker_is_not_authority() -> anyhow::Result<()> {
    let state = history(5, 5000);
    let mut messages = state.conversation.messages().clone();
    let mut old = identified(
        Message::assistant().with_text("Old fallible summary."),
        "prior-summary",
    );
    old.metadata
        .set_operation_note("compaction", "derived_summary", json!(true));
    messages.insert(2, old.clone());
    let conversation = Conversation::new_unvalidated(messages);
    let before = serde_json::to_value(&conversation)?;
    let parts = partition(
        &conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals,
    )?
    .expect("eligible");
    assert_eq!(
        parts
            .eligible
            .iter()
            .filter(|message| is_summary(message))
            .count(),
        1
    );
    assert_eq!(parts.eligible[0].id, old.id);
    let replacement = candidate(parts, "Merged fallible summary.");
    let summaries: Vec<&Message> = replacement
        .messages()
        .iter()
        .filter(|message| is_summary(message))
        .collect();
    assert_eq!(summaries.len(), 1);
    assert_eq!(summaries[0].role, rmcp::model::Role::Assistant);
    assert!(summaries[0].id.is_some());
    assert_eq!(
        summaries[0].as_concat_text(),
        format!("{}Merged fallible summary.", SUMMARY_MARKER)
    );
    assert!(!replacement
        .messages()
        .iter()
        .any(|message| message.id == old.id));
    assert_eq!(serde_json::to_value(&conversation)?, before);
    assert!(!is_summary(&Message::assistant().with_text(SUMMARY_MARKER)));
    let mut false_flag = Message::assistant().with_text("Not derived.");
    false_flag
        .metadata
        .set_operation_note("compaction", "derived_summary", json!(false));
    assert!(!is_summary(&false_flag));
    Ok(())
}

#[test]
fn malformed_summary_role_or_genuine_identity_is_rejected() {
    let state = history(4, 5000);
    let mut messages = state.conversation.messages().clone();
    let mut invalid_summary = Message::user().with_text("Wrong role.");
    invalid_summary
        .metadata
        .set_operation_note("compaction", "derived_summary", json!(true));
    messages.insert(2, invalid_summary);
    let conversation = Conversation::new_unvalidated(messages);
    assert_eq!(
        partition(
            &conversation,
            &state.genuine_reply_ids,
            &state.settled_approvals
        )
        .err()
        .expect("invalid")
        .to_string(),
        "invalid summary role"
    );
    let mut invalid_ids = state.genuine_reply_ids.clone();
    invalid_ids.push("answer-0".to_string());
    assert_eq!(
        partition(&state.conversation, &invalid_ids, &state.settled_approvals)
            .err()
            .expect("invalid")
            .to_string(),
        "invalid reply provenance"
    );
    let mut messages = state.conversation.messages().clone();
    messages[2] = identified(
        Message::user().with_tool_response("orphan", Ok(CallToolResult::success(Vec::new()))),
        "genuine-0",
    );
    assert!(partition(
        &Conversation::new_unvalidated(messages),
        &state.genuine_reply_ids,
        &state.settled_approvals
    )
    .is_err());
}

#[test]
fn reminder_is_history_not_genuine_evidence_and_stays_in_same_partition() -> anyhow::Result<()> {
    let state = history(4, 5000);
    let mut messages = state.conversation.messages().clone();
    messages.insert(
        4,
        identified(
            Message::user().with_text(GROUNDING_ESCALATION_MESSAGE),
            "reminder",
        ),
    );
    messages.insert(
        5,
        identified(
            Message::assistant().with_text("Retry answer."),
            "retry-answer",
        ),
    );
    let conversation = Conversation::new_unvalidated(messages);
    let parts = partition(
        &conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals,
    )?
    .expect("eligible");
    assert_eq!(parts.eligible.len(), 6);
    assert_eq!(parts.recent[0].id.as_deref(), Some("genuine-2"));
    let records = summary_source(&parts.eligible, &state.genuine_reply_ids);
    let reminder = records
        .iter()
        .find(|record| record["original_message_id"] == "reminder")
        .expect("retained");
    assert_eq!(
        reminder["provenance"],
        "application or tool history (NOT learner evidence)"
    );
    assert_eq!(reminder["content"][0]["text"], GROUNDING_ESCALATION_MESSAGE);
    Ok(())
}

#[test]
fn hidden_user_data_does_not_split_a_complete_correlated_pair() {
    let messages = vec![
        Message::user().with_text("Question."),
        Message::assistant().with_tool_request("pair", Ok(CallToolRequestParams::new(TOOL_NAME))),
        Message::user()
            .with_text("Hidden application data.")
            .agent_only(),
        Message::user().with_tool_response("pair", Ok(CallToolResult::success(Vec::new()))),
        Message::assistant().with_text("Answer."),
    ];
    assert!(complete_exchange(&messages));
    assert!(!complete_exchange(&[]));
    assert!(!complete_exchange(&[
        Message::user().with_text("No answer.")
    ]));
    for id in ["", "orphan"] {
        assert!(!complete_exchange(&[
            Message::assistant().with_tool_request("", Ok(CallToolRequestParams::new(TOOL_NAME))),
            Message::user().with_tool_response(id, Ok(CallToolResult::success(Vec::new()))),
            Message::assistant().with_text("Answer."),
        ]));
    }
}

#[test]
fn successful_unsettled_proposal_in_opening_blocks_even_short_history() -> anyhow::Result<()> {
    let state = history(2, 5000);
    let mut messages = state.conversation.messages().clone();
    messages.insert(
        1,
        Message::assistant().with_tool_request(
            "opening-proposal",
            Ok(CallToolRequestParams::new(RECORD_TOOL)),
        ),
    );
    messages.insert(
        2,
        Message::user()
            .with_tool_response("opening-proposal", Ok(CallToolResult::success(Vec::new()))),
    );
    let conversation = Conversation::new_unvalidated(messages);
    let error = partition(
        &conversation,
        &state.genuine_reply_ids,
        &state.settled_approvals,
    )
    .err()
    .expect("pending");
    assert!(error.to_string().contains("approval is still pending"));
    for approved in [false, true] {
        let key = approval_key(&conversation.messages()[0], "opening-proposal")?;
        let settled = HashMap::from([(key, approved)]);
        assert!(partition(&conversation, &state.genuine_reply_ids, &settled)?.is_none());
    }
    Ok(())
}

#[test]
fn successful_proposal_requires_stable_kickoff_identity() {
    let state = history(4, 5000);
    let mut messages = state.conversation.messages().clone();
    messages[2].id = None;
    messages.insert(
        3,
        Message::assistant()
            .with_tool_request("proposal", Ok(CallToolRequestParams::new(RECORD_TOOL))),
    );
    messages.insert(
        4,
        Message::user().with_tool_response("proposal", Ok(CallToolResult::success(Vec::new()))),
    );
    let error = partition(
        &Conversation::new_unvalidated(messages),
        &state.genuine_reply_ids,
        &state.settled_approvals,
    )
    .err()
    .expect("missing kickoff identity");
    assert_eq!(
        error.to_string(),
        "approval kickoff has no stable message ID"
    );
}

#[test]
fn byte_guard_accepts_equality_and_keeps_original_error_text() -> anyhow::Result<()> {
    let policy = Policy::default();
    check_estimate(84_000, &policy)?;
    let error = check_estimate(84_001, &policy).expect_err("over budget");
    assert_eq!(error.to_string(), "Working input exceeds the conservative byte estimate allowance (84001 bytes plus 16000 reserved headroom; total 100000). Static content or the active exchange may be too large; history was not silently truncated. Start a fresh selection or reduce editable content/input.");
    let impossible = Policy {
        trigger_bytes: 1,
        total_bytes: 10,
        headroom_bytes: 20,
    };
    check_estimate(0, &impossible)?;
    assert!(check_estimate(1, &impossible).is_err());
    Ok(())
}

#[test]
fn payload_estimate_counts_exact_native_protocol_arguments_and_thinking_replay(
) -> anyhow::Result<()> {
    let call = CallToolRequestParams::new(TOOL_NAME)
        .with_arguments(serde_json::from_value(json!({"value": "\\\"\né"}))?);
    let tools = vec![record_tool_definition()];
    let conversation = Conversation::new_unvalidated(vec![
        Message::user().with_text("Ordinary input."),
        Message::assistant()
            .with_thinking("private thinking", "signature")
            .with_tool_request("first", Ok(call.clone()))
            .with_tool_request("second", Ok(call.clone())),
    ]);
    let native = serde_json::to_vec(
        &json!({"system_prompt":"prompt", "tools":tools, "messages":conversation.messages()}),
    )?
    .len();
    let arguments = serde_json::to_string(&call.arguments)?;
    let extra_arguments = serde_json::to_vec(&arguments)?.len() * 2;
    let thinking = serde_json::to_vec("private thinking")?.len() * 2;
    // Two message envelopes, four content blocks, one tool definition.
    assert_eq!(
        payload_bytes("prompt", &tools, &conversation)?,
        native + PROTOCOL_ENVELOPE_BYTES + PROTOCOL_ITEM_BYTES * 7 + extra_arguments + thinking
    );
    let base = payload_bytes("prompt", &[], &Conversation::new_unvalidated(Vec::new()))?;
    assert!(payload_bytes(&"p".repeat(90_000), &[], &conversation)? > base + 84_000);
    let mut metadata = Message::assistant().with_text("Short.");
    let before = payload_bytes(
        "prompt",
        &[],
        &Conversation::new_unvalidated(vec![metadata.clone()]),
    )?;
    metadata
        .metadata
        .set_operation_note("fixture", "large", json!("x".repeat(90_000)));
    let measured = payload_bytes(
        "prompt",
        &[],
        &Conversation::new_unvalidated(vec![metadata]),
    )?;
    assert!(measured > before + 84_000);
    assert!(check_estimate(measured, &Policy::default()).is_err());
    let result = Conversation::new_unvalidated(vec![Message::user().with_tool_response(
        "large",
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "x".repeat(90_000),
        )])),
    )]);
    assert!(check_estimate(payload_bytes("prompt", &[], &result)?, &Policy::default()).is_err());
    Ok(())
}

#[test]
fn material_reduction_boundary_edges_are_independent() {
    let policy = Policy::default();
    assert!(material_reduction(10_240, 9_216, &policy));
    assert!(!material_reduction(10_239, 9_216, &policy));
    assert!(material_reduction(60_000, 54_000, &policy));
    assert!(!material_reduction(60_000, 54_001, &policy));
    assert!(material_reduction(100_000, 84_000, &policy));
    assert!(!material_reduction(100_000, 84_001, &policy));
}

#[test]
fn summary_validation_rejects_error_role_empty_fallback_and_multibyte_overflow() {
    use goose_providers::conversation::message::MessageErrorKind;
    for text in [
        "",
        " \n\t",
        "The model returned an empty response. Please resend your message to continue.",
    ] {
        assert!(!valid_summary_output(
            &[Message::assistant().with_text(text)],
            text
        ));
    }
    assert!(!valid_summary_output(
        &[Message::user().with_text("Wrong role.")],
        "Wrong role."
    ));
    assert!(!valid_summary_output(
        &[Message::assistant().with_error(MessageErrorKind::Authentication, "failure")],
        "summary"
    ));
    let text = "é".repeat(SUMMARY_BYTES / 2);
    assert!(valid_summary_output(
        &[Message::assistant().with_text(&text)],
        &text
    ));
    let too_long = format!("{}é", text);
    assert!(!valid_summary_output(
        &[Message::assistant().with_text(&too_long)],
        &too_long
    ));
}

#[test]
fn policy_defaults_are_explicit_not_environment_loaded() {
    let policy = Policy::default();
    assert_eq!(
        (
            policy.trigger_bytes,
            policy.total_bytes,
            policy.headroom_bytes
        ),
        (84_000, 100_000, 16_000)
    );
    assert_eq!(
        (MAX_REPLY_IDS, MAX_SETTLED_APPROVALS, MAX_COMPLETIONS),
        (256, 256, 128)
    );
    assert_eq!((SUMMARY_BYTES, SUMMARY_INPUT_BYTES), (6000, 84000));
}

// Exercise real environment reads only in separate test processes. No test mutates
// the parent process environment, so these cases cannot race another test.
#[test]
fn policy_from_env_uses_isolated_processes() -> anyhow::Result<()> {
    let executable = std::env::current_exe()?;
    let module = module_path!()
        .split_once("::")
        .expect("module-qualified test")
        .1;
    let name = format!("{}::policy_from_env_isolated_child", module);
    let cases = [
        ("default", None),
        ("custom-default", None),
        ("one", Some("1")),
        ("edge", Some("84000")),
        ("custom", Some("1500")),
        ("zero", Some("0")),
        ("over", Some("84001")),
        ("bad", Some("abc")),
        ("negative", Some("-1")),
        ("empty", Some("")),
        ("space", Some(" 1")),
        ("overflow", Some("999999999999999999999999999999999999")),
        ("no-allowance", None),
    ];
    for (case, value) in cases {
        let mut command = Command::new(&executable);
        command
            .args(["--exact", &name, "--nocapture"])
            .env("L14_COMPACTION_POLICY_CASE", case)
            .env_remove("COMPACTION_TRIGGER_BYTES");
        if let Some(value) = value {
            command.env("COMPACTION_TRIGGER_BYTES", value);
        }
        let output = command.output()?;
        assert!(
            output.status.success(),
            "{}: {} {}",
            case,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        let output = Command::new(&executable)
            .args(["--exact", &name, "--nocapture"])
            .env("L14_COMPACTION_POLICY_CASE", "nonunicode")
            .env(
                "COMPACTION_TRIGGER_BYTES",
                std::ffi::OsString::from_vec(vec![0xff]),
            )
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
    }
    Ok(())
}

#[test]
fn policy_from_env_isolated_child() {
    let case = match std::env::var("L14_COMPACTION_POLICY_CASE") {
        Ok(case) => case,
        Err(_error) => return,
    };
    // Even an override in this child must not be read by Default itself.
    assert_eq!(Policy::default().trigger_bytes, 84_000);
    let supplied = if case == "custom" || case == "custom-default" {
        Policy {
            trigger_bytes: 1200,
            total_bytes: 2000,
            headroom_bytes: 500,
        }
    } else if case == "no-allowance" {
        Policy {
            trigger_bytes: 1,
            total_bytes: 10,
            headroom_bytes: 20,
        }
    } else {
        Policy::default()
    };
    let result = Policy::from_env(supplied);
    match case.as_str() {
        "default" | "one" | "edge" | "custom" | "custom-default" => {
            let policy = result.expect("valid explicit policy/environment");
            let expected = match case.as_str() {
                "one" => 1,
                "custom" => 1500,
                "custom-default" => 1200,
                _ => 84000,
            };
            assert_eq!(policy.trigger_bytes, expected);
            assert_eq!(
                (policy.total_bytes, policy.headroom_bytes),
                if case.starts_with("custom") {
                    (2000, 500)
                } else {
                    (100000, 16000)
                }
            );
        }
        _ => {
            let error = result.expect_err("invalid environment/policy").to_string();
            assert!(error.starts_with("COMPACTION_TRIGGER_BYTES must be"));
        }
    }
}

#[test]
fn settlement_keys_are_json_pairs_not_ambiguous_concatenation() -> anyhow::Result<()> {
    let kickoff = identified(Message::user().with_text("Question."), "a:b");
    assert_eq!(
        approval_key(&kickoff, "c")?,
        serde_json::to_string(&json!(["a:b", "c"]))?
    );
    let other = identified(Message::user().with_text("Question."), "a");
    assert_ne!(approval_key(&kickoff, "c")?, approval_key(&other, "b:c")?);
    let quoted = identified(Message::user().with_text("Question."), "a\"b");
    assert_eq!(
        approval_key(&quoted, "c\\d")?,
        serde_json::to_string(&json!(["a\"b", "c\\d"]))?
    );
    Ok(())
}
