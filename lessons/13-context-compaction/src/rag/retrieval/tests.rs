// Lesson 13
use super::*;

fn limits() -> RetrievalLimits {
    RetrievalLimits {
        max_query_bytes: 2000,
        max_http_bytes: 2 * 1024 * 1024,
        sql_timeout: Duration::from_secs(5),
        http_timeout: Duration::from_secs(15),
        max_excerpt_tokens: 1000,
        token_accounting_attempts: 8,
    }
}
#[test]
fn strict_query_and_bytes() {
    assert_eq!(
        validate_query("public question", limits().max_query_bytes),
        Ok(())
    );
    for query in ["", " ", "\t", "ok\0hidden", &"é".repeat(1001)] {
        assert!(
            validate_query(query, limits().max_query_bytes).is_err(),
            "expected rejection: {:?}",
            query
        );
    }
}
#[test]
fn caller_query_limit_counts_utf8_bytes() {
    assert_eq!(validate_query("éé", 4), Ok(()));
    assert_eq!(
        validate_query("éé", 3),
        Err("retrieval query must be nonblank and at most 3 UTF-8 bytes".to_string())
    );
    assert!(validate_query("a", 0).is_err());
}
#[test]
fn caller_http_and_database_limits_are_retained() {
    let mut configured = limits();
    configured.max_http_bytes = 512;
    configured.sql_timeout = Duration::from_millis(750);
    let http = match LocalHttp::new(&[], configured) {
        Ok(http) => http,
        Err(failure) => panic!("{}", failure),
    };
    assert_eq!(http.max_http_bytes, configured.max_http_bytes);
    let config = match database_config(
        "host=localhost user=retrieval_reader",
        configured.sql_timeout,
    ) {
        Ok(config) => config,
        Err(failure) => panic!("{}", failure),
    };
    assert_eq!(config.get_connect_timeout(), Some(&configured.sql_timeout));
}
#[test]
fn local_urls_reject_normalization_and_secrets() {
    for raw in [
        "http://127.1:8080",
        "http://2130706433",
        "http://example.com",
        "http://user:secret@localhost",
        "http://localhost/path",
        "http://localhost?secret=1",
        "http://localhost#x",
        "file://localhost",
    ] {
        assert!(validate_local_url(raw).is_err());
    }
    for raw in [
        "http://127.0.0.1:8080",
        "http://localhost:9999/",
        "http://[::1]:8080",
    ] {
        assert!(validate_local_url(raw).is_ok());
    }
}
#[test]
fn database_hosts_are_local() {
    assert!(database_config("host=localhost user=retrieval_reader", limits().sql_timeout).is_ok());
    for raw in [
        "host=example.com user=x",
        "host=/tmp user=x",
        "user=x",
        "host=localhost hostaddr=8.8.8.8",
    ] {
        assert!(database_config(raw, limits().sql_timeout).is_err());
    }
}
#[test]
fn vector_contract_is_exact() {
    let mut vector = vec![0.0; 1024];
    vector[0] = 1.0;
    let mut response = EmbeddingResponse {
        model: RUNTIME_MODEL.to_string(),
        data: vec![EmbeddingItem {
            index: 0,
            embedding: vector,
        }],
        usage: Usage {
            prompt_tokens: 3,
            total_tokens: 3,
        },
    };
    assert!(check_embedding(&response, 3).is_ok());
    assert!(check_embedding(&response, 4).is_err());
    response.data[0].embedding[0] = f32::NAN;
    assert!(check_embedding(&response, 3).is_err());
    response.data[0].embedding[0] = 0.0;
    assert!(check_embedding(&response, 3).is_err());
}
#[test]
fn stable_citation_membership_not_semantics() {
    let chunk = hash(b"authored public fixture");
    let citation = match citation_id(&chunk) {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    let mut allowed = HashSet::new();
    allowed.insert(citation.clone());
    assert!(validate_citations(&format!("[{}]", citation), &allowed).is_ok());
    assert!(validate_citations("source:invented", &allowed).is_err());
    assert!(citation_id("bad").is_err());
}
#[test]
fn runtime_accepts_build_changes_but_requires_model_and_context() {
    let mut props = json!({"build_info":"b11429-d81235049","model_alias":RUNTIME_MODEL,"model_ftype":"F16","is_sleeping":false,"total_slots":8,"default_generation_settings":{"n_ctx":2048}});
    let mut models = json!({"data":[{"id":RUNTIME_MODEL,"meta":{"n_embd":1024,"ftype":"F16","n_ctx_train":32768,"n_ctx":2048}}]});
    assert_eq!(
        check_runtime(&json!({"status":"ok"}), &props, &models),
        Ok(2048)
    );
    props["build_info"] = json!("b11669-620d2225c");
    assert_eq!(
        check_runtime(&json!({"status":"ok"}), &props, &models),
        Ok(2048)
    );
    props
        .as_object_mut()
        .expect("props object")
        .remove("build_info");
    assert_eq!(
        check_runtime(&json!({"status":"ok"}), &props, &models),
        Ok(2048)
    );
    for (field, value) in [
        ("model_alias", json!("another-model")),
        ("model_ftype", json!("Q8_0")),
        ("is_sleeping", json!(true)),
        ("total_slots", json!(0)),
    ] {
        let mut invalid_props = props.clone();
        invalid_props[field] = value;
        assert!(check_runtime(&json!({"status":"ok"}), &invalid_props, &models).is_err());
    }
    models["data"][0]["meta"]["n_ctx"] = json!(1024);
    assert!(check_runtime(&json!({"status":"ok"}), &props, &models).is_err());
}
fn public_provenance() -> (String, String, Value, Value) {
    let source_hash = hash(b"authored synthetic source bytes");
    let document = hash(format!("public-fixture.pdf\n{}", source_hash).as_bytes());
    let text = "Authored public passage about hypothetical study notes.";
    let block = hash(b"authored synthetic block");
    let locations = format!("{}:0:{};", block, text.len());
    let encoded = match serde_json::to_vec(&[
        "page-paragraph-v1",
        "page-block-groups-v1",
        &document,
        "3000",
        &locations,
        text,
        text,
    ]) {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    let chunk = hash(&encoded);
    let metadata = json!({"id":chunk,"document_id":document,"text":text,"content_sha256":hash(text.as_bytes()),"page_start":1,"page_end":1,"order":0,"review_status":DRAFT,"raw_excerpt":text,"raw_fragments":[{"block_id":block,"byte_start":0,"byte_end":text.len()}]});
    let document_metadata = json!({"id":document,"source_file":"public-fixture.pdf","source_sha256":source_hash,"pages":1,"chunk_policy":"page-block-groups-v1"});
    (chunk, document, metadata, document_metadata)
}
#[test]
fn public_provenance_is_bound_to_text_fragments_and_policy() {
    let (chunk, document, mut metadata, mut document_metadata) = public_provenance();
    let source_hash = hash(b"authored synthetic source bytes");
    let text = "Authored public passage about hypothetical study notes.";
    let content_hash = hash(text.as_bytes());
    assert!(verify_provenance(
        &chunk,
        &document,
        "public-fixture.pdf",
        &source_hash,
        &content_hash,
        text,
        1,
        1,
        0,
        &metadata,
        &document_metadata,
        3000
    )
    .is_ok());
    metadata["raw_fragments"][0]["byte_end"] = json!(2);
    assert!(verify_provenance(
        &chunk,
        &document,
        "public-fixture.pdf",
        &source_hash,
        &content_hash,
        text,
        1,
        1,
        0,
        &metadata,
        &document_metadata,
        3000
    )
    .is_err());
    let (_, _, restored, _) = public_provenance();
    metadata = restored;
    document_metadata["chunk_policy"] = json!("isolated-layout-v1");
    assert!(verify_provenance(
        &chunk,
        &document,
        "public-fixture.pdf",
        &source_hash,
        &content_hash,
        text,
        1,
        1,
        0,
        &metadata,
        &document_metadata,
        3000
    )
    .is_err());
}
#[test]
fn compact_serialization_matches_adapter_map_order() {
    let response = SearchResponse {
        collection: COLLECTION.to_string(),
        model: MODEL.to_string(),
        revision: REVISION.to_string(),
        profile: PROFILE.to_string(),
        status: "no_evidence".to_string(),
        notice: "public fixture".to_string(),
        evidence: Vec::new(),
        used_tokens: 42,
    };
    let value = match serde_json::to_value(&response) {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    let adapter = match serde_json::to_string(&value) {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    assert_eq!(response.to_json(), Ok(adapter));
}
#[tokio::test]
async fn cancelled_query_guard_terminates_driver() {
    let task = tokio::spawn(std::future::pending::<()>());
    {
        let _guard = QueryGuard {
            task: &task,
            completed: false,
        };
    }
    let result = task.await;
    assert!(result.is_err());
}
#[tokio::test]
#[ignore = "requires explicitly configured local reader, Harrier and tutor services"]
async fn live_local_smoke() {
    let database_url = env::var("RETRIEVAL_DATABASE_URL")
        .expect("set RETRIEVAL_DATABASE_URL explicitly for this ignored real-service test");
    let embedding_url = env::var("HARRIER_URL")
        .expect("set HARRIER_URL explicitly for this ignored real-service test");
    let tutor_url =
        env::var("TUTOR_URL").expect("set TUTOR_URL explicitly for this ignored real-service test");
    let defaults = RetrievalDefaults {
        database_url: &database_url,
        embedding_url: &embedding_url,
        tutor_url: &tutor_url,
        limits: limits(),
    };
    let retrieval = match Retrieval::from_env_with(defaults).await {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    let response = match retrieval.search("What does a stop order do?", 4000).await {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    assert!(!response.evidence.is_empty());
    assert!(response.evidence.len() <= 5);
    assert!(response.used_tokens <= 4000);
    let serialized = match response.to_json() {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    let counted = match retrieval
        .http
        .tokens(&retrieval.tutor_url, &serialized, false)
        .await
    {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    assert_eq!(counted, response.used_tokens);
    for passage in &response.evidence {
        let counted = match retrieval
            .http
            .tokens(&retrieval.tutor_url, &passage.excerpt, false)
            .await
        {
            Ok(value) => value,
            Err(error) => panic!("{}", error),
        };
        assert!(counted <= limits().max_excerpt_tokens);
    }
    let limited = match retrieval.search("What does a stop order do?", 400).await {
        Ok(value) => value,
        Err(error) => panic!("{}", error),
    };
    assert!(limited.used_tokens <= 400);
    assert_eq!(limited.status, "budget_limited");
    assert!(limited.evidence.is_empty());
    assert!(retrieval.search("public question", 150).await.is_err());
    assert!(retrieval.search("public question", 0).await.is_err());
    retrieval.shutdown().await;
}

#[test]
fn caller_defaults_are_valid_local_reader_and_service_settings() {
    // Synthetic settings test parameter plumbing without copying the classroom credential.
    let defaults = RetrievalDefaults {
        database_url:
            "host=127.0.0.1 user=retrieval_reader dbname=harrier port=55432 password=fixture",
        embedding_url: "http://127.0.0.1:9999",
        tutor_url: "http://127.0.0.1:8080",
        limits: limits(),
    };
    let config = match database_config(defaults.database_url, defaults.limits.sql_timeout) {
        Ok(config) => config,
        Err(failure) => panic!("{}", failure),
    };
    assert_eq!(config.get_user(), Some("retrieval_reader"));
    assert_eq!(config.get_dbname(), Some("harrier"));
    assert_eq!(config.get_ports(), &[55432]);
    assert!(config.get_password().is_some());
    for (raw, expected_port) in [(defaults.embedding_url, 9999), (defaults.tutor_url, 8080)] {
        let url = match validate_local_url(raw) {
            Ok(url) => url,
            Err(failure) => panic!("{}", failure),
        };
        assert_eq!(url.port(), Some(expected_port));
    }
    // Use an invalid environment name: it cannot be set, so no process-global mutation is needed.
    assert_eq!(
        environment("LESSON_DEFAULT_TEST=ABSENT", defaults.database_url),
        Ok(defaults.database_url.to_string())
    );
    assert_eq!(
        environment("LESSON_DEFAULT_TEST=ABSENT", defaults.embedding_url),
        Ok(defaults.embedding_url.to_string())
    );
    assert_eq!(
        environment("LESSON_DEFAULT_TEST=ABSENT", defaults.tutor_url),
        Ok(defaults.tutor_url.to_string())
    );
}
