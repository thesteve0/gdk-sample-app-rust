//! Supplied catalog infrastructure: never runs the tutor, grades answers or accesses learner files.
use std::{
    collections::HashSet,
    env, fs,
    io::{self, IsTerminal},
    time::Duration,
};

use reqwest::{Client, Method};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const SERVER: &str = "http://127.0.0.1:5000";
const MANIFEST: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/lessons/14-agent-evaluation/datasets/visible-v1.json"
);

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: String,
    name: String,
    description: String,
    records: Vec<Value>,
}

// Presentation only; stdout/stderr gating is independent, as in the lesson application.
#[derive(Clone, Copy)]
enum TextStyle {
    Heading,
    Green,
    Error,
}
fn color_enabled(is_terminal: bool, no_color: bool, dumb_terminal: bool) -> bool {
    is_terminal && !no_color && !dumb_terminal
}
fn terminal_color_enabled(is_terminal: bool) -> bool {
    color_enabled(
        is_terminal,
        env::var_os("NO_COLOR").is_some(),
        env::var_os("TERM") == Some("dumb".into()),
    )
}
fn styled(text: &str, style: TextStyle, enabled: bool) -> String {
    if !enabled {
        return text.to_string();
    }
    let code = match style {
        TextStyle::Heading => "1;36",
        TextStyle::Green => "32",
        TextStyle::Error => "1;31",
    };
    format!("\x1b[{}m{}\x1b[0m", code, text)
}
fn print_line(text: &str, style: TextStyle) {
    println!(
        "{}",
        styled(
            text,
            style,
            terminal_color_enabled(io::stdout().is_terminal())
        )
    );
}

#[tokio::main]
async fn main() {
    let result = run().await;
    if let Err(error) = result {
        let failure = format!("Dataset operation failed: {}", error);
        eprintln!(
            "{}",
            styled(
                &failure,
                TextStyle::Error,
                terminal_color_enabled(io::stderr().is_terminal())
            )
        );
        std::process::exit(1);
    }
}

fn arguments(values: &[String]) -> Result<(String, bool), String> {
    let usage =
        "Usage: cargo run --example seed_mlflow_dataset -- <numeric-experiment-id> [--check]";
    if values.is_empty() || values.len() > 2 {
        return Err(usage.to_string());
    }
    let experiment = &values[0];
    if experiment.is_empty() || !experiment.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err("Experiment ID must contain ASCII digits only".to_string());
    }
    let check = values.len() == 2;
    if check && values[1] != "--check" {
        return Err(usage.to_string());
    }
    Ok((experiment.clone(), check))
}

fn read_manifest(bytes: &[u8]) -> Result<Manifest, String> {
    let parse_result = serde_json::from_slice::<Manifest>(bytes);
    let manifest = match parse_result {
        Ok(manifest) => manifest,
        Err(error) => {
            let failure = format!("Invalid manifest: {}", error);
            return Err(failure);
        }
    };
    if manifest.name != "lesson14-visible-v1"
        || manifest.schema_version != "lesson14-visible-v1"
        || manifest.description.is_empty()
    {
        return Err("Expected the authored lesson14-visible-v1 manifest".to_string());
    }
    if manifest.records.len() != 10 {
        return Err("Expected ten visible cases, no held-back records".to_string());
    }
    let mut seen = HashSet::new();
    for record in &manifest.records {
        let object = match record.as_object() {
            Some(object) => object,
            None => return Err("Record must be an object".to_string()),
        };
        if object.len() != 3
            || !record["inputs"].is_object()
            || !record["expectations"].is_object()
            || !record["tags"].is_object()
        {
            return Err("Record requires only inputs, expectations and tags objects".to_string());
        }
        let id = match record["inputs"]["case_id"].as_str() {
            Some(id) => id,
            None => return Err("Missing case ID".to_string()),
        };
        if !seen.insert(id.to_string())
            || record["tags"]["case_id"] != id
            || record["tags"]["review_status"] != "unreviewed"
        {
            return Err("Case IDs must be unique and tagged; review starts unreviewed".to_string());
        }
        let tags = record["tags"].as_object().expect("validated tags object");
        if !tags.values().all(Value::is_string) {
            return Err("Record tag values must be strings".to_string());
        }
        if !record["inputs"]["setup"].is_object()
            || !record["inputs"]["learner_turns"].is_array()
            || !record["expectations"]["inspection_targets"].is_array()
        {
            return Err("Each case needs setup, learner_turns and inspection_targets".to_string());
        }
    }
    for number in 1..=10 {
        let expected = format!("V{:02}", number);
        if !seen.contains(&expected) {
            return Err("Only V01 through V10 are permitted".to_string());
        }
    }
    Ok(manifest)
}

// Protobuf represents these JSON objects/lists as strings in this pinned REST API.
fn json_field(value: &Value, field: &str) -> anyhow::Result<Value> {
    let raw = value
        .get(field)
        .ok_or_else(|| anyhow::anyhow!("Missing API field {}", field))?;
    match raw.as_str() {
        Some(text) => Ok(serde_json::from_str(text)?),
        None => Ok(raw.clone()),
    }
}

async fn api(
    client: &Client,
    method: Method,
    path: &str,
    body: Option<Value>,
) -> anyhow::Result<Value> {
    let url = format!("{}/api/3.0/mlflow/{}", SERVER, path);
    let mut request = client.request(method, url);
    if let Some(body) = body {
        request = request.json(&body);
    }
    let response = request.send().await?;
    let status = response.status();
    // Do not echo arbitrary server response text into the terminal.
    if !status.is_success() {
        return Err(anyhow::anyhow!("MLflow {} returned HTTP {}", path, status));
    }
    Ok(response.json().await?)
}

async fn datasets(client: &Client, name: &str) -> anyhow::Result<Vec<Value>> {
    let mut found = Vec::new();
    let mut token = String::new();
    let mut seen_tokens = HashSet::new();
    loop {
        let response = api(client, Method::POST, "datasets/search", Some(json!({
            "filter_string": format!("name = '{}'", name), "max_results": 100, "page_token": token
        }))).await?;
        if let Some(values) = response["datasets"].as_array() {
            found.extend(values.clone());
        }
        token = response["next_page_token"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if token.is_empty() {
            break;
        }
        if !seen_tokens.insert(token.clone()) {
            return Err(anyhow::anyhow!("Repeated dataset search page token"));
        }
    }
    Ok(found)
}

async fn records(client: &Client, id: &str) -> anyhow::Result<Vec<Value>> {
    let mut found = Vec::new();
    let mut token = String::new();
    let mut seen_tokens = HashSet::new();
    loop {
        let mut url = reqwest::Url::parse(&format!(
            "{}/api/3.0/mlflow/datasets/{}/records",
            SERVER, id
        ))?;
        url.query_pairs_mut().append_pair("max_results", "100");
        if !token.is_empty() {
            url.query_pairs_mut().append_pair("page_token", &token);
        }
        let response = client
            .get(url)
            .send()
            .await?
            .error_for_status()?
            .json::<Value>()
            .await?;
        let values = json_field(&response, "records")?;
        let values = values
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Records response is not an array"))?;
        found.extend(values.clone());
        token = response["next_page_token"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if token.is_empty() {
            break;
        }
        if !seen_tokens.insert(token.clone()) {
            return Err(anyhow::anyhow!("Repeated record page token"));
        }
    }
    Ok(found)
}

// Check the entire existing set before any upsert; do not overwrite instructor edits.
fn missing_records(expected: &[Value], existing: &[Value]) -> Result<Vec<Value>, String> {
    let mut seen = HashSet::new();
    for record in existing {
        let id = match record["inputs"]["case_id"].as_str() {
            Some(id) => id,
            None => {
                return Err(
                    "Existing record lacks authored case ID; refusing to modify dataset"
                        .to_string(),
                )
            }
        };
        if !seen.insert(id) {
            return Err("Duplicate existing case ID; refusing to modify dataset".to_string());
        }
        let authored = match expected.iter().find(|item| item["inputs"]["case_id"] == id) {
            Some(authored) => authored,
            None => return Err("Extra existing record; refusing to modify dataset".to_string()),
        };
        if record
            .get("outputs")
            .is_some_and(|outputs| !outputs.is_null() && outputs != &json!({}))
            || record
                .get("source_id")
                .is_some_and(|value| !value.is_null())
        {
            return Err("Existing observed outputs or trace provenance; preserve them in a separate dataset".to_string());
        }
        for field in ["inputs", "expectations", "tags"] {
            if record[field] != authored[field] {
                let failure = format!("Existing {} {} differs from manifest; preserve it and use a new dataset version", id, field);
                return Err(failure);
            }
        }
    }
    Ok(expected
        .iter()
        .filter(|item| {
            !seen.contains(
                item["inputs"]["case_id"]
                    .as_str()
                    .expect("validated case ID"),
            )
        })
        .cloned()
        .collect())
}

fn validate_dataset(dataset: &Value, hash: &str) -> anyhow::Result<String> {
    let tags = json_field(dataset, "tags")?;
    if tags["manifest_sha256"] != hash || tags["schema_version"] != "lesson14-visible-v1" {
        return Err(anyhow::anyhow!("Dataset name exists with a different manifest identity; use a new version, never overwrite it"));
    }
    let id = dataset["dataset_id"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing dataset ID"))?;
    if !id.starts_with("d-")
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(anyhow::anyhow!("Unexpected dataset ID format"));
    }
    Ok(id.to_string())
}

async fn run() -> anyhow::Result<()> {
    let values: Vec<String> = env::args().skip(1).collect();
    let (experiment, check) = match arguments(&values) {
        Ok(parsed) => parsed,
        Err(failure) => return Err(anyhow::Error::msg(failure)),
    };
    let bytes = fs::read(MANIFEST)?;
    let manifest = match read_manifest(&bytes) {
        Ok(manifest) => manifest,
        Err(failure) => return Err(anyhow::Error::msg(failure)),
    };
    let hash = format!("{:x}", Sha256::digest(&bytes));
    let client = Client::builder()
        .no_proxy()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(20))
        .build()?;
    let version = client
        .get(format!("{}/version", SERVER))
        .send()
        .await?
        .error_for_status()?
        .text()
        .await?;
    if version.trim() != "3.17.1" {
        return Err(anyhow::anyhow!(
            "Requires pinned MLflow 3.17.1; no writes performed"
        ));
    }
    let experiment_info = client
        .get(format!(
            "{}/api/2.0/mlflow/experiments/get?experiment_id={}",
            SERVER, experiment
        ))
        .send()
        .await?
        .error_for_status()?
        .json::<Value>()
        .await?;
    if experiment_info["experiment"]["lifecycle_stage"] != "active" {
        return Err(anyhow::anyhow!("Experiment must exist and be active"));
    }
    let matches = datasets(&client, &manifest.name).await?;
    if matches.len() > 1 {
        return Err(anyhow::anyhow!(
            "Multiple datasets share this name; refusing ambiguous writes"
        ));
    }
    let dataset = match matches.first() {
        Some(dataset) => dataset.clone(),
        None => {
            if check {
                return Err(anyhow::anyhow!("Dataset absent; --check never creates it"));
            }
            let tags = json!({"manifest_sha256": hash, "schema_version": manifest.schema_version, "rubric_version":"v1", "purpose":"instructor-led catalog; not execution evidence", "source":"course-authored; visible cases only"});
            let response = api(&client, Method::POST, "datasets/create", Some(json!({"name":manifest.name,"experiment_ids":[experiment],"tags":serde_json::to_string(&tags)?}))).await?;
            response["dataset"].clone()
        }
    };
    let id = validate_dataset(&dataset, &hash)?;
    let association = api(
        &client,
        Method::GET,
        &format!("datasets/{}/experiment-ids", id),
        None,
    )
    .await?;
    if !association["experiment_ids"]
        .as_array()
        .is_some_and(|ids| ids.iter().any(|item| item == &experiment))
    {
        return Err(anyhow::anyhow!(
            "Dataset belongs to a different experiment; no association or records changed"
        ));
    }
    let existing = records(&client, &id).await?;
    let missing = match missing_records(&manifest.records, &existing) {
        Ok(missing) => missing,
        Err(failure) => return Err(anyhow::Error::msg(failure)),
    };
    if check && !missing.is_empty() {
        return Err(anyhow::anyhow!(
            "Dataset incomplete; --check never adds records"
        ));
    }
    if !missing.is_empty() {
        api(
            &client,
            Method::POST,
            &format!("datasets/{}/records", id),
            Some(json!({"records":serde_json::to_string(&missing)?})),
        )
        .await?;
    }
    let saved = records(&client, &id).await?;
    let remaining = match missing_records(&manifest.records, &saved) {
        Ok(remaining) => remaining,
        Err(failure) => return Err(anyhow::Error::msg(failure)),
    };
    if !remaining.is_empty() {
        return Err(anyhow::anyhow!(
            "Readback incomplete; rerun can resume an interrupted seed"
        ));
    }
    print_line(
        "Native MLflow visible evaluation catalog",
        TextStyle::Heading,
    );
    println!(
        "Dataset: {}\nDataset ID: {}\nExperiment: {}\nManifest SHA-256: {}",
        manifest.name, id, experiment, hash
    );
    let status = format!("Verified {} authored records; {} added, {} already present. No tutor execution or scoring.", saved.len(), missing.len(), existing.len());
    print_line(&status, TextStyle::Green);
    for record in saved {
        println!(
            "{}: {}",
            record["inputs"]["case_id"].as_str().unwrap_or("?"),
            record["dataset_record_id"].as_str().unwrap_or("?")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn manifest() -> Manifest {
        read_manifest(include_bytes!("datasets/visible-v1.json")).unwrap()
    }
    #[test]
    fn authored_ten_cases_and_no_observed_outputs() {
        let manifest = manifest();
        assert_eq!(manifest.records.len(), 10);
        assert!(manifest
            .records
            .iter()
            .all(|item| item.get("outputs").is_none()));
        assert_eq!(
            manifest.records[1]["expectations"]["calculator"]["amount"],
            "100.00"
        );
        assert_eq!(manifest.records[9]["inputs"]["execution_status"], "blocked");
        assert_eq!(
            manifest.records[3]["inputs"]["separate_probe"]["status"],
            "blocked"
        );
    }
    #[test]
    fn invalid_manifest_and_held_back_rejected() {
        assert!(read_manifest(b"{}").is_err());
        let mut value: Value =
            serde_json::from_slice(include_bytes!("datasets/visible-v1.json")).unwrap();
        value["records"][0]["inputs"]["case_id"] = json!("H01");
        assert!(read_manifest(&serde_json::to_vec(&value).unwrap()).is_err());
    }
    #[test]
    fn numeric_arguments_and_read_only_mode() {
        assert_eq!(
            arguments(&["1".into(), "--check".into()]).unwrap(),
            ("1".into(), true)
        );
        for values in [
            vec![],
            vec!["../1".into()],
            vec!["1".into(), "--force".into()],
            vec!["１２".into()],
        ] {
            assert!(arguments(&values).is_err());
        }
    }
    #[test]
    fn idempotent_and_resumable_without_overwrites() {
        let expected = manifest().records;
        assert_eq!(missing_records(&expected, &[]).unwrap().len(), 10);
        assert_eq!(missing_records(&expected, &expected[..4]).unwrap().len(), 6);
        assert!(missing_records(&expected, &expected).unwrap().is_empty());
        let mut edited = expected.clone();
        edited[0]["expectations"]["changed"] = json!(true);
        assert!(missing_records(&expected, &edited).is_err());
        edited = expected.clone();
        edited[0]["inputs"]["case_id"] = json!("extra");
        assert!(missing_records(&expected, &edited).is_err());
        edited = expected.clone();
        edited.push(edited[0].clone());
        assert!(missing_records(&expected, &edited).is_err());
    }
    #[test]
    fn observed_outputs_and_imported_provenance_are_not_authored_catalog() {
        let expected = manifest().records;
        let mut saved = expected.clone();
        saved[0]["outputs"] = json!({});
        assert!(missing_records(&expected, &saved).unwrap().is_empty());
        saved[0]["outputs"] = json!({"answer":"An observed answer, not an authored expectation"});
        assert!(missing_records(&expected, &saved).is_err());
        saved[0]["outputs"] = json!({});
        saved[0]["source_id"] = json!("tr-imported");
        assert!(missing_records(&expected, &saved).is_err());
    }
    #[test]
    fn json_wire_fields_and_manifest_mismatch() {
        assert_eq!(
            json_field(&json!({"records":"[]"}), "records").unwrap(),
            json!([])
        );
        assert!(json_field(&json!({}), "records").is_err());
        let dataset = json!({"dataset_id":"d-123","tags":"{\"manifest_sha256\":\"abc\",\"schema_version\":\"lesson14-visible-v1\"}"});
        assert_eq!(validate_dataset(&dataset, "abc").unwrap(), "d-123");
        assert!(validate_dataset(&dataset, "different").is_err());
    }
    #[test]
    fn presentation_palette_and_terminal_gating() {
        assert_eq!(styled("ok", TextStyle::Green, true), "\x1b[32mok\x1b[0m");
        assert_eq!(
            styled("head", TextStyle::Heading, true),
            "\x1b[1;36mhead\x1b[0m"
        );
        assert_eq!(
            styled("fail", TextStyle::Error, true),
            "\x1b[1;31mfail\x1b[0m"
        );
        assert_eq!(styled("ok", TextStyle::Green, false), "ok");
        for (terminal, no_color, dumb) in [
            (false, false, false),
            (true, true, false),
            (true, false, true),
        ] {
            assert!(!color_enabled(terminal, no_color, dumb));
        }
        assert!(color_enabled(true, false, false));
    }
}
