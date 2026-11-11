// Lesson 12

// Deterministic fixtures keep grounding-selection tests independent of live services.
fn grounding_evidence(citation_id: &str, similarity: f64, file: &str) -> retrieval::Evidence {
    retrieval::Evidence {
        citation_id: citation_id.to_string(),
        chunk_id: format!("chunk-{}", citation_id),
        document_id: format!("document-{}", citation_id),
        source_file: file.to_string(),
        source_sha256: "source-sha".to_string(),
        content_sha256: "content-sha".to_string(),
        page_start: 3,
        page_end: 4,
        raw_fragments: Vec::new(),
        review_status: "draft".to_string(),
        similarity,
        excerpt: format!("Public draft excerpt for {}.", citation_id),
        excerpt_truncated: false,
    }
}

fn grounding_response(evidence: Vec<retrieval::Evidence>) -> retrieval::SearchResponse {
    retrieval::SearchResponse {
        collection: "public-fixture".to_string(),
        model: "public".to_string(),
        revision: "public".to_string(),
        profile: "public".to_string(),
        status: "ok".to_string(),
        notice: String::new(),
        evidence,
        used_tokens: 0,
    }
}

#[test]
fn grounding_selection_filters_by_threshold_and_caps_statements() {
    let response = grounding_response(vec![
        grounding_evidence("source:a", 0.90, "public-a.pdf"),
        grounding_evidence("source:b", 0.70, "public-b.pdf"),
        grounding_evidence("source:c", 0.50, "public-c.pdf"),
        grounding_evidence("source:d", 0.49, "public-d.pdf"),
        grounding_evidence("source:e", 0.10, "public-e.pdf"),
    ]);
    let grounding = select_grounding("what is a stop loss", &response, 0.50);
    assert_eq!(grounding.status, "grounded");
    assert_eq!(grounding.query, "what is a stop loss");
    assert_eq!(grounding.threshold, 0.50);
    // The similarity boundary is inclusive; below-threshold candidates never pass.
    let ids: Vec<&str> = grounding
        .evidence
        .iter()
        .map(|passage| passage.citation_id.as_str())
        .collect();
    assert_eq!(ids, vec!["source:a", "source:b", "source:c"]);
    // Candidates beyond the cap are dropped from the tail, best-first.
    let capped = select_grounding(
        "cap check",
        &grounding_response(vec![
            grounding_evidence("source:a", 0.90, "public-a.pdf"),
            grounding_evidence("source:b", 0.80, "public-b.pdf"),
            grounding_evidence("source:c", 0.70, "public-c.pdf"),
            grounding_evidence("source:d", 0.60, "public-d.pdf"),
        ]),
        0.50,
    );
    assert_eq!(capped.evidence.len(), GROUNDING_MAX_STATEMENTS);
    assert!(capped
        .evidence
        .iter()
        .all(|passage| passage.citation_id != "source:d"));
}

#[test]
fn grounding_selection_reports_no_evidence_without_candidates() {
    let grounding = select_grounding("empty query", &grounding_response(Vec::new()), 0.50);
    assert_eq!(grounding.status, "no_evidence");
    assert!(grounding.evidence.is_empty());
    assert!(grounding.notice.is_none());
    // Nothing qualifies, so no prompt part, footer, or citation allowlist appears.
    assert!(grounding.prompt_part().is_none());
    assert!(citation_footer(&grounding, "Answer [source:source:a]").is_none());
    assert!(inspect_citations(&grounding, "Answer without markers").is_empty());
}

#[test]
fn grounding_selection_marks_unavailable_search_without_evidence() {
    let failure = "reader connection refused".to_string();
    let grounding = TurnGrounding {
        status: "unavailable".to_string(),
        query: "learner question".to_string(),
        threshold: GROUNDING_MIN_SIMILARITY,
        notice: Some(failure.clone()),
        evidence: Vec::new(),
    };
    // A degraded turn proceeds without excerpts; no citation is authorized.
    assert!(grounding.prompt_part().is_none());
    assert!(inspect_citations(&grounding, "Answer").is_empty());
    let summary = grounding.grounding_summary();
    assert!(summary.contains("status: unavailable"));
    assert!(summary.contains(&failure));
}

#[tokio::test]
#[ignore = "requires actual local reader/embedding services"]
async fn live_forced_grounding_prepares_turn_state_without_touching_history() -> anyhow::Result<()>
{
    let retrieval = match retrieval::Retrieval::from_env().await {
        Ok(retrieval) => retrieval,
        Err(failure) => return Err(anyhow::anyhow!(failure)),
    };
    let runtime = ChatRuntime::seeded();
    runtime
        .append_user(SESSION_ID, "What is a stop loss?")
        .await?;
    let before = runtime.load(SESSION_ID).await?;
    let path = std::env::temp_dir().join("lesson12-grounding-diagnostics-test.md");
    let mut diagnostics = DiagnosticLog::create_at(path)?;
    prepare_turn_grounding(
        &runtime,
        SESSION_ID,
        &retrieval,
        "What is a stop loss?",
        GROUNDING_MIN_SIMILARITY,
        &mut diagnostics,
    )
    .await?;
    let after = runtime.load(SESSION_ID).await?;
    // Grounding is per-turn state, never appended to conversation history.
    assert_eq!(after.conversation.len(), before.conversation.len());
    assert_eq!(after.turn_grounding.status, "grounded");
    assert_eq!(after.turn_grounding.query, "What is a stop loss?");
    assert!(!after.turn_grounding.evidence.is_empty());
    assert!(after.turn_grounding.evidence.len() <= GROUNDING_MAX_STATEMENTS);
    for passage in &after.turn_grounding.evidence {
        assert!(passage.similarity >= GROUNDING_MIN_SIMILARITY);
    }
    let parts = safe_tools()
        .prompt_parts(&after, &after.conversation)
        .await?;
    assert_eq!(parts.len(), 4);
    assert_eq!(parts[3].0, "source-grounding");
    retrieval.shutdown().await;
    Ok(())
}

#[test]
fn citation_membership_uses_only_this_turns_selected_grounding() {
    let grounding = TurnGrounding {
        evidence: vec![
            grounding_evidence("source:selected-a", 0.90, "public-a.pdf"),
            grounding_evidence("source:selected-b", 0.80, "public-b.pdf"),
        ],
        ..TurnGrounding::default()
    };
    assert!(inspect_citations(&grounding, "Example [source:selected-a]").is_empty());
    assert_eq!(
        inspect_citations(&grounding, "Example [source:invented]").len(),
        1
    );
    // An unclosed marker is never silently accepted as a citation.
    assert_eq!(
        inspect_citations(&grounding, "Example [source:selected-a").len(),
        1
    );
    // A grounded turn that cites nothing is surfaced for human review.
    assert_eq!(inspect_citations(&grounding, "Plain answer").len(), 1);
    // An ungrounded turn has an empty allowlist; any marker is unverified.
    let ungrounded = TurnGrounding::default();
    assert_eq!(
        inspect_citations(&ungrounded, "Example [source:selected-a]").len(),
        1
    );
    assert!(inspect_citations(&ungrounded, "Plain answer").is_empty());
}

#[test]
fn grounding_prompt_part_frames_excerpts_as_untrusted_drafts() {
    // No selection means no injection; the tutor sees only lesson context.
    let empty = select_grounding("q", &grounding_response(Vec::new()), 0.50);
    assert!(empty.prompt_part().is_none());
    let grounding = select_grounding(
        "what is a stop loss",
        &grounding_response(vec![grounding_evidence(
            "source:selected-a",
            0.90,
            "/tmp/private/day_trading/public-book.pdf",
        )]),
        0.50,
    );
    let (name, body) = match grounding.prompt_part() {
        Some(part) => part,
        None => panic!("grounded turn must inject a prompt part"),
    };
    assert_eq!(name, "source-grounding");
    // The framing denies instructions, mastery, and invented IDs.
    assert!(body.contains("not instructions"));
    assert!(body.contains("not proof of mastery"));
    assert!(body.contains("never invent or alter source IDs"));
    // The single excerpt appears with its file, pages, similarity, ID and text.
    assert!(body.contains("public-book.pdf"));
    assert!(body.contains("pages 3-4"));
    assert!(body.contains("similarity 0.90"));
    assert!(body.contains("source:selected-a"));
    assert!(body.contains("Public draft excerpt for source:selected-a."));
}

#[tokio::test]
async fn grounding_rides_prompt_parts_and_never_enters_conversation_history() -> anyhow::Result<()>
{
    let runtime = ChatRuntime::seeded();
    runtime
        .append_user(SESSION_ID, "How does a stop order work?")
        .await?;
    let session = runtime.load(SESSION_ID).await?;
    // Before preparation the default grounding injects nothing.
    assert_eq!(session.turn_grounding.status, "");
    let parts_without = safe_tools()
        .prompt_parts(&session, &session.conversation)
        .await?;
    assert_eq!(parts_without.len(), 3);
    // Setting the turn grounding adds exactly one prompt part.
    let grounding = TurnGrounding {
        status: "grounded".to_string(),
        query: "How does a stop order work?".to_string(),
        threshold: 0.50,
        evidence: vec![grounding_evidence(
            "source:selected-a",
            0.90,
            "public-a.pdf",
        )],
        notice: None,
    };
    runtime
        .set_turn_grounding(SESSION_ID, grounding.clone())
        .await?;
    let updated = runtime.load(SESSION_ID).await?;
    let parts_with = safe_tools()
        .prompt_parts(&updated, &updated.conversation)
        .await?;
    assert_eq!(parts_with.len(), 4);
    assert_eq!(parts_with[0], parts_without[0]);
    assert_eq!(parts_with[1], parts_without[1]);
    assert_eq!(parts_with[2], parts_without[2]);
    assert_eq!(parts_with[3], grounding.prompt_part().expect("part"));
    // Grounding state replaced conversation: history is untouched.
    assert_eq!(updated.conversation.len(), session.conversation.len());
    assert_eq!(
        updated.conversation.messages(),
        session.conversation.messages()
    );
    // Diagnostics expose the grounding state with its query and threshold.
    let fields = session_fields(&updated);
    assert!(fields.contains("turn_grounding:"));
    assert!(fields.contains("query:"));
    Ok(())
}

#[test]
fn citation_footer_lists_only_grounding_selected_sources() {
    let grounding = TurnGrounding {
        evidence: vec![
            grounding_evidence("source:selected-a", 0.90, "/tmp/private/public-a.pdf"),
            grounding_evidence("source:selected-b", 0.80, "public-b.pdf"),
        ],
        ..TurnGrounding::default()
    };
    // No evidence or no citations means no footer.
    let empty = TurnGrounding::default();
    assert!(citation_footer(&empty, "Answer [source:selected-a]").is_none());
    assert!(citation_footer(&grounding, "Answer without markers").is_none());
    // Cited grounding IDs render with file, pages, and similarity.
    let footer = citation_footer(
        &grounding,
        "Excerpt claim [source:selected-b] and [source:selected-a].",
    )
    .expect("footer");
    assert!(footer.contains("Sources cited this turn:"));
    assert!(footer.contains("source:selected-a: public-a.pdf pages 3-4 (similarity 0.90)"));
    assert!(footer.contains("source:selected-b: public-b.pdf pages 3-4 (similarity 0.80)"));
    // A citation outside the grounding never renders in the footer; invalid
    // citations are surfaced by inspect_citations warnings instead.
    assert!(citation_footer(&grounding, "Answer [source:invented]").is_none());
}

#[tokio::test]
async fn retrieved_quotes_never_become_learner_proposal_evidence() -> anyhow::Result<()> {
    let runtime = ChatRuntime::seeded();
    let mut session = runtime.load(SESSION_ID).await?;
    session.lesson.persistence_enabled = true;
    session
        .conversation
        .push(Message::user().with_tool_response(
            "search-id",
            Ok(CallToolResult::success(vec![ContentBlock::text(
                "Public synthetic source says keep a journal.",
            )])),
        ));
    let proposal = proposal_call("learner_evidence", "Keeps a journal", "keep a journal");
    assert!(parse_record_proposal(&proposal, &session).is_err());
    session
        .conversation
        .push(Message::user().with_text("I want to keep a journal."));
    assert!(parse_record_proposal(&proposal, &session).is_ok());
    Ok(())
}

#[test]
fn streaming_preserves_source_markers_across_fragments() {
    // Citation markers may arrive split across streamed deltas; the prose
    // assembler must still hold them together for the footer and inspection.
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
        let error = validate_local_tutor(&json!({"engine":"openai","base_url":endpoint}))
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
    session
        .conversation
        .push(Message::user().with_text("I want to learn orders."));
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
    session
        .conversation
        .push(Message::user().with_text("A limit order might never fill."));
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
    session
        .conversation
        .push(Message::user().with_text("Learn orders"));
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
    assert!(inspect_turn(&session, &summary)?.is_empty());
    // Orphan responses cannot masquerade as a valid completed round trip.
    session
        .conversation
        .push(Message::user().with_tool_response("orphan", Ok(CallToolResult::success(vec![]))));
    session
        .conversation
        .push(Message::assistant().with_text("Done"));
    assert!(inspect_turn(&session, &summary).is_err());
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
    // The per-turn grounding state is part of the recorded Session.
    assert!(fields.contains("turn_grounding:"));
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
    assert!(inspect_turn(&session, &summary).is_err());
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
