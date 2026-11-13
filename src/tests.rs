// Lesson 13

fn genuine_fixture(ids: &mut Vec<String>, text: &str) -> Message {
    let message = Message::user()
        .with_text(text)
        .with_generated_id_if_missing();
    ids.push(message.id.clone().expect("fixture ID"));
    message
}

#[test]
fn declared_provider_overrides_cannot_defeat_model_output_limit() {
    let error = validate_local_tutor(&json!({"engine":"openai", "base_url":"http://127.0.0.1:8080", "models":[{"name":"local", "request_params":{"max_tokens":999999}}]}), TUTOR_MODEL_NAME_BYTES).expect_err("override rejected before provider creation");
    assert!(error.to_string().contains("request_params"));
    assert_eq!(TUTOR_MAX_TOKENS, 4096);
}

#[tokio::test]
async fn approval_capacity_uses_live_ledger_and_suppresses_duplicate_batch_decisions(
) -> anyhow::Result<()> {
    let runtime = ChatRuntime::new(SESSION_ID, LessonContext::load()?);
    {
        let mut sessions = runtime.sessions.lock().await;
        let stored = sessions.get_mut(SESSION_ID).expect("fixture");
        for index in 0..255 {
            stored
                .settled_approvals
                .insert(format!("settled-{}", index), false);
        }
    }
    let current = runtime.load(SESSION_ID).await?;
    assert!(approval_pending(&current, "first")?);
    {
        let mut sessions = runtime.sessions.lock().await;
        sessions
            .get_mut(SESSION_ID)
            .expect("fixture")
            .settled_approvals
            .insert("first".to_string(), false);
    }
    let current = runtime.load(SESSION_ID).await?;
    assert!(!approval_pending(&current, "first")?);
    assert!(approval_pending(&current, "second").is_err());
    assert_eq!(current.settled_approvals.len(), 256);
    Ok(())
}

#[test]
fn approval_keys_require_stable_kickoff_ids_and_encode_both_components() -> anyhow::Result<()> {
    assert!(approval_key(&Message::user().with_text("No ID"), "request").is_err());
    let mut first = Message::user().with_text("First kickoff");
    first.id = Some("kickoff|request".to_string());
    let mut second = Message::user().with_text("Second kickoff");
    second.id = Some("kickoff".to_string());
    let first_key = approval_key(&first, "tail")?;
    let second_key = approval_key(&second, "request|tail")?;
    assert_ne!(first_key, second_key);
    assert_eq!(
        serde_json::from_str::<(String, String)>(&first_key)?,
        ("kickoff|request".to_string(), "tail".to_string())
    );
    assert_eq!(first_key, approval_key(&first, "tail")?);
    Ok(())
}

#[tokio::test]
async fn reused_record_request_ids_validate_new_proposals_instead_of_replaying_old_decisions(
) -> anyhow::Result<()> {
    let record_before = read_record(&record_path())?;
    for approved in [false, true] {
        for escalation in [false, true] {
            let runtime = ChatRuntime::seeded();
            let (first_key, second_key) = {
                let mut sessions = runtime.sessions.lock().await;
                let stored = sessions.get_mut(SESSION_ID).expect("fixture Session");
                stored.lesson.persistence_enabled = true;
                let first =
                    genuine_fixture(&mut stored.genuine_reply_ids, "I want to learn orders.");
                let first_key = approval_key(&first, "reused-record")?;
                stored.conversation.push(first);
                stored
                    .conversation
                    .push(Message::assistant().with_tool_request(
                        "reused-record",
                        Ok(CallToolRequestParams::new(RECORD_TOOL)),
                    ));
                stored.conversation.push(Message::user().with_tool_response(
                    "reused-record",
                    Ok(CallToolResult::success(vec![ContentBlock::text(
                        "Prepared",
                    )])),
                ));
                stored
                    .conversation
                    .push(Message::assistant().with_text("First answer."));
                stored.settled_approvals.insert(first_key.clone(), approved);
                let second = if escalation {
                    Message::user()
                        .with_text(GROUNDING_ESCALATION_MESSAGE)
                        .with_generated_id_if_missing()
                } else {
                    genuine_fixture(&mut stored.genuine_reply_ids, "Explain stop orders.")
                };
                let second_key = approval_key(&second, "reused-record")?;
                stored.conversation.push(second);
                // An invalid new proposal must fail validation, never reach stdin.
                stored
                    .conversation
                    .push(Message::assistant().with_tool_request(
                        "reused-record",
                        Ok(proposal_call(
                            "stated_goals",
                            "New note",
                            "Not an actual learner quote",
                        )),
                    ));
                stored.conversation.push(Message::user().with_tool_response(
                    "reused-record",
                    Ok(CallToolResult::success(vec![ContentBlock::text(
                        "Prepared",
                    )])),
                ));
                stored
                    .conversation
                    .push(Message::assistant().with_text("Second answer."));
                (first_key, second_key)
            };
            let before = runtime.load(SESSION_ID).await?;
            assert_ne!(first_key, second_key);
            assert!(!approval_pending(&before, &first_key)?);
            assert!(approval_pending(&before, &second_key)?);
            let temporary = OrchestrationDiagnostics::new()?;
            let mut diagnostics = DiagnosticLog::create_at(temporary.path())?;
            let error = approve_record_proposals(&runtime, SESSION_ID, &mut diagnostics)
                .await
                .expect_err("new attempt must independently validate its proposal");
            assert!(error
                .to_string()
                .contains("learner_quote must exactly quote"));
            let after = runtime.load(SESSION_ID).await?;
            assert_eq!(after.settled_approvals, before.settled_approvals);
            assert_eq!(after.revision, before.revision);
            assert!(!fs::read_to_string(temporary.path())?.contains("## Learning note decision"));
            {
                let mut sessions = runtime.sessions.lock().await;
                sessions
                    .get_mut(SESSION_ID)
                    .expect("fixture Session")
                    .settled_approvals
                    .insert(second_key.clone(), false);
            }
            // Once this specific proposal is settled, even invalid arguments are not replayed.
            assert!(!approval_pending(
                &runtime.load(SESSION_ID).await?,
                &second_key
            )?);
            approve_record_proposals(&runtime, SESSION_ID, &mut diagnostics).await?;
        }
    }
    assert_eq!(read_record(&record_path())?, record_before);
    Ok(())
}

#[tokio::test]
async fn prior_success_cannot_authorize_a_reused_record_id_without_current_success(
) -> anyhow::Result<()> {
    for response in ["rejected", "missing", "error"] {
        let runtime = ChatRuntime::seeded();
        {
            let mut sessions = runtime.sessions.lock().await;
            let stored = sessions.get_mut(SESSION_ID).expect("fixture Session");
            stored.lesson.persistence_enabled = true;
            stored.conversation.push(
                Message::assistant().with_tool_request(
                    "reused-record",
                    Ok(CallToolRequestParams::new(RECORD_TOOL)),
                ),
            );
            stored.conversation.push(Message::user().with_tool_response(
                "reused-record",
                Ok(CallToolResult::success(vec![ContentBlock::text(
                    "Prepared",
                )])),
            ));
            stored
                .conversation
                .push(Message::assistant().with_text("Prior answer."));
            let second = genuine_fixture(&mut stored.genuine_reply_ids, "Explain stop orders.");
            stored.conversation.push(second);
            // Missing arguments expose any accidental match against the prior successful result.
            stored.conversation.push(
                Message::assistant().with_tool_request(
                    "reused-record",
                    Ok(CallToolRequestParams::new(RECORD_TOOL)),
                ),
            );
            if response != "missing" {
                let result = if response == "rejected" {
                    Ok(CallToolResult::error(vec![ContentBlock::text("Rejected")]))
                } else {
                    Err(ErrorData::invalid_params("Rejected", None))
                };
                stored
                    .conversation
                    .push(Message::user().with_tool_response("reused-record", result));
            }
            stored
                .conversation
                .push(Message::assistant().with_text("Current answer."));
        }
        let before = runtime.load(SESSION_ID).await?;
        let temporary = OrchestrationDiagnostics::new()?;
        let mut diagnostics = DiagnosticLog::create_at(temporary.path())?;
        approve_record_proposals(&runtime, SESSION_ID, &mut diagnostics).await?;
        let after = runtime.load(SESSION_ID).await?;
        assert_eq!(after.settled_approvals, before.settled_approvals);
        assert_eq!(after.revision, before.revision);
        assert!(!fs::read_to_string(temporary.path())?.contains("## Learning note decision"));
    }
    Ok(())
}

#[tokio::test]
async fn cooperative_deadline_preserves_received_usage_and_hard_abort_is_bounded(
) -> anyhow::Result<()> {
    let cancel = CancellationToken::new();
    let partial = Mutex::new(Vec::new());
    let future = async {
        // Represents the SDK's received usage accumulator before cancellation.
        let received_usage = ProviderUsage::new(
            "test-model".to_string(),
            goose_providers::conversation::token_usage::Usage {
                output_tokens: Some(17),
                ..Default::default()
            },
        );
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
async fn stream_text_protocol_and_token_limits_cancel_and_prevent_approval() -> anyhow::Result<()> {
    let mut budget = StreamBudget::default();
    let event = AgentEvent::Message(Message::assistant().with_text("x".repeat(TUTOR_TEXT_BYTES)));
    assert!(!budget.observe(&event, TUTOR_TEXT_BYTES)?);
    assert!(budget.observe(
        &AgentEvent::Message(Message::assistant().with_text("x")),
        TUTOR_TEXT_BYTES
    )?);
    let mut budget = StreamBudget::default();
    let event = AgentEvent::Message(Message::assistant().with_tool_request(
        "huge",
        Ok(
            CallToolRequestParams::new(TOOL_NAME).with_arguments(rmcp::model::object(
                json!({"data": "x".repeat(STREAM_PROTOCOL_BYTES)}),
            )),
        ),
    ));
    assert!(budget.observe(&event, TUTOR_TEXT_BYTES)?);
    assert_eq!(budget.text_bytes, 0);
    for event in [
        event,
        AgentEvent::Message(Message::assistant().with_text("x".repeat(TUTOR_TEXT_BYTES + 1))),
    ] {
        let cancel = CancellationToken::new();
        let (sender, receiver) = mpsc::channel(1);
        sender.send(event).await?;
        drop(sender);
        let mut output = Vec::new();
        let summary = display_events_bounded(receiver, &mut output, false, &cancel).await?;
        assert!(cancel.is_cancelled() && summary.output_exceeded);
        assert!(output.is_empty());
        let state = ChatRuntime::new(SESSION_ID, LessonContext::load()?)
            .load(SESSION_ID)
            .await?;
        assert!(inspect_turn(&state, &CitationRegistry::new(), &summary).is_err());
    }
    let mut message = Message::assistant().with_text("truncated");
    message.metadata.output_token_limit_reached = true;
    let cancel = CancellationToken::new();
    let (sender, receiver) = mpsc::channel(1);
    sender.send(AgentEvent::Message(message)).await?;
    drop(sender);
    let summary = display_events_bounded(receiver, &mut Vec::new(), false, &cancel).await?;
    assert!(cancel.is_cancelled() && summary.output_token_limit_reached);
    Ok(())
}

// Synthetic public evidence exercises application state without substituting for live retrieval.
fn grounding_fixture(id: &str, similarity: f64, excerpt: &str) -> retrieval::Evidence {
    retrieval::Evidence {
        citation_id: format!("source:{}", id),
        chunk_id: format!("chunk-{}", id),
        document_id: format!("document-{}", id),
        source_file: format!("public-{}.pdf", id),
        source_sha256: "synthetic-source-sha".to_string(),
        content_sha256: "synthetic-content-sha".to_string(),
        page_start: 3,
        page_end: 4,
        raw_fragments: Vec::new(),
        review_status: "draft".to_string(),
        similarity,
        excerpt: excerpt.to_string(),
        excerpt_truncated: false,
    }
}

fn forced_grounding_policy() -> GroundingPolicy {
    GroundingPolicy {
        min_similarity: GROUNDING_MIN_SIMILARITY,
        max_statements: GROUNDING_MAX_STATEMENTS,
        token_budget: GROUNDING_TOKEN_BUDGET,
    }
}

fn grounding_response(evidence: Vec<retrieval::Evidence>) -> retrieval::SearchResponse {
    retrieval::SearchResponse {
        collection: "public-fixture".to_string(),
        model: "public".to_string(),
        revision: "public".to_string(),
        profile: "public".to_string(),
        status: "ok".to_string(),
        notice: "Synthetic public evidence".to_string(),
        evidence,
        used_tokens: 0,
    }
}

#[tokio::test]
async fn forced_grounding_is_replaced_per_turn_without_leaking_evidence_into_history(
) -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let kickoff = runtime.load(SESSION_ID).await?;
    assert_eq!(kickoff.turn_grounding.status, "none");
    assert!(kickoff.turn_grounding.evidence.is_empty());
    let response = grounding_response(vec![
        grounding_fixture("previous", 0.95, "First-turn public excerpt."),
        grounding_fixture("second", 0.90, "Another public excerpt."),
        grounding_fixture("third", 0.80, "Third public excerpt."),
        grounding_fixture("over-cap", 0.70, "Not selected."),
        grounding_fixture("below-threshold", 0.10, "Not selected either."),
    ]);
    let policy = forced_grounding_policy();
    let first_grounding = rag::select_grounding("First question", &response, &policy);
    runtime
        .append_grounded_user(SESSION_ID, "First question", first_grounding)
        .await?;
    let first = runtime.load(SESSION_ID).await?;
    assert_eq!(
        first.turn_grounding.evidence.len(),
        GROUNDING_MAX_STATEMENTS
    );
    assert!(first
        .turn_grounding
        .evidence
        .iter()
        .all(|passage| passage.similarity >= GROUNDING_MIN_SIMILARITY));
    let response = grounding_response(vec![grounding_fixture(
        "current",
        GROUNDING_MIN_SIMILARITY,
        "Second-turn public excerpt.",
    )]);
    let grounding = rag::select_grounding("Second question", &response, &policy);
    runtime
        .append_grounded_user(SESSION_ID, "Second question", grounding)
        .await?;
    let second = runtime.load(SESSION_ID).await?;
    assert_eq!(second.id, first.id);
    assert_eq!(second.revision, first.revision + 1);
    assert_eq!(second.turn_grounding.query, "Second question");
    assert_eq!(second.turn_grounding.evidence.len(), 1);
    assert_eq!(
        second.turn_grounding.evidence[0].citation_id,
        "source:current"
    );
    assert_eq!(
        &second.conversation.messages()[..first.conversation.len()],
        first.conversation.messages()
    );
    let parts = safe_tools()
        .prompt_parts(&second, &second.conversation)
        .await?;
    assert!(parts[0].1.contains("Second-turn public excerpt."));
    assert!(!parts[0].1.contains("First-turn public excerpt."));
    let history = serde_json::to_string(second.conversation.messages())?;
    assert!(!history.contains("public excerpt"));
    assert!(!history.contains("source:current"));
    assert!(!history.contains("source:previous"));
    assert_eq!(second.genuine_reply_ids.len(), 2);
    let empty = rag::select_grounding(
        "Unrelated question",
        &grounding_response(Vec::new()),
        &policy,
    );
    runtime
        .append_grounded_user(SESSION_ID, "Unrelated question", empty)
        .await?;
    let third = runtime.load(SESSION_ID).await?;
    assert_eq!(third.turn_grounding.status, "no_evidence");
    assert!(third.turn_grounding.evidence.is_empty());
    assert!(safe_tools()
        .prompt_parts(&third, &third.conversation)
        .await?
        .iter()
        .all(|(name, _body)| name != "source-grounding"));
    assert_eq!(
        first.turn_grounding.evidence.len(),
        GROUNDING_MAX_STATEMENTS
    );
    Ok(())
}

#[tokio::test]
#[ignore = "requires actual local reader/embedding/tutor services"]
async fn live_forced_grounding_obeys_budget_and_current_turn_selection() -> anyhow::Result<()> {
    let retrieval = match retrieval::Retrieval::from_env_with(retrieval_defaults()).await {
        Ok(retrieval) => retrieval,
        Err(failure) => return Err(anyhow::anyhow!(failure)),
    };
    let result = async {
        let runtime = ChatRuntime::seeded();
        let policy = forced_grounding_policy();
        for query in [
            "How does liquidity affect the bid-ask spread?",
            "Why keep a trading journal to understand trading psychology?",
        ] {
            let preparation = rag::prepare_grounding(&retrieval, query, &policy).await;
            let response = match preparation.response {
                Ok(response) => response,
                Err(failure) => return Err(anyhow::anyhow!(failure)),
            };
            assert!(response.used_tokens <= GROUNDING_TOKEN_BUDGET);
            assert!(!response.evidence.is_empty());
            let expected = rag::select_grounding(query, &response, &policy);
            assert!(!preparation.grounding.evidence.is_empty());
            assert!(preparation.grounding.evidence.len() <= GROUNDING_MAX_STATEMENTS);
            assert!(preparation
                .grounding
                .evidence
                .iter()
                .all(|passage| passage.similarity >= GROUNDING_MIN_SIMILARITY));
            assert_eq!(
                serde_json::to_value(&preparation.grounding.evidence)?,
                serde_json::to_value(&expected.evidence)?
            );
            runtime
                .append_grounded_user(SESSION_ID, query, preparation.grounding)
                .await?;
            let current = runtime.load(SESSION_ID).await?;
            assert_eq!(current.turn_grounding.query, query);
            assert_eq!(
                serde_json::to_value(&current.turn_grounding.evidence)?,
                serde_json::to_value(&expected.evidence)?
            );
            assert_eq!(
                current
                    .conversation
                    .last()
                    .expect("learner input")
                    .as_concat_text(),
                query
            );
            for message in current.conversation.messages() {
                assert!(message.get_tool_request_ids().is_empty());
                assert!(message.get_tool_response_ids().is_empty());
            }
        }
        Ok(())
    }
    .await;
    retrieval.shutdown().await;
    result
}

#[tokio::test]
async fn search_is_not_advertised_and_receives_a_correlated_unknown_tool_error(
) -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session.lesson.persistence_enabled = true;
    let tools = safe_tools();
    let definitions = tools.inference_tools(&session).await?;
    assert_eq!(definitions.len(), 2);
    assert!(definitions
        .iter()
        .all(|tool| tool.name.as_ref() != "search_educational_passages"));
    session
        .conversation
        .push(Message::assistant().with_tool_request(
            "removed-search",
            Ok(CallToolRequestParams::new("search_educational_passages")),
        ));
    let (sender, mut receiver) = mpsc::channel(2);
    let emit = Emitter::new(sender, CancellationToken::new());
    let effects = applied_effects(tools.run(&session, &session.conversation, &emit).await?)?;
    let mut responses = Vec::new();
    for effect in effects {
        if let ChatEffect::AppendMessage(message) = effect {
            assert_eq!(message.role, rmcp::model::Role::User);
            for block in &message.content {
                if let Some(response) = block.as_tool_response() {
                    assert_eq!(response.id, "removed-search");
                    let failure = response
                        .tool_result
                        .as_ref()
                        .expect_err("unknown search rejected");
                    assert!(failure.message.contains("unknown or unavailable tool"));
                    responses.push(response.id.clone());
                }
            }
            session.conversation.push(message);
        }
    }
    assert_eq!(responses, vec!["removed-search"]);
    assert!(matches!(
        tools.run(&session, &session.conversation, &emit).await?,
        OperationResult::NotApplicable
    ));
    drop(emit);
    assert!(receiver.recv().await.is_some());
    assert!(receiver.recv().await.is_none());
    Ok(())
}

#[tokio::test]
async fn budget_preparer_captures_exact_grounded_prompt_without_mutating_history(
) -> anyhow::Result<()> {
    use goose_agent::inference::InferenceRequestPreparer;
    use goose_agent::operation::InferenceInput;
    let runtime = ChatRuntime::seeded();
    let kickoff = runtime.load(SESSION_ID).await?;
    let captures = PromptCaptures::default();
    let preparer = compaction::BudgetPreparer {
        captures: captures.clone(),
    };
    let kickoff_parts = safe_tools()
        .prompt_parts(&kickoff, &kickoff.conversation)
        .await?;
    assert_eq!(kickoff.turn_grounding.status, "none");
    assert!(kickoff.turn_grounding.evidence.is_empty());
    assert!(kickoff_parts
        .iter()
        .all(|(name, _body)| name != "source-grounding"));
    let input = InferenceInput {
        prompt_parts: kickoff_parts,
        tools: safe_tools().inference_tools(&kickoff).await?,
        moim_parts: Vec::new(),
    };
    let prepared = preparer
        .prepare(&kickoff, &kickoff.conversation, input)
        .await?;
    assert!(prepared.system_prompt.starts_with(&kickoff.lesson.contract));
    assert!(!prepared.system_prompt.contains(GROUNDING_CONTRACT));
    assert_eq!(captures.drain().await, vec![prepared.system_prompt]);
    let grounding = rag::select_grounding(
        "Learner question",
        &grounding_response(vec![grounding_fixture(
            "capture",
            0.95,
            "Public attached grounding excerpt.",
        )]),
        &forced_grounding_policy(),
    );
    runtime
        .append_grounded_user(SESSION_ID, "Learner question", grounding.clone())
        .await?;
    let session = runtime.load(SESSION_ID).await?;
    let before = serde_json::to_value(session.conversation.messages())?;
    let parts = safe_tools()
        .prompt_parts(&session, &session.conversation)
        .await?;
    assert_eq!(parts[0].0, "source-grounding");
    assert!(parts[0].1.starts_with(GROUNDING_CONTRACT));
    assert_eq!(
        parts[0],
        grounding
            .prompt_part(GROUNDING_CONTRACT)
            .expect("grounding part")
    );
    let expected = parts
        .iter()
        .map(|(_name, body)| body.as_str())
        .collect::<Vec<_>>()
        .join("\n\n");
    let definitions = safe_tools().inference_tools(&session).await?;
    let prepared = preparer
        .prepare(
            &session,
            &session.conversation,
            InferenceInput {
                prompt_parts: parts,
                tools: definitions.clone(),
                moim_parts: Vec::new(),
            },
        )
        .await?;
    assert_eq!(prepared.system_prompt, expected);
    assert_eq!(prepared.tools, definitions);
    assert!(prepared.additional_messages.is_empty());
    assert_eq!(captures.drain().await, vec![expected]);
    assert!(captures.drain().await.is_empty());
    let after = runtime.load(SESSION_ID).await?;
    assert_eq!(serde_json::to_value(after.conversation.messages())?, before);
    assert_eq!(after.revision, session.revision);
    assert_eq!(after.genuine_reply_ids, session.genuine_reply_ids);
    let history = serde_json::to_string(after.conversation.messages())?;
    assert!(!history.contains("Public attached grounding excerpt."));
    assert!(!history.contains(GROUNDING_CONTRACT));
    assert_eq!(after.conversation.len(), kickoff.conversation.len() + 1);
    Ok(())
}

#[tokio::test]
async fn escalation_append_preserves_genuine_provenance_and_current_evidence() -> anyhow::Result<()>
{
    let runtime = ChatRuntime::seeded();
    let grounding = rag::select_grounding(
        "Learn orders",
        &grounding_response(vec![grounding_fixture(
            "escalation",
            0.90,
            "Public source excerpt.",
        )]),
        &forced_grounding_policy(),
    );
    runtime
        .append_grounded_user(SESSION_ID, "I want to learn orders.", grounding)
        .await?;
    {
        let mut sessions = runtime.sessions.lock().await;
        let stored = sessions.get_mut(SESSION_ID).expect("fixture");
        stored.lesson.persistence_enabled = true;
        stored.applied_passes = MAX_APPLIED_PASSES;
        stored.halt_reason = Some("prior attempt ended".to_string());
        stored
            .settled_approvals
            .insert("settled".to_string(), false);
    }
    let before = runtime.load(SESSION_ID).await?;
    runtime
        .append_application(SESSION_ID, GROUNDING_ESCALATION_MESSAGE)
        .await?;
    let after = runtime.load(SESSION_ID).await?;
    assert_eq!(after.id, before.id);
    assert_eq!(after.genuine_reply_ids, before.genuine_reply_ids);
    assert_eq!(after.revision, before.revision + 1);
    assert_eq!(after.applied_passes, 0);
    assert_eq!(after.halt_reason, None);
    assert_eq!(after.settled_approvals, before.settled_approvals);
    assert_eq!(
        after.turn_grounding.grounding_summary(),
        before.turn_grounding.grounding_summary()
    );
    assert_eq!(
        serde_json::to_value(&after.turn_grounding.evidence)?,
        serde_json::to_value(&before.turn_grounding.evidence)?
    );
    assert_eq!(
        &after.conversation.messages()[..before.conversation.len()],
        before.conversation.messages()
    );
    let application = after.conversation.last().expect("application reminder");
    assert_eq!(application.role, rmcp::model::Role::User);
    assert_eq!(application.as_concat_text(), GROUNDING_ESCALATION_MESSAGE);
    assert!(!after
        .genuine_reply_ids
        .contains(application.id.as_ref().expect("application ID")));
    assert!(parse_record_proposal(
        &proposal_call(
            "learner_evidence",
            "Discussed excerpts",
            "source excerpts supplied for this turn"
        ),
        &after
    )
    .is_err());
    assert!(parse_record_proposal(
        &proposal_call("stated_goals", "Learn orders", "learn orders"),
        &after
    )
    .is_ok());
    Ok(())
}

#[tokio::test]
async fn retrieved_quotes_never_become_learner_proposal_evidence() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session.lesson.persistence_enabled = true;
    session.turn_grounding = TurnGrounding {
        status: "grounded".to_string(),
        query: "Journal question".to_string(),
        threshold: GROUNDING_MIN_SIMILARITY,
        evidence: vec![grounding_fixture(
            "journal",
            0.90,
            "Public synthetic source says keep a journal.",
        )],
        notice: None,
    };
    let proposal = proposal_call("learner_evidence", "Keeps a journal", "keep a journal");
    assert!(parse_record_proposal(&proposal, &session).is_err());
    session
        .conversation
        .push(Message::user().with_tool_response(
            "search-id",
            Ok(CallToolResult::success(vec![ContentBlock::text(
                "Public synthetic source says keep a journal.",
            )])),
        ));
    assert!(parse_record_proposal(&proposal, &session).is_err());
    session.conversation.push(genuine_fixture(
        &mut session.genuine_reply_ids,
        "I want to keep a journal.",
    ));
    assert!(parse_record_proposal(&proposal, &session).is_ok());
    Ok(())
}

#[test]
fn streaming_preserves_source_markers() {
    let mut prose = TerminalProse::default();
    let mut text = prose.fragment("Evidence [sou");
    text.push_str(&prose.fragment("rce:public-id]"));
    assert_eq!(text, "Evidence [source:public-id]");
}

#[test]
fn tutor_endpoint_is_literal_loopback_without_url_secrets() {
    // Test invalid addresses before ambient-proxy checks; never echo the URL.
    for endpoint in [
        "https://example.org",
        "http://127.1:8080",
        "http://127.0.0.1.evil:8080",
        "http://user:secret@127.0.0.1:8080",
        "http://127.0.0.1:8080?key=secret",
    ] {
        let error = validate_local_tutor(
            &json!({"engine":"openai","base_url":endpoint}),
            TUTOR_MODEL_NAME_BYTES,
        )
        .unwrap_err()
        .to_string();
        assert!(!error.contains("secret"));
        assert!(!error.contains(endpoint));
    }
}

fn proposal_call(category: &str, note: &str, quote: &str) -> CallToolRequestParams {
    let arguments = json!({"category": category, "note": note, "learner_quote": quote});
    let arguments = match arguments {
        Value::Object(arguments) => arguments,
        _ => panic!("fixture object"),
    };
    CallToolRequestParams::new(RECORD_TOOL).with_arguments(arguments)
}

#[tokio::test]
async fn proposal_is_only_advertised_with_consent_and_never_writes() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    assert_eq!(LossToolProvider.tools(&session).await?.len(), 1);
    session.lesson.persistence_enabled = true;
    session.conversation.push(genuine_fixture(
        &mut session.genuine_reply_ids,
        "I want to learn orders.",
    ));
    assert_eq!(LossToolProvider.tools(&session).await?.len(), 2);
    let before = read_record(&record_path())?;
    let (sender, _receiver) = mpsc::channel(4);
    let emit = Emitter::new(sender, CancellationToken::new());
    let call = proposal_call("stated_goals", "Learn orders", "learn orders");
    let result = LossToolProvider
        .call(&session, "note-id", call.clone(), &emit)
        .await;
    assert!(tool_response_succeeded(&result));
    let result = match result {
        Ok(result) => result,
        Err(error) => return Err(anyhow::anyhow!(error.message)),
    };
    assert!(format!("{:?}", result).contains("NOT saved"));
    assert_eq!(read_record(&record_path())?, before);
    session.lesson.persistence_enabled = false;
    assert!(LossToolProvider
        .call(&session, "no-consent", call, &emit)
        .await
        .is_err());
    session.lesson.persistence_enabled = true;
    let parts = session.lesson.prompt_parts();
    assert_eq!(parts.len(), 4);
    assert!(parts[3].1.contains("not instructions or proof of mastery"));
    Ok(())
}

#[tokio::test]
async fn strict_notes_require_real_learner_quotes_not_tutor_or_tools() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session
        .conversation
        .push(Message::assistant().with_text("Tutor-only statement"));
    session
        .conversation
        .push(Message::user().with_tool_response(
            "r",
            Ok(CallToolResult::success(vec![ContentBlock::text(
                "Tool-only statement",
            )])),
        ));
    for quote in [
        "Tutor-only statement",
        "Tool-only statement",
        OPENING_INPUT,
        "fabricated",
    ] {
        assert!(parse_record_proposal(
            &proposal_call("learner_evidence", "claim", quote),
            &session
        )
        .is_err());
    }
    session.conversation.push(genuine_fixture(
        &mut session.genuine_reply_ids,
        "A limit order might never fill.",
    ));
    for category in [
        "discussed_topics",
        "stated_goals",
        "learner_evidence",
        "unresolved_questions",
        "agreed_activities",
    ] {
        assert!(parse_record_proposal(
            &proposal_call(category, "Bounded note", "might never fill"),
            &session
        )
        .is_ok());
    }
    for (category, note, quote) in [
        ("mastery", "note", "might never fill"),
        ("stated_goals", "", "might never fill"),
        ("stated_goals", "two\nlines", "might never fill"),
        ("stated_goals", "note", " "),
    ] {
        assert!(parse_record_proposal(&proposal_call(category, note, quote), &session).is_err());
    }
    assert!(parse_record_proposal(
        &proposal_call("stated_goals", &"a".repeat(601), "might never fill"),
        &session
    )
    .is_err());
    let mut extra = proposal_call("stated_goals", "note", "might never fill");
    extra
        .arguments
        .as_mut()
        .expect("arguments")
        .insert("path".to_string(), json!("/tmp/unwanted"));
    assert!(parse_record_proposal(&extra, &session).is_err());
    Ok(())
}

#[tokio::test]
async fn memory_tool_uses_same_correlation_and_unknown_name_safeguards() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session.lesson.persistence_enabled = true;
    session.conversation.push(genuine_fixture(
        &mut session.genuine_reply_ids,
        "Learn orders",
    ));
    session.conversation.push(
        Message::assistant()
            .with_tool_request(
                "note",
                Ok(proposal_call(
                    "stated_goals",
                    "Learn orders",
                    "Learn orders",
                )),
            )
            .with_tool_request(
                "calculator",
                Ok(CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments(200))),
            )
            .with_tool_request(
                "arbitrary-file",
                Ok(CallToolRequestParams::new("write_file")),
            ),
    );
    let (sender, _receiver) = mpsc::channel(8);
    let emit = Emitter::new(sender, CancellationToken::new());
    for _pass in 0..2 {
        let result = safe_tools()
            .run(&session, &session.conversation, &emit)
            .await?;
        for effect in applied_effects(result)? {
            if let ChatEffect::AppendMessage(message) = effect {
                session.conversation.push(message);
            }
        }
    }
    let mut ids = HashSet::new();
    for message in session.conversation.messages() {
        for block in &message.content {
            if let Some(response) = block.as_tool_response() {
                assert!(ids.insert(response.id.clone()));
                if response.id == "arbitrary-file" {
                    assert!(response.tool_result.is_err());
                } else {
                    assert!(tool_response_succeeded(&response.tool_result));
                }
            }
        }
    }
    assert_eq!(
        ids,
        HashSet::from([
            "note".to_string(),
            "calculator".to_string(),
            "arbitrary-file".to_string()
        ])
    );
    Ok(())
}
#[tokio::test]
async fn presentation_output_is_clean_ordered_and_fallible() -> anyhow::Result<()> {
    let (sender, receiver) = mpsc::channel(8);
    let mut reconstructed = request("shown-call", TOOL_NAME, 200);
    reconstructed.metadata.set_operation_note(
        "lesson-display",
        "reconstructed_request",
        json!(true),
    );
    sender.send(AgentEvent::Message(reconstructed)).await?;
    sender
        .send(AgentEvent::Message(Message::user().with_tool_response(
            "shown-call",
            Ok(CallToolResult::success(vec![ContentBlock::text("$100.00")])),
        )))
        .await?;
    sender
        .send(AgentEvent::Message(
            Message::assistant().with_text("Final answer"),
        ))
        .await?;
    drop(sender);
    let mut bytes = Vec::new();
    let summary = display_events_to(receiver, &mut bytes, true).await?;
    assert!(summary.displayed_text);
    assert_eq!(summary.tool_events.len(), 2);
    let output = String::from_utf8(bytes)?;
    let plain = plain_output(&output);
    assert!(plain.contains("Maximum planned loss: $100.00"));
    assert!(!plain.contains("shown-call"));
    assert!(!plain.contains("++++++++"));
    assert!(!plain.contains("→"));
    assert!(plain.contains("Tutor:\nFinal answer\n"));
    assert!(plain.find("Calculating planned loss") < plain.find("Maximum planned loss"));
    assert!(plain.find("Maximum planned loss") < plain.find("Final answer"));
    assert!(output.contains("\x1b[35m"));
    assert!(output.contains("\x1b[32m"));
    assert!(output.contains("\x1b[1;36m"));
    assert!(!output.contains("\x1b[2m"));
    assert!(output.contains("\x1b[0m\nFinal answer"));
    // A broken stream returns an error rather than panicking past partial-state logging.
    struct FailedOutput;
    impl Write for FailedOutput {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::BrokenPipe,
                "test broken output",
            ))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let (sender, receiver) = mpsc::channel(1);
    sender
        .send(AgentEvent::Message(
            Message::assistant().with_text("partial"),
        ))
        .await?;
    drop(sender);
    let mut failed = FailedOutput;
    assert!(display_events_to(receiver, &mut failed, false)
        .await
        .is_err());
    Ok(())
}

#[test]
fn reply_commands_are_application_controls_not_tutor_messages() {
    assert_eq!(reply_action(" /finish "), ReplyAction::Finish);
    assert_eq!(reply_action("/quit"), ReplyAction::Quit);
    assert_eq!(reply_action("  "), ReplyAction::Empty);
    assert_eq!(reply_action("Explain /finish to me"), ReplyAction::Message);
}

#[tokio::test]
async fn selected_context_is_contributed_before_inference_and_stays_fixed() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let before = runtime.load(SESSION_ID).await?;
    let parts = safe_tools()
        .prompt_parts(&before, &before.conversation)
        .await?;
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0].1, before.lesson.contract);
    assert_eq!(parts[1].1, before.lesson.material);
    assert_eq!(parts[2].1, before.lesson.guidance);
    assert!(parts[1].1.contains("## Source attribution"));
    runtime
        .append_user(SESSION_ID, "I want to understand orders")
        .await?;
    let after = runtime.load(SESSION_ID).await?;
    assert_eq!(
        safe_tools()
            .prompt_parts(&after, &after.conversation)
            .await?,
        parts
    );
    // The selected context is separate from conversation turns.
    assert_eq!(after.conversation.len(), 2);
    assert_eq!(
        after.conversation.messages()[0].as_concat_text(),
        OPENING_INPUT
    );
    Ok(())
}

#[tokio::test]
async fn reselecting_starts_fresh_history_and_bounds() -> anyhow::Result<()> {
    let first = ChatRuntime::new("first", LessonContext::load()?);
    first
        .append_user("first", "Learner private history")
        .await?;
    let second = ChatRuntime::new("second", LessonContext::load()?);
    let session = second.load("second").await?;
    assert_eq!(session.id, "second");
    assert_eq!(session.conversation.len(), 1);
    assert_eq!(session.applied_passes, 0);
    assert_eq!(session.halt_reason, None);
    assert!(!session_fields(&session).contains("Learner private history"));
    assert!(second.load("first").await.is_err());
    Ok(())
}

#[tokio::test]
async fn optional_tool_use_does_not_require_a_calculation_for_every_turn() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session
        .conversation
        .push(Message::assistant().with_text("What would you like to understand?"));
    let summary = EventSummary {
        displayed_text: true,
        ..EventSummary::default()
    };
    assert!(inspect_turn(&session, &CitationRegistry::new(), &summary)?.is_empty());
    // Orphan responses cannot masquerade as a valid completed round trip.
    session
        .conversation
        .push(Message::user().with_tool_response("orphan", Ok(CallToolResult::success(vec![]))));
    session
        .conversation
        .push(Message::assistant().with_text("Done"));
    assert!(inspect_turn(&session, &CitationRegistry::new(), &summary).is_err());
    Ok(())
}
use super::*;
use crate::{diagnostics::plain_output, provider_setup::DEFAULT_PROVIDER_CONFIG};
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
    assert!(fields.contains("lesson:"));
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
    assert!(inspect_turn(&session, &CitationRegistry::new(), &summary).is_err());
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
    let mut output = Vec::new();
    let (produced, displayed) =
        tokio::join!(producer, display_events_to(receiver, &mut output, false));
    produced?;
    let summary = displayed?;
    assert_eq!(summary.message_events, 3);
    assert!(!summary.saw_text);
    assert!(summary.displayed_text);
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
pub(super) fn arguments(shares: u32) -> JsonObject {
    let value = json!({"entry_price": "51.20", "stop_price": "50.70", "share_count": shares});
    match value {
        Value::Object(arguments) => arguments,
        _ => panic!("test fixture is a JSON object"),
    }
}

pub(super) fn request(id: &str, name: &str, shares: u32) -> Message {
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
        .append_user(SESSION_ID, "Calculate for my hypothetical trade")
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

// Deterministic orchestration fixtures: scripted Operations, not provider/live evidence.
// Events use the real consumer; Effects use the real GDK loop and ChatRuntime.
struct OrchestrationDiagnostics {
    directory: PathBuf,
}

impl OrchestrationDiagnostics {
    fn new() -> anyhow::Result<Self> {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let sequence = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let directory = env::temp_dir().join(format!(
            "gdk-orchestration-{}-{}-{}",
            std::process::id(),
            stamp,
            sequence
        ));
        fs::create_dir(&directory)?;
        Ok(Self { directory })
    }

    fn path(&self) -> PathBuf {
        self.directory.join("diagnostics.md")
    }
}

impl Drop for OrchestrationDiagnostics {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.directory);
    }
}

enum OrchestrationExit {
    Continue,
    Yield,
    Fail,
}

struct OrchestrationWork {
    expected_passes: usize,
    events: Vec<Message>,
    effects: Vec<ChatEffect>,
    exit: OrchestrationExit,
}

impl OrchestrationWork {
    fn saved(expected_passes: usize, messages: Vec<Message>) -> Self {
        let effects = messages.iter().cloned().map(ChatEffect::from).collect();
        Self {
            expected_passes,
            events: messages,
            effects,
            exit: OrchestrationExit::Continue,
        }
    }
}

struct OrchestrationScript {
    work: Mutex<std::collections::VecDeque<OrchestrationWork>>,
    observed_passes: Mutex<Vec<usize>>,
    attempt_starts: std::sync::atomic::AtomicUsize,
}

impl OrchestrationScript {
    fn new(work: Vec<OrchestrationWork>) -> Self {
        Self {
            work: Mutex::new(work.into()),
            observed_passes: Mutex::new(Vec::new()),
            attempt_starts: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    fn attempts(&self) -> usize {
        self.attempt_starts
            .load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[async_trait]
impl Operation<ChatSession, ChatEffect> for OrchestrationScript {
    fn name(&self) -> &'static str {
        "deterministic-orchestration-fixture"
    }

    async fn run(
        &self,
        session: &ChatSession,
        conversation: &Conversation,
        emit: &Emitter,
    ) -> anyhow::Result<OperationResult<ChatEffect>> {
        let turn = messages_since_kickoff(conversation)?;
        if goose_agent::operation::ends_turn(turn) {
            return not_applicable();
        }
        if turn.len() == 1 {
            self.attempt_starts
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        let work = self.work.lock().await.pop_front();
        let work = match work {
            Some(work) => work,
            None => {
                return Err(anyhow::anyhow!(
                    "unexpected scripted orchestration invocation"
                ))
            }
        };
        assert_eq!(session.applied_passes, work.expected_passes);
        self.observed_passes
            .lock()
            .await
            .push(session.applied_passes);
        for event in work.events {
            emit.emit(AgentEvent::Message(event)).await;
        }
        match work.exit {
            OrchestrationExit::Continue => applied(work.effects),
            OrchestrationExit::Yield => yielded_with(work.effects),
            OrchestrationExit::Fail => Err(anyhow::anyhow!("scripted orchestration run failure")),
        }
    }
}

async fn orchestration_runtime(memory: bool) -> anyhow::Result<ChatRuntime> {
    let runtime = ChatRuntime::seeded();
    {
        let mut sessions = runtime.sessions.lock().await;
        let stored = sessions.get_mut(SESSION_ID).expect("fixture Session");
        stored.lesson.persistence_enabled = memory;
        // Complete the opening exchange without making it genuine learner evidence.
        stored.conversation.push(
            Message::assistant()
                .with_text("Fixture welcome.")
                .with_generated_id_if_missing(),
        );
    }
    let grounding = rag::select_grounding(
        "Fixture learner question",
        &grounding_response(vec![grounding_fixture(
            "orchestration",
            0.90,
            "Authored public fixture evidence.",
        )]),
        &forced_grounding_policy(),
    );
    runtime
        .append_grounded_user(SESSION_ID, "I want to learn orders.", grounding)
        .await?;
    Ok(runtime)
}

fn orchestration_machine(
    script: Arc<OrchestrationScript>,
    cancel: &CancellationToken,
) -> StateMachine<'static, ChatSession, ChatEffect> {
    StateMachine::new(
        vec![
            Step::Operation(Arc::new(RunLimit)),
            Step::Operation(Arc::new(safe_tools())),
            Step::Operation(script),
        ],
        cancel.clone(),
    )
}

fn orchestration_reminders(session: &ChatSession) -> usize {
    session
        .conversation
        .messages()
        .iter()
        .filter(|message| message.as_concat_text() == GROUNDING_ESCALATION_MESSAGE)
        .count()
}

// The estimator only formats stored state; fixtures use a locally declared model.
fn estimation_sizer() -> context_size::ContextSize {
    let config = deserialize_provider_config(
        r#"{"name":"local","engine":"openai","display_name":"Local","base_url":"http://127.0.0.1:8080","requires_auth":false,"preserves_thinking":false,"models":[{"name":"fixture-model"}]}"#,
    )
    .expect("fixture provider config");
    context_size::ContextSize::new(&ModelConfig::new("fixture-model"), &config)
}

#[tokio::test]
async fn deterministic_orchestration_silent_ignore_retries_once_with_fresh_bounds(
) -> anyhow::Result<()> {
    let runtime = orchestration_runtime(false).await?;
    let before = runtime.load(SESSION_ID).await?;
    let mut work = Vec::new();
    let answers = ["Fixture initial answer.", "Fixture second answer."];
    for answer in answers {
        // All four actual calculator calls use the SAME IDs on the next attempt.
        // SafeTools must use the new kickoff window, not reject retained-history IDs.
        let mut requests = Message::assistant();
        for index in 0..MAX_TOOL_REQUESTS {
            requests = requests.with_tool_request(
                format!("reused-fixture-{}", index),
                Ok(CallToolRequestParams::new(TOOL_NAME).with_arguments(arguments(200))),
            );
        }
        work.push(OrchestrationWork::saved(0, vec![requests]));
        // The request and real SafeTools dispatch consume two work passes.
        for pass in 2..MAX_APPLIED_PASSES - 1 {
            work.push(OrchestrationWork::saved(pass, Vec::new()));
        }
        work.push(OrchestrationWork::saved(
            MAX_APPLIED_PASSES - 1,
            vec![Message::assistant().with_text(answer)],
        ));
    }
    let script = Arc::new(OrchestrationScript::new(work));
    let cancel = CancellationToken::new();
    let machine = orchestration_machine(script.clone(), &cancel);
    let temporary = OrchestrationDiagnostics::new()?;
    let mut diagnostics = DiagnosticLog::create_at(temporary.path())?;
    run_turn(
        &machine,
        &runtime,
        &cancel,
        SESSION_ID,
        2,
        &PromptCaptures::default(),
        &estimation_sizer(),
        &LossToolProvider,
        &CitationRegistry::new(),
        &mut diagnostics,
    )
    .await?;
    let saved = runtime.load(SESSION_ID).await?;
    assert_eq!(script.attempts(), 2);
    assert!(script.work.lock().await.is_empty());
    let expected_passes = [vec![0], (2..MAX_APPLIED_PASSES).collect()].concat();
    assert_eq!(
        *script.observed_passes.lock().await,
        [expected_passes.clone(), expected_passes].concat()
    );
    assert_eq!(saved.applied_passes, MAX_APPLIED_PASSES);
    assert!(saved.halt_reason.is_none());
    assert_eq!(saved.id, before.id);
    assert_eq!(saved.genuine_reply_ids, before.genuine_reply_ids);
    assert_eq!(saved.genuine_reply_ids.len(), 1);
    assert_eq!(orchestration_reminders(&saved), 1);
    let messages = saved.conversation.messages();
    assert_eq!(
        messages
            .iter()
            .filter(|message| answers.contains(&message.as_concat_text().as_str()))
            .count(),
        2
    );
    assert_eq!(
        messages.last().expect("second answer").as_concat_text(),
        answers[1]
    );
    assert_eq!(
        serde_json::to_value(&saved.turn_grounding.evidence)?,
        serde_json::to_value(&before.turn_grounding.evidence)?
    );
    let reminder = messages
        .iter()
        .find(|message| message.as_concat_text() == GROUNDING_ESCALATION_MESSAGE)
        .expect("one application reminder");
    assert!(!saved
        .genuine_reply_ids
        .contains(reminder.id.as_ref().expect("reminder ID")));
    let responses = messages
        .iter()
        .flat_map(|message| &message.content)
        .filter_map(|block| block.as_tool_response())
        .collect::<Vec<_>>();
    assert_eq!(responses.len(), 2 * MAX_TOOL_REQUESTS);
    for response in responses {
        assert!(tool_response_succeeded(&response.tool_result));
        let result = response
            .tool_result
            .as_ref()
            .expect("actual calculator result");
        assert!(result
            .content
            .iter()
            .any(|content| { content.as_text().is_some_and(|text| text.text == "$100.00") }));
    }
    let transcript = fs::read_to_string(temporary.path())?;
    assert_eq!(transcript.matches("## Grounding escalation (").count(), 1);
    assert_eq!(
        transcript
            .matches("## Grounding outcome after one escalation (")
            .count(),
        1
    );
    assert!(!transcript.contains("## Learning note decision"));
    Ok(())
}

async fn orchestration_acknowledged_once(answer: &str) -> anyhow::Result<()> {
    let runtime = orchestration_runtime(false).await?;
    let script = Arc::new(OrchestrationScript::new(vec![OrchestrationWork::saved(
        0,
        vec![Message::assistant().with_text(answer)],
    )]));
    let cancel = CancellationToken::new();
    let machine = orchestration_machine(script.clone(), &cancel);
    let temporary = OrchestrationDiagnostics::new()?;
    let mut diagnostics = DiagnosticLog::create_at(temporary.path())?;
    run_turn(
        &machine,
        &runtime,
        &cancel,
        SESSION_ID,
        2,
        &PromptCaptures::default(),
        &estimation_sizer(),
        &LossToolProvider,
        &CitationRegistry::new(),
        &mut diagnostics,
    )
    .await?;
    let saved = runtime.load(SESSION_ID).await?;
    assert_eq!(script.attempts(), 1);
    assert_eq!(*script.observed_passes.lock().await, vec![0]);
    assert!(script.work.lock().await.is_empty());
    assert_eq!(saved.applied_passes, 1);
    assert_eq!(orchestration_reminders(&saved), 0);
    assert_eq!(saved.genuine_reply_ids.len(), 1);
    assert_eq!(
        saved
            .conversation
            .last()
            .expect("saved answer")
            .as_concat_text(),
        answer
    );
    assert!(!fs::read_to_string(temporary.path())?.contains("## Grounding escalation"));
    Ok(())
}

// A bare decline is accepted without retry: the strengthened grounding
// instruction asks the tutor to take the added context from the excerpts
// unless there is a very strong stated reason, but a valid [source:none]
// answer still ends the turn with no application re-ask.
#[tokio::test]
async fn deterministic_orchestration_declined_excerpts_finish_without_retry() -> anyhow::Result<()>
{
    orchestration_acknowledged_once("Fixture excerpts do not support this answer. [source:none]")
        .await
}

#[tokio::test]
async fn deterministic_orchestration_cited_excerpts_finish_without_retry() -> anyhow::Result<()> {
    orchestration_acknowledged_once("Fixture supported statement. [source:orchestration]").await
}

#[tokio::test]
async fn deterministic_orchestration_failed_halted_or_incomplete_attempt_never_approves_or_retries(
) -> anyhow::Result<()> {
    // No stdin/global-environment manipulation: a full ledger also makes accidental
    // approval fail before confirmation, rather than blocking the test process.
    let record_before = read_record(&record_path())?;
    for failure in ["run", "halt", "incomplete"] {
        let runtime = orchestration_runtime(true).await?;
        {
            let mut sessions = runtime.sessions.lock().await;
            let stored = sessions.get_mut(SESSION_ID).expect("fixture Session");
            for index in 0..MAX_SETTLED_APPROVALS {
                stored
                    .settled_approvals
                    .insert(format!("prior-{}", index), false);
            }
        }
        let before = runtime.load(SESSION_ID).await?;
        let proposal = Message::assistant().with_tool_request(
            "pending-fixture-note",
            Ok(proposal_call(
                "stated_goals",
                "Learn orders",
                "learn orders",
            )),
        );
        let visible = Message::assistant().with_text("Fixture visible but unapproved text.");
        let mut ending = OrchestrationWork::saved(2, vec![visible.clone()]);
        match failure {
            "run" => ending.exit = OrchestrationExit::Fail,
            "halt" => {
                ending
                    .effects
                    .push(ChatEffect::Halt("fixture safeguard".to_string()));
                ending.exit = OrchestrationExit::Yield;
            }
            "incomplete" => {
                // A live Event without a saved final answer must not pass inspection.
                ending.effects.clear();
                ending.exit = OrchestrationExit::Yield;
            }
            _ => unreachable!(),
        }
        let script = Arc::new(OrchestrationScript::new(vec![
            OrchestrationWork::saved(0, vec![proposal]),
            ending,
        ]));
        let cancel = CancellationToken::new();
        let machine = orchestration_machine(script.clone(), &cancel);
        let temporary = OrchestrationDiagnostics::new()?;
        let mut diagnostics = DiagnosticLog::create_at(temporary.path())?;
        let result = run_turn(
            &machine,
            &runtime,
            &cancel,
            SESSION_ID,
            2,
            &PromptCaptures::default(),
            &estimation_sizer(),
            &LossToolProvider,
            &CitationRegistry::new(),
            &mut diagnostics,
        )
        .await;
        let error = result.expect_err("unsuccessful attempt cannot classify or approve");
        let expected_error = match failure {
            "run" => "scripted orchestration run failure",
            "halt" => "run stopped at a safeguard: fixture safeguard",
            "incomplete" => "no ordinary final tutor answer was both displayed and saved",
            _ => unreachable!(),
        };
        assert!(error.to_string().contains(expected_error), "{}", error);
        assert_eq!(script.attempts(), 1);
        assert_eq!(*script.observed_passes.lock().await, vec![0, 2]);
        assert!(script.work.lock().await.is_empty());
        let saved = runtime.load(SESSION_ID).await?;
        assert_eq!(orchestration_reminders(&saved), 0);
        assert_eq!(saved.genuine_reply_ids, before.genuine_reply_ids);
        assert_eq!(saved.settled_approvals, before.settled_approvals);
        assert_eq!(saved.lesson.learner_record, before.lesson.learner_record);
        let response = saved
            .conversation
            .messages()
            .iter()
            .flat_map(|message| &message.content)
            .filter_map(|block| block.as_tool_response())
            .find(|response| response.id == "pending-fixture-note")
            .expect("real proposal-only SafeTools acknowledgment");
        assert!(tool_response_succeeded(&response.tool_result));
        let transcript = fs::read_to_string(temporary.path())?;
        assert!(transcript.contains("## Complete Session after run"));
        assert!(transcript.contains(if failure == "run" {
            "## Run failure"
        } else {
            "## Turn failure"
        }));
        assert!(!transcript.contains("## Grounding outcome"));
        assert!(!transcript.contains("## Grounding escalation"));
        assert!(!transcript.contains("## Learning note decision"));
        assert_eq!(read_record(&record_path())?, record_before);
    }
    Ok(())
}

#[tokio::test]
async fn deterministic_orchestration_first_approval_validation_failure_prevents_escalation(
) -> anyhow::Result<()> {
    let runtime = orchestration_runtime(true).await?;
    let before = runtime.load(SESSION_ID).await?;
    let record_before = read_record(&record_path())?;
    // Deliberately inconsistent saved proposal state exercises approval's independent
    // validation. This is NOT a claimed tool result/provider response: the scripted
    // Operation supplies the batch so SafeTools cannot validate it beforehand.
    let invalid_request = Message::assistant().with_tool_request(
        "invalid-fixture-note",
        Ok(proposal_call(
            "stated_goals",
            "Fixture note",
            "Never said by the learner",
        )),
    );
    let scripted_response = Message::user().with_tool_response(
        "invalid-fixture-note",
        Ok(CallToolResult::success(vec![ContentBlock::text(
            "Fixture acknowledgment",
        )])),
    );
    let script = Arc::new(OrchestrationScript::new(vec![OrchestrationWork::saved(
        0,
        vec![
            invalid_request,
            scripted_response,
            Message::assistant().with_text("Fixture uncited ordinary answer."),
        ],
    )]));
    let cancel = CancellationToken::new();
    let machine = orchestration_machine(script.clone(), &cancel);
    let temporary = OrchestrationDiagnostics::new()?;
    let mut diagnostics = DiagnosticLog::create_at(temporary.path())?;
    let result = run_turn(
        &machine,
        &runtime,
        &cancel,
        SESSION_ID,
        2,
        &PromptCaptures::default(),
        &estimation_sizer(),
        &LossToolProvider,
        &CitationRegistry::new(),
        &mut diagnostics,
    )
    .await;
    let error = result.expect_err("approval must independently validate genuine source");
    assert!(error
        .to_string()
        .contains("learner_quote must exactly quote"));
    assert_eq!(script.attempts(), 1);
    assert_eq!(*script.observed_passes.lock().await, vec![0]);
    let saved = runtime.load(SESSION_ID).await?;
    assert_eq!(saved.applied_passes, 1);
    assert_eq!(orchestration_reminders(&saved), 0);
    assert_eq!(saved.genuine_reply_ids, before.genuine_reply_ids);
    assert_eq!(saved.settled_approvals, before.settled_approvals);
    assert_eq!(saved.lesson.learner_record, before.lesson.learner_record);
    let transcript = fs::read_to_string(temporary.path())?;
    assert!(transcript.contains("## Grounding outcome (application classification)"));
    assert!(!transcript.contains("## Run failure"));
    assert!(!transcript.contains("## Turn failure"));
    assert!(!transcript.contains("## Grounding escalation"));
    assert!(!transcript.contains("## Learning note decision"));
    assert_eq!(read_record(&record_path())?, record_before);
    Ok(())
}
