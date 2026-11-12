// Lesson 13
// Rag calibration harness (instructor experimentation tool; not part of the tutor).
// Uses the application's retrieval library so calibration exercises the
// exact same embedding, SQL, provenance, and budget path the tutor will use.
// Feedback goes to rag-calibration.jsonl (gitignored; no excerpts are written to it).
#![allow(clippy::too_many_lines)]

// The application supplies these defaults explicitly; environment variables
// still override them, exactly as in the tutor application.
const RETRIEVAL_DATABASE_URL: &str = "postgresql://retrieval_reader:7297cb0587b95545f039df77dc32684e75f73093a444895c@127.0.0.1:55432/harrier";
const HARRIER_URL: &str = "http://127.0.0.1:9999";
const TUTOR_URL: &str = "http://127.0.0.1:8080";
const MAX_QUERY_BYTES: usize = 2000;
const HTTP_LIMIT: usize = 2 * 1024 * 1024;
const SQL_TIMEOUT: Duration = Duration::from_secs(5);
const HTTP_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_EXCERPT_TOKENS: usize = 1000;
const TOKEN_ACCOUNTING_ATTEMPTS: usize = 8;

use gdk_hello::rag::retrieval;

use serde::Serialize;
use std::{fs, time::Duration};

fn retrieval_limits() -> retrieval::RetrievalLimits {
    retrieval::RetrievalLimits {
        max_query_bytes: MAX_QUERY_BYTES,
        max_http_bytes: HTTP_LIMIT,
        sql_timeout: SQL_TIMEOUT,
        http_timeout: HTTP_TIMEOUT,
        max_excerpt_tokens: MAX_EXCERPT_TOKENS,
        token_accounting_attempts: TOKEN_ACCOUNTING_ATTEMPTS,
    }
}

const FEEDBACK_PATH: &str = "rag-calibration.jsonl";
// Show every SQL candidate regardless of serialized size so nothing is hidden
// from rating; the tutor itself uses a much smaller per-turn grounding budget.
const CALIBRATION_TOKEN_BUDGET: usize = 100_000;
const DEFAULT_THRESHOLD: f64 = 0.5;
const EXCERPT_DISPLAY_CHARS: usize = 280;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum Judgment {
    Keep,
    Drop,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct JudgmentRecord {
    rank: usize,
    chunk_id: String,
    similarity: f64,
    source_file: String,
    page_start: i32,
    page_end: i32,
    judgment: Judgment,
}

#[derive(Debug, Serialize, serde::Deserialize)]
struct FeedbackRecord {
    timestamp_unix_seconds: u64,
    query: String,
    collection: String,
    // Taken from the SearchResponse of the query being rated, so feedback is
    // always tied to the embedding profile that produced the scores.
    profile: String,
    threshold_marker: f64,
    judgments: Vec<JudgmentRecord>,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let threshold_marker = effective_threshold();
    println!("Rag calibration for collection {}", retrieval::COLLECTION);
    println!("Embedding profile is shown per query (taken from the search response).");
    println!(
        "Threshold marker used for SELECTED/BELOW marks: {:.2} (set GROUNDING_MIN_SIMILARITY to override)",
        threshold_marker
    );
    println!(
        "Feedback file: {} (append-only; no excerpts are stored)",
        FEEDBACK_PATH
    );
    println!("Commands: type a query, 'summary', or empty line to quit.");
    let retrieval = match retrieval::Retrieval::from_env_with(retrieval::RetrievalDefaults {
        database_url: RETRIEVAL_DATABASE_URL,
        embedding_url: HARRIER_URL,
        tutor_url: TUTOR_URL,
        limits: retrieval_limits(),
    })
    .await
    {
        Ok(retrieval) => retrieval,
        Err(failure) => {
            eprintln!("Cannot start retrieval services: {}", failure);
            std::process::exit(1);
        }
    };
    while let Some(query) = read_line("query> ") {
        let trimmed = query.trim().to_string();
        if trimmed.is_empty() {
            break;
        }
        if trimmed == "summary" {
            print_summary(FEEDBACK_PATH);
            continue;
        }
        run_calibration_query(&retrieval, &trimmed, threshold_marker);
    }
    let _ = retrieval.shutdown().await;
}

fn read_line(prompt: &str) -> Option<String> {
    use std::io::Write;
    print!("{}", prompt);
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    match std::io::stdin().read_line(&mut line) {
        Ok(0) => None,
        Ok(_count) => Some(line),
        Err(_error) => None,
    }
}

// Same policy range as the tutor application; the marker is advisory only.
fn effective_threshold() -> f64 {
    match std::env::var("GROUNDING_MIN_SIMILARITY") {
        Ok(raw) => match raw.trim().parse::<f64>() {
            Ok(value) if (0.0..=1.0).contains(&value) => value,
            _ => {
                eprintln!(
                    "Ignoring invalid GROUNDING_MIN_SIMILARITY '{}'; using {:.2}",
                    raw.trim(),
                    DEFAULT_THRESHOLD
                );
                DEFAULT_THRESHOLD
            }
        },
        Err(_error) => DEFAULT_THRESHOLD,
    }
}

fn file_name(source_file: &str) -> String {
    source_file
        .rsplit('/')
        .next()
        .unwrap_or(source_file)
        .to_string()
}

fn display_excerpt(text: &str) -> String {
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = collapsed.chars();
    let mut out = String::new();
    for _ in 0..EXCERPT_DISPLAY_CHARS {
        match chars.next() {
            Some(character) => out.push(character),
            None => return collapsed,
        }
    }
    if chars.next().is_some() {
        out.push('…');
    }
    out
}

fn run_calibration_query(retrieval: &retrieval::Retrieval, query: &str, threshold_marker: f64) {
    println!("Searching {} …", retrieval::COLLECTION);
    let response =
        match futures::executor::block_on(retrieval.search(query, CALIBRATION_TOKEN_BUDGET)) {
            Ok(response) => response,
            Err(failure) => {
                eprintln!("Search failed: {}", failure);
                return;
            }
        };
    if response.evidence.is_empty() {
        println!("No candidates returned (status {}).", response.status);
        return;
    }
    for (index, passage) in response.evidence.iter().enumerate() {
        let rank = index + 1;
        let marker = if passage.similarity >= threshold_marker {
            "SELECTED"
        } else {
            "below"
        };
        println!(
            "{:>2}. sim {:.4} [{}] {} pages {}-{} chunk {}",
            rank,
            passage.similarity,
            marker,
            file_name(&passage.source_file),
            passage.page_start,
            passage.page_end,
            short_chunk_id(&passage.chunk_id)
        );
        println!("    \"{}\"", display_excerpt(&passage.excerpt));
    }
    let judgments = collect_judgments(&response.evidence);
    if judgments.is_empty() {
        println!("No ratings recorded for this query.");
        return;
    }
    let record = FeedbackRecord {
        timestamp_unix_seconds: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0),
        query: query.to_string(),
        collection: retrieval::COLLECTION.to_string(),
        profile: response.profile.clone(),
        threshold_marker,
        judgments,
    };
    match append_feedback(FEEDBACK_PATH, &record) {
        Ok(()) => println!(
            "Recorded {} rating(s) to {}.",
            record.judgments.len(),
            FEEDBACK_PATH
        ),
        Err(failure) => eprintln!("Could not write feedback: {}", failure),
    }
}

fn short_chunk_id(chunk_id: &str) -> String {
    chunk_id.chars().take(12).collect()
}

// One rating line for all results: e.g. "k d s" rates 1=keep 2=drop 3=skip.
fn collect_judgments(evidence: &[retrieval::Evidence]) -> Vec<JudgmentRecord> {
    let answer = match read_line("rate 1..N as k/d/s (Enter skips all)> ") {
        Some(answer) => answer,
        None => return Vec::new(),
    };
    let mut records = Vec::new();
    for (index, token) in answer.split_whitespace().enumerate() {
        let judgment = match token.to_ascii_lowercase().as_str() {
            "k" | "keep" => Judgment::Keep,
            "d" | "drop" => Judgment::Drop,
            _ => continue,
        };
        let passage = match evidence.get(index) {
            Some(passage) => passage,
            None => break,
        };
        records.push(JudgmentRecord {
            rank: index + 1,
            chunk_id: passage.chunk_id.clone(),
            similarity: passage.similarity,
            source_file: passage.source_file.clone(),
            page_start: passage.page_start,
            page_end: passage.page_end,
            judgment,
        });
    }
    records
}

fn append_feedback(path: &str, record: &FeedbackRecord) -> Result<(), String> {
    let line = match serde_json::to_string(record) {
        Ok(line) => line,
        Err(_error) => return Err("feedback serialization failed".to_string()),
    };
    let result = fs::OpenOptions::new().create(true).append(true).open(path);
    let mut file = match result {
        Ok(file) => file,
        Err(error) => return Err(format!("cannot open {}: {}", path, error)),
    };
    use std::io::Write;
    match writeln!(file, "{}", line) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("cannot write {}: {}", path, error)),
    }
}

fn print_summary(path: &str) {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(_error) => {
            println!("No feedback file at {} yet.", path);
            return;
        }
    };
    let mut keeps: Vec<f64> = Vec::new();
    let mut drops: Vec<f64> = Vec::new();
    let mut queries = 0usize;
    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let record: FeedbackRecord = match serde_json::from_str(line) {
            Ok(record) => record,
            Err(_error) => continue,
        };
        queries += 1;
        for judgment in &record.judgments {
            match judgment.judgment {
                Judgment::Keep => keeps.push(judgment.similarity),
                Judgment::Drop => drops.push(judgment.similarity),
            }
        }
    }
    println!("--- Calibration summary ({} rated queries) ---", queries);
    print_class("keep", &keeps);
    print_class("drop", &drops);
    suggest_threshold(&keeps, &drops);
}

fn print_class(name: &str, values: &[f64]) {
    if values.is_empty() {
        println!("{}: no ratings", name);
        return;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let count = sorted.len();
    let middle = count / 2;
    let median = if count % 2 == 1 {
        sorted[middle]
    } else {
        (sorted[middle - 1] + sorted[middle]) / 2.0
    };
    println!(
        "{}: n={} min={:.4} median={:.4} max={:.4}",
        name,
        count,
        sorted[0],
        median,
        sorted[count - 1]
    );
}

fn suggest_threshold(keeps: &[f64], drops: &[f64]) {
    let keep_min = match keeps.iter().cloned().reduce(f64::min) {
        Some(value) => value,
        None => {
            println!("suggestion: rate at least one keep to calibrate a threshold");
            return;
        }
    };
    let drop_max = match drops.iter().cloned().reduce(f64::max) {
        Some(value) => value,
        None => {
            println!(
                "suggestion: no drops rated; lowest keep is {:.4} — verify with more queries",
                keep_min
            );
            return;
        }
    };
    if drop_max < keep_min {
        let midpoint = (drop_max + keep_min) / 2.0;
        println!(
            "suggestion: clean separation; threshold between {:.4} and {:.4} (midpoint {:.2})",
            drop_max, keep_min, midpoint
        );
    } else {
        println!(
            "suggestion: overlap (drop max {:.4} >= keep min {:.4}); similarity alone cannot separate these — review overlapping queries",
            drop_max, keep_min
        );
    }
}
