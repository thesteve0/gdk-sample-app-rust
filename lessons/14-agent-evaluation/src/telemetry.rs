// Lesson 14
//! Supplied local trace transport, not an evaluation engine or full Goose CLI exporter.
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::{trace::SdkTracerProvider, Resource};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    path::PathBuf,
    time::Duration,
};
use tracing_subscriber::prelude::*;

pub fn initialize() -> anyhow::Result<Option<SdkTracerProvider>> {
    let experiment = match env::var("MLFLOW_EXPERIMENT_ID") {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if experiment.is_empty() || !experiment.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(anyhow::anyhow!("MLFLOW_EXPERIMENT_ID must be numeric"));
    }
    for variable in [
        "OTEL_EXPORTER_OTLP_HEADERS",
        "OTEL_EXPORTER_OTLP_TRACES_HEADERS",
        "OTEL_RESOURCE_ATTRIBUTES",
    ] {
        if env::var_os(variable).is_some() {
            return Err(anyhow::anyhow!(
                "unset {}: this lesson supplies its own nonsecret headers/resource",
                variable
            ));
        }
    }
    // Fixed local transport: never forward configuration credentials or use a hosted judge.
    let client = reqwest::blocking::Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(10))
        .build()?;
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_http_client(client)
        .with_endpoint("http://127.0.0.1:5000/v1/traces")
        .with_headers(HashMap::from([(
            "x-mlflow-experiment-id".to_string(),
            experiment,
        )]))
        .build()?;
    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(
            Resource::builder()
                .with_service_name("gdk-learning-partner")
                .build(),
        )
        .build();
    let tracer = provider.tracer("lesson14");
    // No formatting subscriber: ordinary learner output must not contain telemetry internals.
    tracing_subscriber::registry()
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;
    Ok(Some(provider))
}

pub fn shutdown(provider: Option<SdkTracerProvider>) -> anyhow::Result<()> {
    if let Some(provider) = provider {
        let flush = provider.force_flush();
        let shutdown = provider.shutdown();
        flush?;
        shutdown?;
    }
    Ok(())
}

// The optional evaluation name isolates BOTH memory and overwritten diagnostics.
// Storage still resolves against the repository, never a subprocess working directory.
pub fn storage_path(file: &str) -> PathBuf {
    let root = PathBuf::from(super::REPOSITORY_ROOT);
    match env::var("EVALUATION_STORAGE") {
        Ok(name) => root
            .join(".evaluation")
            .join("sessions")
            .join(name)
            .join(file),
        Err(_) => root.join(file),
    }
}

pub fn prepare_storage() -> anyhow::Result<()> {
    if let Ok(name) = env::var("EVALUATION_STORAGE") {
        validate_storage_name(&name)?;
        let directory = storage_path("learner-record.md")
            .parent()
            .expect("fixed file parent")
            .to_path_buf();
        let root = PathBuf::from(super::REPOSITORY_ROOT);
        for ancestor in [
            root.join(".evaluation"),
            root.join(".evaluation/sessions"),
            directory.clone(),
        ] {
            if let Ok(metadata) = std::fs::symlink_metadata(&ancestor) {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    return Err(anyhow::anyhow!(
                        "evaluation storage ancestors must be regular directories"
                    ));
                }
            }
        }
        std::fs::create_dir_all(&directory)?;
        for file in ["learner-record.md", "session-output.md"] {
            if let Ok(metadata) = std::fs::symlink_metadata(directory.join(file)) {
                if metadata.file_type().is_symlink() || !metadata.is_file() {
                    return Err(anyhow::anyhow!(
                        "evaluation storage files must be regular files"
                    ));
                }
            }
        }
        if std::fs::symlink_metadata(&directory)?
            .file_type()
            .is_symlink()
        {
            return Err(anyhow::anyhow!(
                "evaluation directory must not be a symlink"
            ));
        }
    }
    Ok(())
}

fn validate_storage_name(name: &str) -> anyhow::Result<()> {
    if name.is_empty()
        || name.len() > 80
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
    {
        return Err(anyhow::anyhow!(
            "EVALUATION_STORAGE must be 1..80 ASCII letters, digits, '-' or '_'"
        ));
    }
    Ok(())
}

// Only observable ordinary dialogue/tool data; no provider reasoning or Message metadata.
pub fn observable(messages: &[goose_providers::conversation::message::Message]) -> String {
    use goose_providers::conversation::message::MessageContent;
    let records: Vec<_> = messages
        .iter()
        .map(|message| {
            let content: Vec<_> = message
                .content
                .iter()
                .filter_map(|block| match block {
                    MessageContent::Text(text) => Some(serde_json::json!({"text":text.text})),
                    MessageContent::ToolRequest(request) => {
                        Some(serde_json::json!({"request_id":request.id,"call":request.tool_call}))
                    }
                    MessageContent::ToolResponse(response) => Some(
                        serde_json::json!({"request_id":response.id,"result":response.tool_result}),
                    ),
                    _ => None,
                })
                .collect();
            serde_json::json!({"message_id":message.id,"speaker":message.role,"content":content})
        })
        .collect();
    serde_json::json!(records).to_string()
}

pub fn identify_current_span() -> anyhow::Result<()> {
    use sha2::{Digest, Sha256};
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let span = tracing::Span::current();
    if span.is_disabled() {
        return Ok(());
    }
    let root = PathBuf::from(super::REPOSITORY_ROOT);
    let paths = [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "src/main.rs",
        "src/calculator.rs",
        "src/chooser.rs",
        "src/diagnostics.rs",
        "src/learning_record.rs",
        "src/presentation.rs",
        "src/provider_setup.rs",
        "src/retrieval.rs",
        "src/compaction.rs",
        "src/telemetry.rs",
        "lessons/14-agent-evaluation/RUBRIC.md",
        "lessons/14-agent-evaluation/SCENARIOS.md",
    ];
    let mut identities = BTreeMap::new();
    for path in paths {
        let bytes = std::fs::read(root.join(path))?;
        identities.insert(path.to_string(), format!("{:x}", Sha256::digest(bytes)));
    }
    for file in [
        "tutor-contract.md",
        "getting-started-material.md",
        "getting-started-guidance.md",
        "compaction-guidance.md",
    ] {
        let path = format!("{}/{}", super::CONTENT_DIRECTORY, file);
        let bytes = std::fs::read(root.join(&path))?;
        identities.insert(path, format!("{:x}", Sha256::digest(bytes)));
    }
    span.set_attribute(
        "course.content_sha256",
        serde_json::json!(identities).to_string(),
    );

    span.set_attribute("course.collection", super::retrieval::COLLECTION);
    span.set_attribute(
        "course.embedding_profile",
        super::retrieval::profile_identity(),
    );
    span.set_attribute(
        "course.compaction_trigger",
        env::var("COMPACTION_TRIGGER_BYTES").unwrap_or("60000".to_string()),
    );
    span.set_attribute("course.allowance_bytes", 84000i64);
    span.set_attribute("course.rubric_version", "v1");
    span.set_attribute("course.case_version", "v1");
    span.set_attribute(
        "course.case",
        env::var("EVALUATION_CASE").unwrap_or("unspecified".to_string()),
    );
    span.set_attribute(
        "course.comparison",
        env::var("EVALUATION_COMPARISON").unwrap_or("unspecified".to_string()),
    );
    Ok(())
}

pub fn inference_input(
    messages: &[goose_providers::conversation::message::Message],
    prompt: &str,
    tools: &[rmcp::model::Tool],
) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let span = tracing::Span::current();
    if span.is_disabled() {
        return;
    }
    span.set_attribute("mlflow.spanInputs", serde_json::json!({"ordinary_messages":observable(messages),"application_instruction":prompt,"capabilities":tools}).to_string());
}

pub fn saved_message(message: &goose_providers::conversation::message::Message) {
    if tracing::Span::current().is_disabled() {
        return;
    }
    tracing::info!("course.saved_observable_message" = observable(std::slice::from_ref(message)));
}

pub fn maintenance_output(report: &super::compaction::Report) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    tracing::Span::current().set_attribute("mlflow.spanOutputs", serde_json::json!({"disposition":report.disposition,"before_bytes":report.before_bytes,"after_bytes":report.after_bytes,"counting":report.counting_method,"summary":report.summary,"covered_ids":report.covered_message_ids}).to_string());
}
// alpha.11 converts each assembled output Message to an Effect inside its chat span.
// Preserve every projected message as an Event; the output field shows the last Message.
pub fn inference_message(message: &goose_providers::conversation::message::Message) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let span = tracing::Span::current();
    if span.is_disabled() {
        return;
    }
    if let Some(metadata) = span.metadata() {
        if metadata.name() == "chat" {
            let text = observable(std::slice::from_ref(message));
            span.set_attribute("mlflow.spanOutputs", text.clone());
            tracing::info!("course.inference_output_message" = text);
        }
    }
}

pub fn identity_context(session_id: &str) {
    use tracing_opentelemetry::OpenTelemetrySpanExt;
    let span = tracing::Span::current();
    span.set_attribute("course.session_id", session_id.to_string());
    span.set_attribute(
        "course.storage",
        env::var("EVALUATION_STORAGE").unwrap_or("ordinary-root".to_string()),
    );
    span.set_attribute(
        "course.case",
        env::var("EVALUATION_CASE").unwrap_or("unspecified".to_string()),
    );
    span.set_attribute(
        "course.comparison",
        env::var("EVALUATION_COMPARISON").unwrap_or("unspecified".to_string()),
    );
}

pub struct SummaryPreparer;
#[async_trait::async_trait]
impl<S: Sync> goose_agent::inference::InferenceRequestPreparer<S> for SummaryPreparer {
    async fn prepare(
        &self,
        _session: &S,
        conversation: &goose_providers::conversation::Conversation,
        input: goose_agent::operation::InferenceInput,
    ) -> anyhow::Result<goose_agent::inference::PreparedInferenceRequest> {
        let prompt = input
            .prompt_parts
            .into_iter()
            .map(|(_name, text)| text)
            .collect::<Vec<_>>()
            .join("\n\n");
        inference_input(conversation.messages(), &prompt, &input.tools);
        Ok(goose_agent::inference::PreparedInferenceRequest {
            system_prompt: prompt,
            tools: input.tools,
            additional_messages: Vec::new(),
        })
    }
}

#[cfg(test)]
mod tests;
