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

#[test]
fn source_inventory_covers_library_and_all_production_rag_modules() {
    let expected = [
        "src/lib.rs",
        "src/rag/mod.rs",
        "src/rag/retrieval.rs",
        "src/rag/grounding.rs",
        "src/rag/citations.rs",
        "src/rag/transport.rs",
    ];
    for path in expected {
        assert!(SOURCE_PATHS.contains(&path), "missing {}", path);
    }
    let unique: std::collections::HashSet<_> = SOURCE_PATHS.iter().collect();
    assert_eq!(unique.len(), SOURCE_PATHS.len());
    assert!(!SOURCE_PATHS.contains(&"src/retrieval.rs"));
    assert!(SOURCE_PATHS.iter().all(|path| !path.contains("tests")));
    assert!(SOURCE_PATHS
        .iter()
        .all(|path| !path.contains("provider-config")));
    assert!(!SOURCE_PATHS.contains(&".env"));
}

#[test]
fn grounding_projection_distinguishes_unavailable_and_empty_from_selected_evidence() {
    let mut grounding = gdk_hello::rag::TurnGrounding::default();
    for status in ["none", "no_evidence", "unavailable"] {
        grounding.status = status.to_string();
        let value = grounding_projection(&grounding);
        assert_eq!(value["phase"], "automatic_grounding");
        assert_eq!(value["status"], status);
        assert_eq!(value["selected_evidence"], serde_json::json!([]));
        assert_eq!(value["support_check"], "not_semantically_verified");
        assert!(value.get("tool_name").is_none());
    }
}

#[test]
fn attempts_never_claim_visibility_or_semantic_support() {
    use gdk_hello::rag::GroundingOutcome;
    let cases = [
        (GroundingOutcome::NotSupplied, "not_supplied"),
        (GroundingOutcome::Cited, "cited"),
        (GroundingOutcome::Declined, "declined"),
        (GroundingOutcome::SilentIgnore, "silent_ignore"),
        (
            GroundingOutcome::InvalidAcknowledgment,
            "invalid_acknowledgment",
        ),
    ];
    for (outcome, label) in cases {
        let value = attempt_projection(1, &outcome, false, "ordinary answer", &[]);
        assert_eq!(value["phase"], "attempt");
        assert_eq!(value["attempt"], 1);
        assert_eq!(value["acknowledgment"], label);
        assert_eq!(value["presentation"], "not_yet_presented");
        assert_eq!(value["support_check"], "not_semantically_verified");
    }
}

#[test]
fn withheld_attempt_and_warned_visible_correction_are_distinct_records() {
    let invalid = vec!["source:never-supplied".to_string()];
    let withheld = attempt_projection(
        1,
        &gdk_hello::rag::GroundingOutcome::InvalidAcknowledgment,
        true,
        "withheld ordinary answer",
        &invalid,
    );
    assert_eq!(withheld["presentation"], "withheld");
    assert_eq!(withheld["invalid_markers"], serde_json::json!(invalid));
    assert_eq!(withheld["ordinary_answer"], "withheld ordinary answer");
    let warnings = vec!["Support remains unverified".to_string()];
    let visible = visible_projection(2, "final warned answer", &warnings);
    assert_eq!(visible["phase"], "learner_visible");
    assert_eq!(visible["attempt"], 2);
    assert_eq!(visible["ordinary_answer"], "final warned answer");
    assert_eq!(visible["warnings"], serde_json::json!(warnings));
    assert_eq!(visible["support_check"], "not_semantically_verified");
}

#[test]
fn observable_projection_excludes_thinking_and_signatures() {
    let message = goose_providers::conversation::message::Message::assistant()
        .with_text("ordinary answer")
        .with_thinking("private reasoning must not export", "private signature")
        .with_redacted_thinking("private redacted payload");
    let text = observable(&[message]);
    assert!(text.contains("ordinary answer"));
    for hidden in ["private reasoning", "private signature", "private redacted"] {
        assert!(!text.contains(hidden));
    }
}

#[derive(Clone, Debug)]
struct CapturedSpans(std::sync::Arc<std::sync::Mutex<Vec<opentelemetry_sdk::trace::SpanData>>>);

impl opentelemetry_sdk::trace::SpanExporter for CapturedSpans {
    async fn export(
        &self,
        batch: Vec<opentelemetry_sdk::trace::SpanData>,
    ) -> opentelemetry_sdk::error::OTelSdkResult {
        self.0.lock().expect("capture lock").extend(batch);
        Ok(())
    }
}

#[test]
fn trace_records_separate_preparation_attempt_correction_and_visible_output() {
    let captured = CapturedSpans(std::sync::Arc::new(std::sync::Mutex::new(Vec::new())));
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(captured.clone())
        .build();
    let subscriber = tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(provider.tracer("telemetry-test")));
    tracing::subscriber::with_default(subscriber, || {
        let turn = tracing::info_span!("test_turn");
        let _turn = turn.enter();
        {
            let preparation = tracing::info_span!("grounding_preparation");
            let _preparation = preparation.enter();
            grounding_preparation(&gdk_hello::rag::TurnGrounding::default());
        }
        {
            let attempt = tracing::info_span!("buffered_attempt");
            let _attempt = attempt.enter();
            attempt_output(
                1,
                &gdk_hello::rag::GroundingOutcome::InvalidAcknowledgment,
                true,
                "withheld ordinary answer",
                &["source:never-supplied".to_string()],
            );
        }
        correction_requested(1, "application correction");
        learner_visible_output(2, "actual visible answer", &[]);
    });
    provider.force_flush().expect("flush local capture");
    provider.shutdown().expect("shutdown local capture");
    let spans = captured.0.lock().expect("capture lock");
    let output = |name: &str| {
        spans
            .iter()
            .find(|span| span.name == name)
            .expect("captured span")
            .attributes
            .iter()
            .find(|attribute| attribute.key.as_str() == "mlflow.spanOutputs")
            .expect("output attribute")
            .value
            .to_string()
    };
    let preparation: serde_json::Value =
        serde_json::from_str(&output("grounding_preparation")).expect("grounding JSON");
    assert_eq!(preparation["phase"], "automatic_grounding");
    let attempt: serde_json::Value =
        serde_json::from_str(&output("buffered_attempt")).expect("attempt JSON");
    assert_eq!(attempt["presentation"], "withheld");
    let visible: serde_json::Value =
        serde_json::from_str(&output("test_turn")).expect("visible JSON");
    assert_eq!(visible["phase"], "learner_visible");
    assert_eq!(visible["ordinary_answer"], "actual visible answer");
    let turn = spans.iter().find(|span| span.name == "test_turn").unwrap();
    assert!(turn.events.iter().any(|event| {
        event.attributes.iter().any(|attribute| {
            attribute.key.as_str() == "course.rag.correction"
                && attribute.value.to_string().contains("correction_appended")
        })
    }));
}

#[test]
fn selected_grounding_projection_retains_actual_excerpt_identity_and_qualification() {
    let grounding = gdk_hello::rag::TurnGrounding {
        status: "grounded".to_string(),
        query: "actual learner question".to_string(),
        threshold: 0.55,
        notice: None,
        evidence: vec![gdk_hello::rag::retrieval::Evidence {
            citation_id: format!("source:{}", "a".repeat(64)),
            chunk_id: "a".repeat(64),
            document_id: "b".repeat(64),
            source_file: "public-fixture.pdf".to_string(),
            source_sha256: "c".repeat(64),
            content_sha256: "d".repeat(64),
            page_start: 2,
            page_end: 2,
            raw_fragments: Vec::new(),
            review_status: "draft fixture".to_string(),
            similarity: 0.7,
            excerpt: "A fallible public test excerpt.".to_string(),
            excerpt_truncated: true,
        }],
    };
    let value = grounding_projection(&grounding);
    assert_eq!(value["query"], "actual learner question");
    assert_eq!(
        value["selected_evidence"][0]["citation_id"],
        grounding.evidence[0].citation_id
    );
    assert_eq!(
        value["selected_evidence"][0]["excerpt"],
        "A fallible public test excerpt."
    );
    assert_eq!(value["selected_evidence"][0]["excerpt_truncated"], true);
    assert_eq!(
        value["selected_evidence"][0]["review_status"],
        "draft fixture"
    );
    assert_eq!(value["support_check"], "not_semantically_verified");
    for forbidden in [
        "database_url",
        "embedding_url",
        "tutor_url",
        "provider_config",
    ] {
        assert!(value.get(forbidden).is_none());
    }
}
