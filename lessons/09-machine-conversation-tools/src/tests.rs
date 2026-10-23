// Lesson 9
use super::*;
use crate::calculator::execute_maximum_planned_loss;
use crate::presentation::session_fields;
use crate::provider_setup::DEFAULT_PROVIDER_CONFIG;
use goose_agent::operation::Inference;
use goose_providers::conversation::token_usage::Usage;
use rmcp::model::JsonObject;

#[tokio::test]
async fn effects_are_saved_and_loading_returns_an_independent_snapshot() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let before = runtime.load(SESSION_ID).await?;
    let (sender, _receiver) = mpsc::channel(2);
    let emit = Emitter::new(sender, CancellationToken::new());
    let usage = ProviderUsage::new("test-model".to_string(), Usage::default());
    let mut effects = vec![
        ChatEffect::from(Message::assistant().with_text("Paris.")),
        ChatEffect::record_usage(usage),
    ];
    for effect in &mut effects {
        effect.ensure_message_ids();
    }
    runtime.apply_effects(&before, &mut effects, &emit).await?;
    let after = runtime.load(SESSION_ID).await?;
    assert_eq!(before.conversation.len(), 1);
    assert_eq!(after.conversation.len(), 2);
    assert_eq!(after.usage.len(), 1);
    let fields = session_fields(&after);
    for name in [
        "model:",
        "input_tokens:",
        "output_tokens:",
        "total_tokens:",
        "cache_read_input_tokens:",
        "cache_write_input_tokens:",
        "stats:",
        "cost:",
        "cost_source:",
        "finish_reasons:",
        "response_id:",
        "additional_data:",
    ] {
        assert!(fields.contains(name), "missing usage field {}", name);
    }
    assert!(fields.contains("stats: None"));
    let saved = match after.conversation.last() {
        Some(saved) => saved,
        None => return Err(anyhow::anyhow!("missing saved reply")),
    };
    assert_eq!(saved.as_concat_text(), "Paris.");
    assert!(saved.id.is_some());
    assert!(runtime.load("missing").await.is_err());
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

#[tokio::test]
async fn empty_response_diagnostic_is_not_an_ordinary_answer() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session.conversation.push(Message::assistant().with_text(
        "The model returned an empty response. Please resend your message to continue.",
    ));
    let summary = EventSummary {
        saw_text: true,
        displayed_text: true,
        ..EventSummary::default()
    };
    assert!(inspect_turn(&session, &summary, 1).is_err());
    Ok(())
}

#[tokio::test]
async fn event_consumption_does_not_save_messages_and_drains_after_close() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let (sender, receiver) = mpsc::channel(1);
    let producer = async {
        for _index in 0..3 {
            sender
                .send(AgentEvent::Message(
                    Message::assistant().with_text("Paris."),
                ))
                .await?;
        }
        drop(sender);
        Ok::<(), anyhow::Error>(())
    };
    let (produced, displayed) = tokio::join!(producer, display_events(receiver));
    produced?;
    let summary = displayed?;
    assert_eq!(summary.message_events, 3);
    assert!(summary.saw_text);
    let session = runtime.load(SESSION_ID).await?;
    assert_eq!(session.conversation.len(), 1);
    Ok(())
}

#[tokio::test]
async fn ordinary_saved_answer_makes_inference_decline_without_a_provider_call(
) -> anyhow::Result<()> {
    let json = fs::read_to_string(DEFAULT_PROVIDER_CONFIG)?;
    let provider = from_json(&json, None, EnvKeyResolver {})?;
    let config: Value = serde_json::from_str(&json)?;
    let model = ModelConfig::new(first_configured_model(&config)?);
    let runner: InferenceRunner<'_, ChatSession, ChatEffect> =
        InferenceRunner::new(Arc::from(provider), model);
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    assert!(runner.applies(&session.conversation));
    session
        .conversation
        .push(Message::assistant().with_text("Paris."));
    assert!(!runner.applies(&session.conversation));
    let steps = vec![Step::Inference(Arc::new(runner))];
    let machine = StateMachine::new(steps, CancellationToken::new());
    runtime
        .sessions
        .lock()
        .await
        .insert(SESSION_ID.to_string(), session);
    let (sender, mut receiver) = mpsc::channel(1);
    let emit = Emitter::new(sender, CancellationToken::new());
    let final_session = machine.run(&runtime, SESSION_ID, &emit).await?;
    drop(emit);
    assert_eq!(final_session.conversation.len(), 2);
    assert!(receiver.recv().await.is_none());
    Ok(())
}
fn arguments(shares: u32) -> JsonObject {
    let value = json!({"entry_price": "51.20", "stop_price": "50.70", "share_count": shares});
    match value {
        Value::Object(arguments) => arguments,
        _ => panic!("test fixture is a JSON object"),
    }
}

fn request(id: &str, name: &str, shares: u32) -> Message {
    let call = CallToolRequestParams::new(name.to_string()).with_arguments(arguments(shares));
    Message::assistant().with_tool_request(id, Ok(call))
}

fn safe_tools() -> SafeTools {
    SafeTools {
        inner: ToolOperation::new().with_provider(Arc::new(LossToolProvider)),
    }
}

fn applied_effects(result: OperationResult<ChatEffect>) -> anyhow::Result<Vec<ChatEffect>> {
    match result {
        OperationResult::Applied(result) => Ok(result.effects),
        OperationResult::NotApplicable => Err(anyhow::anyhow!("expected applied operation")),
    }
}

#[test]
fn exact_loss_and_tweak_are_validated() -> Result<(), String> {
    let default_loss = execute_maximum_planned_loss(&arguments(200))?;
    let tweaked_loss = execute_maximum_planned_loss(&arguments(100))?;
    assert_eq!(default_loss, "$100.00");
    assert_eq!(tweaked_loss, "$50.00");
    Ok(())
}

#[test]
fn untrusted_shape_and_domain_values_fail_without_panicking() {
    for value in [
        json!({"entry_price":51.20,"stop_price":"50.70","share_count":200}),
        json!({"entry_price":"51.20","stop_price":"50.70","share_count":200,"ticker":"X"}),
        json!({"entry_price":"51.20","stop_price":"50.70"}),
        json!({"entry_price":"51.20","stop_price":"50.70","share_count":-1}),
        json!({"entry_price":"51.20","stop_price":"50.70","share_count":0}),
        json!({"entry_price":"51.20","stop_price":"50.70","share_count":1.5}),
        json!({"entry_price":"50.70","stop_price":"50.70","share_count":200}),
        json!({"entry_price":"49.00","stop_price":"50.70","share_count":200}),
        json!({"entry_price":"51.20000","stop_price":"50.70","share_count":200}),
        json!({"entry_price":"0","stop_price":"50.70","share_count":200}),
        json!({"entry_price":"51.20","stop_price":"-1","share_count":200}),
        json!({"entry_price":"79228162514264337593543950335","stop_price":"1","share_count":200}),
    ] {
        let args = match value {
            Value::Object(args) => args,
            _ => panic!("test fixture is a JSON object"),
        };
        assert!(execute_maximum_planned_loss(&args).is_err());
    }
    for raw in ["1e2", "+1", " 1", "1_0", ".5", "5.", "1.2.3", "not-money"] {
        assert!(parse_price("price", raw).is_err());
    }
    assert!(execute_allowed_tool(&Ok(CallToolRequestParams::new(TOOL_NAME))).is_err());
    let unknown = CallToolRequestParams::new("place_order").with_arguments(arguments(200));
    assert!(execute_allowed_tool(&Ok(unknown)).is_err());
    assert!(execute_allowed_tool(&Err(ErrorData::invalid_params("broken", None))).is_err());
}

#[tokio::test]
async fn same_capability_and_history_survive_the_new_user_turn() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let first = runtime.load(SESSION_ID).await?;
    let tools = safe_tools();
    let definitions_first = tools.inference_tools(&first).await?;
    let (sender, _receiver) = mpsc::channel(8);
    let emit = Emitter::new(sender, CancellationToken::new());
    let mut effects = vec![ChatEffect::from(
        Message::assistant().with_text("A stop-loss is a planned exit."),
    )];
    runtime.apply_effects(&first, &mut effects, &emit).await?;
    let before = runtime.load(SESSION_ID).await?;
    runtime
        .append_user("Calculate for my hypothetical trade")
        .await?;
    let second = runtime.load(SESSION_ID).await?;
    assert_eq!(
        &second.conversation.messages()[..before.conversation.len()],
        before.conversation.messages()
    );
    assert_eq!(second.id, first.id);
    assert_eq!(second.applied_passes, 0);
    assert_eq!(tools.inference_tools(&second).await?, definitions_first);
    assert_eq!(definitions_first.len(), 1);
    assert_eq!(definitions_first[0].name.as_ref(), TOOL_NAME);
    Ok(())
}

#[tokio::test]
async fn every_block_gets_a_correlated_response_including_unknown_and_malformed_calls(
) -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    let mixed = request("valid-1", TOOL_NAME, 200)
        .with_text("interleaved text")
        .with_tool_request("unknown", Ok(CallToolRequestParams::new("place_order")))
        .with_tool_request("broken", Err(ErrorData::invalid_params("bad syntax", None)))
        .with_tool_request(
            "valid-2",
            Ok(CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments(100))),
        );
    session.conversation.push(mixed);
    let (sender, mut receiver) = mpsc::channel(8);
    let emit = Emitter::new(sender, CancellationToken::new());
    let tools = safe_tools();
    // Unknown requests are answered first, without hiding the known ones.
    let first_result = tools.run(&session, &session.conversation, &emit).await?;
    let first_effects = applied_effects(first_result)?;
    for effect in first_effects {
        if let ChatEffect::AppendMessage(message) = effect {
            session.conversation.push(message);
        }
    }
    let second_result = tools.run(&session, &session.conversation, &emit).await?;
    let second_effects = applied_effects(second_result)?;
    for effect in second_effects {
        if let ChatEffect::AppendMessage(message) = effect {
            session.conversation.push(message);
        }
    }
    let mut responses = HashMap::new();
    for message in session.conversation.messages() {
        for block in &message.content {
            if let Some(response) = block.as_tool_response() {
                assert_eq!(message.role, rmcp::model::Role::User);
                assert!(responses
                    .insert(response.id.clone(), response.tool_result.clone())
                    .is_none());
            }
        }
    }
    assert_eq!(responses.len(), 4);
    assert!(matches!(responses.get("unknown"), Some(Err(_))));
    assert!(matches!(responses.get("broken"), Some(Err(_))));
    assert_eq!(
        responses.get("valid-1"),
        Some(&Ok(CallToolResult::success(vec![ContentBlock::text(
            "$100.00"
        )])))
    );
    assert_eq!(
        responses.get("valid-2"),
        Some(&Ok(CallToolResult::success(vec![ContentBlock::text(
            "$50.00"
        )])))
    );
    let declined = tools.run(&session, &session.conversation, &emit).await?;
    assert!(matches!(declined, OperationResult::NotApplicable));
    drop(emit);
    let mut events = 0;
    while receiver.recv().await.is_some() {
        events += 1;
    }
    assert_eq!(events, 2);
    Ok(())
}

#[tokio::test]
async fn request_bound_answers_excess_without_executing_and_then_yields() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    let mut message = Message::assistant();
    for index in 0..MAX_TOOL_REQUESTS + 2 {
        let id = format!("call-{}", index);
        message = message.with_tool_request(
            id,
            Ok(CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments(200))),
        );
    }
    session.conversation.push(message);
    let (sender, _receiver) = mpsc::channel(8);
    let emit = Emitter::new(sender, CancellationToken::new());
    let tools = safe_tools();
    for _pass in 0..2 {
        let result = tools.run(&session, &session.conversation, &emit).await?;
        let effects = applied_effects(result)?;
        for effect in effects {
            if let ChatEffect::AppendMessage(message) = effect {
                session.conversation.push(message);
            }
        }
    }
    let mut count = 0;
    let mut errors = 0;
    for message in session.conversation.messages() {
        for block in &message.content {
            if let Some(response) = block.as_tool_response() {
                count += 1;
                if response.tool_result.is_err() {
                    errors += 1;
                }
            }
        }
    }
    assert_eq!(count, MAX_TOOL_REQUESTS + 2);
    assert_eq!(errors, 2);
    let stopped = RunLimit.run(&session, &session.conversation, &emit).await?;
    match stopped {
        OperationResult::Applied(result) => {
            assert!(result.yield_to_client);
            assert!(matches!(result.effects.first(), Some(ChatEffect::Halt(_))));
        }
        OperationResult::NotApplicable => {
            return Err(anyhow::anyhow!("request limit did not halt"))
        }
    }
    Ok(())
}

#[tokio::test]
async fn duplicate_ids_halt_and_pass_bound_keeps_the_gdk_run_loop() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session.conversation.push(
        request("same", TOOL_NAME, 200)
            .with_tool_request("same", Ok(CallToolRequestParams::new(TOOL_NAME))),
    );
    let (sender, _receiver) = mpsc::channel(8);
    let emit = Emitter::new(sender, CancellationToken::new());
    let result = safe_tools()
        .run(&session, &session.conversation, &emit)
        .await?;
    match result {
        OperationResult::Applied(result) => assert!(result.yield_to_client),
        OperationResult::NotApplicable => {
            return Err(anyhow::anyhow!("duplicate IDs did not halt"))
        }
    }
    // No provider needed: the first operation halts before inference could act.
    let mut stored = runtime.sessions.lock().await;
    let session = match stored.get_mut(SESSION_ID) {
        Some(session) => session,
        None => return Err(anyhow::anyhow!("missing Session")),
    };
    session.applied_passes = MAX_APPLIED_PASSES;
    drop(stored);
    let steps = vec![Step::Operation(Arc::new(RunLimit))];
    let machine = StateMachine::new(steps, CancellationToken::new());
    let final_session = machine.run(&runtime, SESSION_ID, &emit).await?;
    assert!(final_session.halt_reason.is_some());
    assert_eq!(final_session.applied_passes, MAX_APPLIED_PASSES + 1);
    // Normal completion at the bound is not misreported as a runaway loop.
    let mut finished = final_session.clone();
    finished
        .conversation
        .push(Message::assistant().with_text("Done."));
    let result = RunLimit
        .run(&finished, &finished.conversation, &emit)
        .await?;
    assert!(matches!(result, OperationResult::NotApplicable));
    Ok(())
}
#[tokio::test]
async fn complete_request_display_precedes_tool_result_and_final_answer() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let before = runtime.load(SESSION_ID).await?;
    let (sender, mut receiver) = mpsc::channel(8);
    let emit = Emitter::new(sender, CancellationToken::new());
    let request_message = request("order-call", TOOL_NAME, 200).with_text("Calculating.");
    // Provider deltas may contain request fragments; they are not the saved display.
    emit.emit(AgentEvent::Message(request_message.clone()))
        .await;
    let mut effects = vec![ChatEffect::from(request_message)];
    runtime.apply_effects(&before, &mut effects, &emit).await?;
    let saved = runtime.load(SESSION_ID).await?;
    let tools_result = safe_tools().run(&saved, &saved.conversation, &emit).await?;
    let mut effects = applied_effects(tools_result)?;
    runtime.apply_effects(&saved, &mut effects, &emit).await?;
    emit.emit(AgentEvent::Message(
        Message::assistant().with_text("The planned loss is $100.00."),
    ))
    .await;
    drop(emit);
    let mut messages = Vec::new();
    while let Some(event) = receiver.recv().await {
        if let AgentEvent::Message(message) = event {
            messages.push(message);
        }
    }
    assert_eq!(messages.len(), 4);
    assert!(!is_reconstructed_request_display(&messages[0]));
    assert!(is_reconstructed_request_display(&messages[1]));
    assert_eq!(
        messages[1].get_tool_request_ids(),
        HashSet::from(["order-call"])
    );
    assert!(messages[2].is_tool_response());
    assert_eq!(
        messages[2].get_tool_response_ids(),
        HashSet::from(["order-call"])
    );
    assert_eq!(messages[3].as_concat_text(), "The planned loss is $100.00.");
    for message in saved.conversation.messages() {
        assert!(!is_reconstructed_request_display(message));
    }
    Ok(())
}

#[tokio::test]
async fn outbound_history_preview_retains_first_answer_and_changes_no_state() -> anyhow::Result<()>
{
    let runtime = ChatRuntime::seeded();
    let first = runtime.load(SESSION_ID).await?;
    let (sender, _receiver) = mpsc::channel(8);
    let emit = Emitter::new(sender, CancellationToken::new());
    let explanation = "A stop-loss is a planned exit, not a guaranteed execution price.";
    let mut effects = vec![ChatEffect::from(
        Message::assistant().with_text(explanation),
    )];
    runtime.apply_effects(&first, &mut effects, &emit).await?;
    let follow_up = "Calculate maximum planned loss for 200 shares.";
    runtime.append_user(follow_up).await?;
    let before = runtime.load(SESSION_ID).await?;
    let original = crate::presentation::session_fields(&before);
    let preview = crate::presentation::conversation_history_preview(&before);
    assert_eq!(preview, format!("{:#?}", before.conversation.messages()));
    assert!(preview.contains(USER_PROMPT));
    assert!(preview.contains(explanation));
    assert!(preview.contains(follow_up));
    assert!(preview.find(explanation) < preview.find(follow_up));
    assert!(!preview.contains('\x1b'));
    assert_eq!(crate::presentation::session_fields(&before), original);
    let after = runtime.load(SESSION_ID).await?;
    assert_eq!(crate::presentation::session_fields(&after), original);
    Ok(())
}
