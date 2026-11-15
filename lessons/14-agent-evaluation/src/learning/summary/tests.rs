// Lesson 14
use super::*;
use goose_agent::operation::applied;
use goose_providers::conversation::{
    message::{MessageContent, RedactedThinkingContentBlock},
    token_usage::Usage,
};
use rmcp::model::CallToolRequestParams;
use tracing::Instrument;

fn runtime(observer: Observer) -> SummaryRuntime {
    SummaryRuntime {
        state: Mutex::new(SummarySession {
            conversation: Conversation::new_unvalidated(vec![Message::user()
                .with_text("Public historical fixture.")
                .with_generated_id_if_missing()]),
            usage: Vec::new(),
            batches: 0,
        }),
        observer,
    }
}

fn usage() -> ProviderUsage {
    ProviderUsage::new(
        "public-fixture-model".to_string(),
        Usage {
            input_tokens: Some(11),
            output_tokens: Some(17),
            ..Default::default()
        },
    )
}

#[test]
fn guidance_and_fixed_execution_limits_match_the_original_summary() -> anyhow::Result<()> {
    assert!(validate_guidance("").is_err());
    assert!(validate_guidance(" \n\t").is_err());
    validate_guidance(&"é".repeat(6_000))?;
    assert!(validate_guidance(&"é".repeat(6_001)).is_err());
    assert_eq!(SUMMARY_MAX_TOKENS, 4096);
    assert_eq!(SUMMARY_BYTES, 6000);
    assert_eq!(SUMMARY_PARTIAL_MESSAGES, 4096);
    assert_eq!(STREAM_PROTOCOL_BYTES, 2_000_000);
    assert_eq!(RUN_TIMEOUT_SECONDS, 180);
    assert_eq!(CANCELLATION_GRACE_SECONDS, 5);
    Ok(())
}

#[tokio::test]
async fn stop_yields_after_one_effect_batch_without_any_provider() -> anyhow::Result<()> {
    struct FixtureBatch;
    #[async_trait]
    impl Operation<SummarySession, SummaryEffect> for FixtureBatch {
        fn name(&self) -> &'static str {
            "public-summary-fixture"
        }
        async fn run(
            &self,
            _session: &SummarySession,
            _conversation: &Conversation,
            _emit: &Emitter,
        ) -> anyhow::Result<OperationResult<SummaryEffect>> {
            applied([
                SummaryEffect::record_usage(usage()),
                SummaryEffect::from(Message::assistant().with_text("Fallible public summary.")),
            ])
        }
    }
    let runtime = runtime(Observer::default());
    let initial = runtime.load("compaction-summary").await?;
    let stop = SummaryStop {
        guidance: "Editable public guidance.".to_string(),
    };
    let (sender, _receiver) = mpsc::channel(1);
    let emit = Emitter::new(sender, CancellationToken::new());
    assert!(matches!(
        stop.run(&initial, &initial.conversation, &emit).await?,
        OperationResult::NotApplicable
    ));
    assert!(stop.inference_tools(&initial).await?.is_empty());
    assert_eq!(
        stop.prompt_parts(&initial, &initial.conversation).await?,
        vec![(
            "summary-guidance".to_string(),
            "Editable public guidance.".to_string()
        )]
    );
    let machine = StateMachine::new(
        vec![
            Step::Operation(Arc::new(stop)),
            Step::Operation(Arc::new(FixtureBatch)),
        ],
        CancellationToken::new(),
    );
    let saved = machine.run(&runtime, initial.id(), &emit).await?;
    assert_eq!(saved.conversation.len(), 2);
    assert!(saved.conversation.messages()[1].id.is_some());
    assert_eq!(saved.usage.len(), 1);
    assert_eq!(
        serde_json::to_value(&saved.usage[0])?,
        serde_json::to_value(usage())?
    );
    // The original runtime also counts the final empty yielded batch.
    assert_eq!(saved.batches, 2);
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn inference_observer_reenters_conversion_chat_span_and_saved_keeps_parent(
) -> anyhow::Result<()> {
    fn inference(message: &Message) {
        assert_eq!(
            tracing::Span::current()
                .metadata()
                .expect("enabled span")
                .name(),
            "chat"
        );
        assert_eq!(message.as_concat_text(), "Exact assembled output.");
        assert!(message.id.is_some());
        tracing::info!("public.summary.inference" = message.as_concat_text());
    }
    fn saved(message: &Message) {
        tracing::info!("public.summary.saved" = message.as_concat_text());
        assert_eq!(
            tracing::Span::current()
                .metadata()
                .expect("enabled span")
                .name(),
            "summary-parent"
        );
        assert_eq!(message.as_concat_text(), "Exact assembled output.");
    }
    // No global subscriber, observer storage or task-local callback is involved.
    let _default = tracing::subscriber::set_default(tracing_subscriber::registry());
    let runtime = runtime(Observer {
        inference,
        saved,
        ..Observer::default()
    });
    let initial = runtime.load("compaction-summary").await?;
    let output = Message::assistant()
        .with_thinking("Retained diagnostic reasoning.", "signature")
        .with_text("Exact assembled output.");
    let output_before = serde_json::to_value(&output)?;
    let chat = tracing::info_span!("chat");
    let effect = chat.in_scope(|| SummaryEffect::from(output));
    drop(chat);
    let mut effects = [effect, SummaryEffect::record_usage(usage())];
    for effect in &mut effects {
        effect.ensure_message_ids();
    }
    let (sender, _receiver) = mpsc::channel(1);
    let emit = Emitter::new(sender, CancellationToken::new());
    runtime
        .apply_effects(&initial, &mut effects, &emit)
        .instrument(tracing::info_span!("summary-parent"))
        .await?;
    let state = runtime.load("compaction-summary").await?;
    let mut saved_output = serde_json::to_value(&state.conversation.messages()[1])?;
    // ID allocation is the sole permitted change before storage.
    saved_output["id"] = output_before["id"].clone();
    assert_eq!(saved_output, output_before);
    assert_eq!(state.usage.len(), 1);
    Ok(())
}

#[tokio::test]
async fn request_preparer_observes_exact_joined_input_without_added_messages() -> anyhow::Result<()>
{
    fn input(messages: &[Message], prompt: &str, tools: &[Tool]) {
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].as_concat_text(), "Public historical fixture.");
        assert_eq!(prompt, "first\n\nsecond");
        assert!(tools.is_empty());
    }
    let runtime = runtime(Observer::default());
    let state = runtime.load("compaction-summary").await?;
    let preparer = SummaryPreparer {
        observer: Observer {
            input,
            ..Observer::default()
        },
    };
    let prepared = preparer
        .prepare(
            &state,
            &state.conversation,
            InferenceInput {
                prompt_parts: vec![
                    ("first-part".to_string(), "first".to_string()),
                    ("second-part".to_string(), "second".to_string()),
                ],
                ..Default::default()
            },
        )
        .await?;
    assert_eq!(prepared.system_prompt, "first\n\nsecond");
    assert!(prepared.tools.is_empty());
    assert!(prepared.additional_messages.is_empty());
    Ok(())
}

#[tokio::test]
async fn stream_text_protocol_and_token_limits_cancel_without_provider() -> anyhow::Result<()> {
    let mut budget = StreamBudget::default();
    assert!(!budget.observe(
        &AgentEvent::Message(Message::assistant().with_text("x".repeat(SUMMARY_BYTES))),
        SUMMARY_BYTES
    )?);
    assert!(budget.observe(
        &AgentEvent::Message(Message::assistant().with_text("x")),
        SUMMARY_BYTES
    )?);
    let protocol = AgentEvent::Message(
        Message::assistant().with_thinking("x".repeat(STREAM_PROTOCOL_BYTES), "public-signature"),
    );
    let mut protocol_budget = StreamBudget::default();
    assert!(protocol_budget.observe(&protocol, SUMMARY_BYTES)?);
    assert_eq!(protocol_budget.text_bytes, 0);
    let mut truncated = Message::assistant().with_text("Truncated public fixture.");
    truncated.metadata.output_token_limit_reached = true;
    for (event, expected_bytes, expected_tokens, expected_partial) in [
        (protocol, true, false, 0),
        (
            AgentEvent::Message(Message::assistant().with_text("x".repeat(SUMMARY_BYTES + 1))),
            true,
            false,
            0,
        ),
        (AgentEvent::Message(truncated.clone()), false, true, 1),
    ] {
        let cancel = CancellationToken::new();
        let finished = CancellationToken::new();
        let (sender, mut receiver) = mpsc::channel(1);
        sender.send(event).await?;
        drop(sender);
        let (exceeded, tokens, partial) = drain_events(&mut receiver, &cancel, &finished).await?;
        assert!(cancel.is_cancelled());
        assert_eq!(exceeded, expected_bytes);
        assert_eq!(tokens, expected_tokens);
        assert_eq!(partial.len(), expected_partial);
        if expected_tokens {
            assert_eq!(
                serde_json::to_value(&partial[0])?,
                serde_json::to_value(&truncated)?
            );
        }
    }
    Ok(())
}

#[tokio::test]
async fn finished_drain_closes_queue_even_with_a_retained_sender() -> anyhow::Result<()> {
    let (sender, mut receiver) = mpsc::channel(2);
    let first = Message::assistant().with_text("First public delta.");
    let second = Message::assistant().with_text("Second public delta.");
    sender.send(AgentEvent::Message(first.clone())).await?;
    sender.send(AgentEvent::Message(second.clone())).await?;
    let cancel = CancellationToken::new();
    let finished = CancellationToken::new();
    finished.cancel();
    let (bytes, tokens, partial) = tokio::time::timeout(
        Duration::from_secs(1),
        drain_events(&mut receiver, &cancel, &finished),
    )
    .await??;
    assert!(!bytes && !tokens && !cancel.is_cancelled());
    assert_eq!(
        serde_json::to_value(partial)?,
        serde_json::to_value([first, second])?
    );
    assert!(sender.is_closed());
    Ok(())
}

#[tokio::test]
async fn stream_partial_message_count_is_bounded_without_changing_messages() -> anyhow::Result<()> {
    let (sender, mut receiver) = mpsc::channel(32);
    let cancel = CancellationToken::new();
    let finished = CancellationToken::new();
    let producer = async {
        for _index in 0..SUMMARY_PARTIAL_MESSAGES + 1 {
            // Empty text keeps this explicit count fixture below both byte limits.
            sender
                .send(AgentEvent::Message(Message::assistant()))
                .await?;
        }
        drop(sender);
        Ok::<(), anyhow::Error>(())
    };
    let (produced, drained) =
        tokio::join!(producer, drain_events(&mut receiver, &cancel, &finished));
    produced?;
    let (bytes, tokens, partial) = drained?;
    assert!(!bytes && !tokens && !cancel.is_cancelled());
    assert_eq!(partial.len(), SUMMARY_PARTIAL_MESSAGES);
    assert!(partial.iter().all(|message| message.content.is_empty()));
    Ok(())
}

#[tokio::test]
async fn cooperative_deadline_preserves_usage_and_hard_abort_is_bounded() -> anyhow::Result<()> {
    let cancel = CancellationToken::new();
    let partial = Mutex::new(Vec::new());
    let future = async {
        let received_usage = usage();
        cancel.cancelled().await;
        partial.lock().await.push(received_usage);
        Ok(())
    };
    let outcome = cooperative_run(
        future,
        &cancel,
        Duration::from_millis(1),
        Duration::from_millis(20),
    )
    .await;
    assert!(outcome.deadline_reached && !outcome.hard_abort);
    assert_eq!(partial.lock().await[0].usage.output_tokens, Some(17));
    let cancel = CancellationToken::new();
    let outcome = cooperative_run(
        std::future::pending::<anyhow::Result<()>>(),
        &cancel,
        Duration::from_millis(1),
        Duration::from_millis(1),
    )
    .await;
    assert!(outcome.deadline_reached && outcome.hard_abort && outcome.result.is_none());
    Ok(())
}

#[tokio::test]
async fn child_cancellation_does_not_cancel_parent_and_cooperative_save_completes(
) -> anyhow::Result<()> {
    let parent = CancellationToken::new();
    let child = parent.child_token();
    let partial = Mutex::new(Vec::new());
    let future = async {
        child.cancel();
        child.cancelled().await;
        partial.lock().await.push(usage());
        Ok(())
    };
    let outcome = cooperative_run(
        future,
        &child,
        Duration::from_secs(1),
        Duration::from_millis(20),
    )
    .await;
    assert!(!outcome.deadline_reached && !outcome.hard_abort);
    assert!(matches!(outcome.result, Some(Ok(()))));
    assert!(child.is_cancelled() && !parent.is_cancelled());
    assert_eq!(partial.lock().await.len(), 1);
    Ok(())
}

#[test]
fn summary_projection_excludes_metadata_and_reasoning_without_mutating_history(
) -> anyhow::Result<()> {
    let mut learner = Message::user()
        .with_text("I remain unsure about this public example.")
        .with_generated_id_if_missing();
    learner
        .metadata
        .set_operation_note("public-fixture", "internal", json!("metadata-only"));
    let tutor = Message::assistant()
        .with_thinking("reasoning-only", "signature-only")
        .with_text("Public ordinary explanation.");
    let source = vec![learner, tutor];
    let original = serde_json::to_value(&source)?;
    let ids = vec![source[0].id.clone().expect("fixture ID")];
    let (prompt, conversation) = summary_request(&source, &ids, "Editable summary instruction.")?;
    assert!(prompt.ends_with("Editable summary instruction."));
    assert_eq!(conversation.len(), 1);
    let projected = conversation.messages()[0].as_concat_text();
    assert!(projected.contains("genuine learner reply"));
    assert!(projected.contains("Public ordinary explanation."));
    for excluded in ["metadata-only", "reasoning-only", "signature-only"] {
        assert!(!projected.contains(excluded));
    }
    assert_eq!(serde_json::to_value(&source)?, original);
    Ok(())
}

#[test]
fn summary_output_allows_diagnostic_reasoning_but_requires_ordinary_assistant_text() {
    let mut ordinary = Message::assistant()
        .with_thinking("Retained private diagnostic.", "signature")
        .with_text("Fallible public summary.");
    ordinary.content.push(MessageContent::RedactedThinking(
        RedactedThinkingContentBlock {
            data: "opaque diagnostic".to_string(),
        },
    ));
    assert!(valid_summary_output(
        std::slice::from_ref(&ordinary),
        &ordinary.as_concat_text()
    ));
    ordinary.metadata.output_token_limit_reached = true;
    assert!(!valid_summary_output(
        std::slice::from_ref(&ordinary),
        &ordinary.as_concat_text()
    ));
    for invalid in [
        Message::user().with_text("Wrong speaker."),
        Message::assistant().with_text("   "),
        Message::assistant().with_thinking("Reasoning only.", ""),
        Message::assistant()
            .with_tool_request("unexpected", Ok(CallToolRequestParams::new("unavailable"))),
        Message::assistant().with_text(
            "The model returned an empty response. Please resend your message to continue.",
        ),
        Message::assistant().with_text("x".repeat(SUMMARY_BYTES + 1)),
    ] {
        assert!(!valid_summary_output(
            std::slice::from_ref(&invalid),
            &invalid.as_concat_text()
        ));
    }
    let boundary = Message::assistant().with_text("é".repeat(SUMMARY_BYTES / 2));
    assert!(valid_summary_output(
        std::slice::from_ref(&boundary),
        &boundary.as_concat_text()
    ));
}
