//! Instructor preparation CLI. Loading keeps private passages off terminal output.
//! No corpus acceptance or semantic source validation is implied by successful loading.
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    io::{self, IsTerminal, Read, Write},
    path::{Component, Path, PathBuf},
    time::Duration,
};

use anyhow::{anyhow, bail, Context, Result};
use postgres::{config::Host, Client, Config, GenericClient, NoTls, Row};
use reqwest::{blocking::Client as HttpClient, redirect::Policy, Url};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

const MODEL: &str = "microsoft/harrier-oss-v1-0.6b";
const REVISION: &str = "f9b9dc8d367d443f2479d27aa5d8d2850c0774ee";
// MODEL/REVISION describe the original corpus model, not this GGUF derivative.
const RUNTIME_MODEL: &str = "TheSteve0/harrier-oss-v1-0.6b-GGUF";
const GGUF_SHA256: &str = "02b8bc790139170d3710b411777342945c0b36d35fdaf0e24684c2d3e135a23b";
const LLAMA_BUILD: &str = "b11429-d81235049";
// Full artifact/build pin makes old HF collections incompatible by construction.
const PROFILE: &str = "harrier-llamacpp-f16-last-l2-web-search-v1-gguf-02b8bc790139170d3710b411777342945c0b36d35fdaf0e24684c2d3e135a23b-build-b11429-d81235049";
const QUERY_PREFIX: &str =
    "Instruct: Given a web search query, retrieve relevant passages that answer the query\nQuery: ";
const DIMENSIONS: usize = 1024;
const MAX_TOKENS: usize = 32768;
const BATCH_SIZE: usize = 8;
const SCHEMA_SQL: &str = include_str!("../../sql/schema.sql");
const QUERY_SQL: &str = include_str!("../../sql/query.sql");
const INSERT_PASSAGE: &str = "INSERT INTO retrieval.passages
    (collection_id,chunk_id,document_id,chunk_order,page_start,page_end,
     content_sha256,passage,metadata,token_count,embedding)
    VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,CAST($11::text AS halfvec(1024)))";

#[derive(Debug, PartialEq)]
enum Command {
    Init,
    Load {
        run: PathBuf,
        collection: String,
    },
    Query {
        collection: String,
        text: String,
        top_k: i64,
    },
}

fn main() {
    if let Err(error) = run() {
        // Every external error is replaced at its boundary: never print URLs,
        // request bodies, database details, or private parsing fragments.
        print_notice(&format!("Harrier: {}", error), true);
        std::process::exit(1);
    }
}

// JSON stdout is never styled. Presentation codes are confined to terminal stderr.
fn style_enabled(terminal: bool, no_color: bool, term: &str) -> bool {
    terminal && !no_color && term != "dumb"
}

fn styled(text: &str, error: bool, enabled: bool) -> String {
    if !enabled {
        return text.to_string();
    }
    let code = if error { "1;31" } else { "32" };
    format!("\x1b[{}m{}\x1b[0m", code, text)
}

fn print_notice(text: &str, error: bool) {
    let enabled = style_enabled(
        io::stderr().is_terminal(),
        env::var_os("NO_COLOR").is_some(),
        &env::var("TERM").unwrap_or_default(),
    );
    eprintln!("{}", styled(text, error, enabled));
}

fn safe<T, E>(result: std::result::Result<T, E>, stage: &'static str) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(anyhow!(stage)),
    }
}

fn command(args: Vec<String>) -> Result<Command> {
    let mut args = args.into_iter();
    let name = args
        .next()
        .context("expected init, load, or query command")?;
    if name == "init" {
        if args.next().is_some() {
            bail!("init takes no options");
        }
        return Ok(Command::Init);
    }
    if name != "load" && name != "query" {
        bail!("expected init, load, or query command");
    }
    let mut options = HashMap::new();
    while let Some(flag) = args.next() {
        let allowed = if name == "load" {
            ["--run", "--collection"].contains(&flag.as_str())
        } else {
            ["--collection", "--text", "--top-k"].contains(&flag.as_str())
        };
        if !allowed || options.contains_key(&flag) {
            bail!("unknown or repeated command option");
        }
        let value = args.next().context("command option requires a value")?;
        options.insert(flag, value);
    }
    let collection = options
        .remove("--collection")
        .context("--collection is required")?;
    validate_collection(&collection)?;
    if name == "load" {
        let run = options
            .remove("--run")
            .context("load requires explicit --run PATH")?;
        if run.is_empty() {
            bail!("--run must not be empty");
        }
        return Ok(Command::Load {
            run: PathBuf::from(run),
            collection,
        });
    }
    let text = options
        .remove("--text")
        .context("query requires --text TEXT")?;
    if text.trim().is_empty() {
        bail!("query text must not be blank");
    }
    let top_k = match options.remove("--top-k") {
        Some(value) => safe(value.parse::<i64>(), "--top-k must be an integer")?,
        None => 5,
    };
    if !(1..=100).contains(&top_k) {
        bail!("--top-k must be 1..100");
    }
    Ok(Command::Query {
        collection,
        text,
        top_k,
    })
}

fn validate_collection(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 120
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_- .".contains(&byte))
        || name.trim() != name
    {
        bail!("collection name must be 1..120 ASCII letters, digits, spaces, dots, underscores, or hyphens");
    }
    Ok(())
}

fn database_config(raw: &str) -> Result<Config> {
    let mut config = safe(
        raw.parse::<Config>(),
        "DATABASE_URL configuration is invalid",
    )?;
    if config.get_hosts().is_empty() {
        bail!("DATABASE_URL must explicitly select a loopback TCP host");
    }
    for host in config.get_hosts() {
        match host {
            Host::Tcp(name) => {
                if !["127.0.0.1", "localhost", "::1", "[::1]"].contains(&name.as_str()) {
                    bail!("DATABASE_URL must use a literal loopback TCP host");
                }
            }
            _ => bail!("DATABASE_URL must use a literal loopback TCP host"),
        }
    }
    // A hostaddr override must not silently send NoTls traffic elsewhere.
    if config
        .get_hostaddrs()
        .iter()
        .any(|address| !address.is_loopback())
    {
        bail!("DATABASE_URL host addresses must be loopback");
    }
    config.connect_timeout(Duration::from_secs(15));
    Ok(config)
}

fn database() -> Result<Client> {
    let raw = safe(env::var("DATABASE_URL"), "DATABASE_URL is required")?;
    let config = database_config(&raw)?;
    safe(config.connect(NoTls), "database connection failed")
}

fn endpoint(raw: &str) -> Result<Url> {
    // Inspect the original authority too: URL parsing normalizes alternate IPv4
    // spellings such as 127.1, which this local-only contract does not permit.
    let (_, tail) = raw.split_once("://").context("HARRIER_URL is invalid")?;
    let authority = tail
        .split(['/', '?', '#'])
        .next()
        .context("HARRIER_URL is invalid")?;
    let host = if authority.starts_with('[') {
        let close = authority.find(']').context("HARRIER_URL is invalid")?;
        let remainder = &authority[close + 1..];
        if !remainder.is_empty() && !remainder.starts_with(':') {
            bail!("HARRIER_URL is invalid");
        }
        &authority[..=close]
    } else {
        authority
            .split(':')
            .next()
            .context("HARRIER_URL is invalid")?
    };
    if !["127.0.0.1", "localhost", "[::1]"].contains(&host) {
        bail!("HARRIER_URL must use literal 127.0.0.1, localhost, or [::1]");
    }
    let url = safe(Url::parse(raw), "HARRIER_URL is invalid")?;
    if !["http", "https"].contains(&url.scheme())
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || !["", "/"].contains(&url.path())
    {
        bail!("HARRIER_URL must be a loopback HTTP origin without credentials, path, query, or fragment");
    }
    Ok(url)
}

#[derive(Clone, Debug, Deserialize)]
struct Identity {
    model: String,
    revision: String,
    profile: String,
    dimensions: usize,
}

fn check_identity(identity: &Identity) -> Result<()> {
    if identity.model != MODEL
        || identity.revision != REVISION
        || identity.profile != PROFILE
        || identity.dimensions != DIMENSIONS
    {
        bail!("embedding identity does not match the pinned Harrier contract");
    }
    Ok(())
}

#[derive(Deserialize)]
struct Embedded {
    #[serde(flatten)]
    identity: Identity,
    embeddings: Vec<Vec<f32>>,
    token_counts: Vec<usize>,
}

fn check_vector(vector: &[f32]) -> Result<()> {
    if vector.len() != DIMENSIONS || vector.iter().any(|value| !value.is_finite()) {
        bail!("embedding vector has invalid dimensions or nonfinite values");
    }
    let squared_length: f64 = vector.iter().map(|value| f64::from(*value).powi(2)).sum();
    let length = squared_length.sqrt();
    if length == 0.0 || (length - 1.0).abs() > 0.002 {
        bail!("embedding vector is not nonzero L2 normalized within 0.002");
    }
    Ok(())
}

fn check_embeddings(response: &Embedded, expected: usize) -> Result<()> {
    check_identity(&response.identity)?;
    if response.embeddings.len() != expected || response.token_counts.len() != expected {
        bail!("embedding response count does not match input count");
    }
    for (vector, tokens) in response.embeddings.iter().zip(&response.token_counts) {
        check_vector(vector)?;
        if *tokens == 0 || *tokens > MAX_TOKENS {
            bail!("embedding input token count is outside 1..32768; truncation is forbidden");
        }
    }
    Ok(())
}

#[derive(Deserialize)]
struct OpenAiEmbedding {
    index: usize,
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct EmbeddingUsage {
    prompt_tokens: usize,
    total_tokens: usize,
}

#[derive(Deserialize)]
struct OpenAiEmbeddings {
    model: String,
    data: Vec<OpenAiEmbedding>,
    usage: EmbeddingUsage,
}

#[derive(Deserialize)]
struct Tokenized {
    tokens: Vec<u32>,
}

fn prepared_input(text: &str, kind: &str) -> Result<String> {
    if text.trim().is_empty() {
        bail!("embedding input must not be blank");
    }
    match kind {
        "query" => Ok(format!("{}{}", QUERY_PREFIX, text)),
        "passage" => Ok(text.to_string()),
        _ => bail!("invalid embedding input kind"),
    }
}

fn check_token_count(tokens: usize, slot_context: usize) -> Result<()> {
    if slot_context == 0 || tokens == 0 || tokens > slot_context.min(MAX_TOKENS) {
        bail!(
            "embedding input exceeds the active token limit or is empty; truncation is forbidden"
        );
    }
    Ok(())
}

fn check_openai_embeddings(
    response: &OpenAiEmbeddings,
    token_counts: &[usize],
    slot_context: usize,
) -> Result<()> {
    if response.model != RUNTIME_MODEL || response.data.len() != token_counts.len() {
        bail!("embedding response identity or count does not match input");
    }
    let mut total = 0_usize;
    let mut indices = HashSet::new();
    for (position, (item, tokens)) in response.data.iter().zip(token_counts).enumerate() {
        // Reject duplicate, missing, out-of-range or reordered indices; never guess.
        if !indices.insert(item.index) || item.index != position {
            bail!("embedding response indices are duplicated or out of order");
        }
        check_token_count(*tokens, slot_context)?;
        check_vector(&item.embedding)?;
        total = total
            .checked_add(*tokens)
            .context("embedding token count overflow")?;
    }
    if token_counts.is_empty()
        || response.usage.prompt_tokens != total
        || response.usage.total_tokens != total
    {
        bail!("embedding usage differs from preflight tokenization; truncation is forbidden");
    }
    Ok(())
}

fn check_runtime(health: &Value, props: &Value, models: &Value) -> Result<usize> {
    if health["status"] != "ok"
        || props["build_info"] != LLAMA_BUILD
        || props["model_alias"] != RUNTIME_MODEL
        || props["model_ftype"] != "F16"
        || props["is_sleeping"] != false
    {
        bail!("llama.cpp health or runtime identity does not match the pinned contract");
    }
    let entries = models["data"]
        .as_array()
        .context("llama.cpp model inventory is invalid")?;
    if entries.len() != 1 {
        bail!("llama.cpp must serve exactly the pinned embedding model");
    }
    let model = &entries[0];
    let context = props["default_generation_settings"]["n_ctx"]
        .as_u64()
        .context("llama.cpp slot context is invalid")?;
    if model["id"] != RUNTIME_MODEL
        || model["meta"]["n_embd"] != DIMENSIONS
        || model["meta"]["ftype"] != "F16"
        || model["meta"]["n_ctx_train"] != MAX_TOKENS
        || model["meta"]["n_ctx"].as_u64() != Some(context)
        || props["total_slots"].as_u64().unwrap_or(0) == 0
        || context == 0
    {
        bail!("llama.cpp model dimensions, format or slot context are incompatible");
    }
    safe(
        usize::try_from(context),
        "llama.cpp slot context is invalid",
    )
}

fn verify_runtime_file(props: &Value) -> Result<()> {
    let path = props["model_path"]
        .as_str()
        .context("llama.cpp model file identity is missing")?;
    let path = Path::new(path);
    // HF cache snapshots are symlinks: hash their resolved regular local blob.
    // Neither the props response nor its private filesystem path is printed.
    // The content hash binds artifact identity, independent of cache/local filename.
    if !path.is_absolute() {
        bail!("llama.cpp model file must use an absolute local path");
    }
    let resolved = safe(
        fs::canonicalize(path),
        "llama.cpp model file cannot be resolved locally",
    )?;
    let metadata = safe(
        fs::symlink_metadata(&resolved),
        "llama.cpp model file is inaccessible",
    )?;
    if !metadata.is_file() {
        bail!("llama.cpp resolved model file must be regular");
    }
    let mut file = safe(
        fs::File::open(resolved),
        "llama.cpp model file cannot be opened",
    )?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 65536];
    loop {
        let size = safe(
            file.read(&mut buffer),
            "llama.cpp model checksum read failed",
        )?;
        if size == 0 {
            break;
        }
        digest.update(&buffer[..size]);
    }
    if format!("{:x}", digest.finalize()) != GGUF_SHA256 {
        bail!("llama.cpp GGUF SHA-256 does not match the pinned artifact");
    }
    Ok(())
}

struct Service {
    client: HttpClient,
    url: Url,
    slot_context: usize,
}

impl Service {
    fn connect() -> Result<Self> {
        let raw = match env::var("HARRIER_URL") {
            Ok(value) => value,
            Err(env::VarError::NotPresent) => "http://127.0.0.1:9999".to_string(),
            Err(_) => bail!("HARRIER_URL is invalid"),
        };
        let url = endpoint(&raw)?;
        let client = safe(
            HttpClient::builder()
                .no_proxy()
                .redirect(Policy::none())
                .timeout(Duration::from_secs(180))
                .build(),
            "embedding HTTP client initialization failed",
        )?;
        let mut service = Self {
            client,
            url,
            slot_context: 0,
        };
        let health = service.get_json("health")?;
        let props = service.get_json("props")?;
        let models = service.get_json("v1/models")?;
        service.slot_context = check_runtime(&health, &props, &models)?;
        verify_runtime_file(&props)?;
        Ok(service)
    }

    fn get_json(&self, route: &str) -> Result<Value> {
        let url = safe(
            self.url.join(route),
            "embedding identity endpoint is invalid",
        )?;
        let response = safe(
            self.client.get(url).send(),
            "embedding identity request failed",
        )?;
        if !response.status().is_success() {
            bail!("embedding identity endpoint returned unsuccessful status");
        }
        safe(
            response.json::<Value>(),
            "embedding identity response is invalid",
        )
    }

    fn post_json<T: serde::de::DeserializeOwned>(&self, route: &str, body: &Value) -> Result<T> {
        let url = safe(self.url.join(route), "embedding endpoint is invalid")?;
        let response = safe(
            self.client.post(url).json(body).send(),
            "embedding service request failed",
        )?;
        if !response.status().is_success() {
            bail!("embedding service returned unsuccessful status; no input was stored");
        }
        safe(
            response.json::<T>(),
            "embedding service response is invalid",
        )
    }

    fn embed(&self, texts: &[&str], kind: &str) -> Result<Embedded> {
        if texts.is_empty() || texts.len() > BATCH_SIZE || !["passage", "query"].contains(&kind) {
            bail!("invalid embedding batch");
        }
        let mut embeddings = Vec::with_capacity(texts.len());
        let mut token_counts = Vec::with_capacity(texts.len());
        // Sequential singleton requests preserve association even if server workers
        // complete out of order. No chat template or Python-side prefix is used.
        for text in texts {
            let prepared = prepared_input(text, kind)?;
            let tokenized: Tokenized = self.post_json(
                "tokenize",
                &json!({
                    "content": prepared, "add_special": true, "parse_special": true
                }),
            )?;
            let count = tokenized.tokens.len();
            check_token_count(count, self.slot_context)?;
            let response: OpenAiEmbeddings = self.post_json(
                "v1/embeddings",
                &json!({
                    "input": [prepared], "model": RUNTIME_MODEL, "encoding_format": "float",
                    "embd_normalize": 2
                }),
            )?;
            check_openai_embeddings(&response, &[count], self.slot_context)?;
            let item = response
                .data
                .into_iter()
                .next()
                .context("embedding response is empty")?;
            embeddings.push(item.embedding);
            token_counts.push(count);
        }
        let embedded = Embedded {
            identity: Identity {
                model: MODEL.into(),
                revision: REVISION.into(),
                profile: PROFILE.into(),
                dimensions: DIMENSIONS,
            },
            embeddings,
            token_counts,
        };
        check_embeddings(&embedded, texts.len())?;
        Ok(embedded)
    }
}

#[derive(Debug, Deserialize)]
struct Artifact {
    path: String,
    bytes: u64,
    sha256: String,
}

#[derive(Debug, Deserialize)]
struct Document {
    id: String,
    source_file: String,
    source_sha256: String,
    source_bytes: u64,
    pages: usize,
    draft_chunks: usize,
    chunk_policy: String,
}

#[derive(Debug, Deserialize)]
struct Fragment {
    block_id: String,
    byte_start: usize,
    byte_end: usize,
}

#[derive(Debug, Deserialize)]
struct Chunk {
    id: String,
    document_id: String,
    order: usize,
    page_start: usize,
    page_end: usize,
    text: String,
    content_sha256: String,
    raw_excerpt: String,
    raw_fragments: Vec<Fragment>,
}

struct Passage {
    chunk: Chunk,
    metadata: Value,
}
struct LoadedDocument {
    document: Document,
    metadata: Value,
}
struct Corpus {
    manifest: Value,
    manifest_hash: String,
    documents: Vec<LoadedDocument>,
    passages: Vec<Passage>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn check_hash(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("artifact or content SHA-256 is invalid");
    }
    Ok(())
}

fn relative_path(path: &str) -> Result<()> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains('\0')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        bail!("manifest artifact path must be a nontraversing relative path");
    }
    Ok(())
}

fn checked_path(path: &Path, directory: bool) -> Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        safe(env::current_dir(), "cannot resolve run directory")?.join(path)
    };
    let mut current = PathBuf::new();
    for component in absolute.components() {
        if matches!(component, Component::ParentDir) {
            bail!("run paths cannot traverse parent directories");
        }
        current.push(component);
        let metadata = safe(
            fs::symlink_metadata(&current),
            "run artifact path is inaccessible",
        )?;
        if metadata.file_type().is_symlink() {
            bail!("run artifact paths must not contain symlinks");
        }
    }
    let metadata = safe(
        fs::symlink_metadata(&absolute),
        "run artifact path is inaccessible",
    )?;
    if (directory && !metadata.is_dir()) || (!directory && !metadata.is_file()) {
        bail!("run artifact has an invalid file type");
    }
    Ok(absolute)
}

fn read_json(bytes: &[u8]) -> Result<Value> {
    safe(
        serde_json::from_slice(bytes),
        "run artifact JSON is invalid",
    )
}

fn verify_artifact(root: &Path, artifact: &Artifact) -> Result<()> {
    relative_path(&artifact.path)?;
    check_hash(&artifact.sha256)?;
    let path = checked_path(&root.join(&artifact.path), false)?;
    let mut file = safe(fs::File::open(path), "cannot open run artifact")?;
    let mut digest = Sha256::new();
    let mut count = 0_u64;
    let mut buffer = [0_u8; 65536];
    loop {
        let size = safe(file.read(&mut buffer), "cannot read run artifact")?;
        if size == 0 {
            break;
        }
        count += size as u64;
        digest.update(&buffer[..size]);
    }
    if count != artifact.bytes || format!("{:x}", digest.finalize()) != artifact.sha256 {
        bail!("run artifact size or SHA-256 mismatch");
    }
    Ok(())
}

fn artifact_bytes(
    root: &Path,
    artifacts: &HashMap<String, Artifact>,
    name: &str,
) -> Result<Vec<u8>> {
    let artifact = artifacts
        .get(name)
        .context("manifest is missing a required artifact")?;
    let path = checked_path(&root.join(name), false)?;
    let bytes = safe(fs::read(path), "cannot read required run artifact")?;
    // Verify again on the bytes actually consumed, not just an earlier read.
    if bytes.len() as u64 != artifact.bytes || sha256(&bytes) != artifact.sha256 {
        bail!("required artifact changed or has invalid checksum");
    }
    Ok(bytes)
}

fn check_document(document: &Document) -> Result<()> {
    check_hash(&document.id)?;
    check_hash(&document.source_sha256)?;
    relative_path(&document.source_file)?;
    if Path::new(&document.source_file).components().count() != 1
        || document.pages == 0
        || document.pages > i32::MAX as usize
        || document.draft_chunks > i32::MAX as usize
    {
        bail!("document source, page count, or chunk count is invalid");
    }
    let expected =
        sha256(format!("{}\n{}", document.source_file, document.source_sha256).as_bytes());
    if document.id != expected
        || !["isolated-layout-v1", "page-block-groups-v1"].contains(&document.chunk_policy.as_str())
    {
        bail!("document identity or chunk policy is invalid");
    }
    Ok(())
}

fn check_chunk(
    chunk: &Chunk,
    document: &Document,
    order: usize,
    previous_page: usize,
    max_chars: usize,
) -> Result<()> {
    check_hash(&chunk.id)?;
    check_hash(&chunk.content_sha256)?;
    if chunk.document_id != document.id
        || chunk.order != order
        || chunk.order > i32::MAX as usize
        || chunk.page_start == 0
        || chunk.page_start < previous_page
        || chunk.page_start != chunk.page_end
        || chunk.page_end > document.pages
    {
        bail!("chunk document identity, order, or physical pages are invalid");
    }
    if chunk.text.trim().is_empty()
        || chunk.text.chars().count() > max_chars
        || sha256(chunk.text.as_bytes()) != chunk.content_sha256
        || chunk.raw_fragments.is_empty()
    {
        bail!("chunk text, content hash, or raw provenance is invalid");
    }
    let mut locations = String::new();
    for fragment in &chunk.raw_fragments {
        check_hash(&fragment.block_id)?;
        if fragment.byte_end <= fragment.byte_start {
            bail!("chunk raw fragment range is invalid");
        }
        locations.push_str(&format!(
            "{}:{}:{};",
            fragment.block_id,
            fragment.byte_start,
            fragment.byte_end - fragment.byte_start
        ));
    }
    let limit = max_chars.to_string();
    let mut parts = vec!["page-paragraph-v1"];
    if document.chunk_policy == "page-block-groups-v1" {
        parts.push("page-block-groups-v1");
    }
    parts.extend([
        document.id.as_str(),
        limit.as_str(),
        locations.as_str(),
        chunk.raw_excerpt.as_str(),
        chunk.text.as_str(),
    ]);
    let encoded = safe(serde_json::to_vec(&parts), "cannot encode chunk identity")?;
    if sha256(&encoded) != chunk.id {
        bail!("chunk identity does not match its original content and policy");
    }
    Ok(())
}

fn corpus(root: &Path) -> Result<Corpus> {
    let root = checked_path(root, true)?;
    let path = checked_path(&root.join("manifest.json"), false)?;
    let bytes = safe(fs::read(path), "cannot read run manifest")?;
    let manifest_hash = sha256(&bytes);
    let manifest = read_json(&bytes)?;
    if manifest["format_version"] != "pdf-corpus-draft-v2"
        || manifest["status"] != "draft_pending_layout_and_tokenizer_review"
        || manifest["chunk_version"] != "page-paragraph-v1"
    {
        bail!("run manifest is not the supported completed draft format");
    }
    let max_chars = manifest["max_chunk_chars"]
        .as_u64()
        .context("manifest has invalid character limit")?;
    if !(200..=20000).contains(&max_chars) {
        bail!("manifest has invalid character limit");
    }
    let artifact_values = manifest["artifacts"]
        .as_array()
        .context("manifest lacks artifact inventory")?;
    let mut artifacts = HashMap::new();
    for value in artifact_values {
        let artifact: Artifact = safe(
            serde_json::from_value(value.clone()),
            "manifest artifact record is invalid",
        )?;
        verify_artifact(&root, &artifact)?;
        if artifacts.insert(artifact.path.clone(), artifact).is_some() {
            bail!("manifest has duplicate artifact paths");
        }
    }
    let document_values = manifest["documents"]
        .as_array()
        .context("manifest lacks document inventory")?;
    let mut documents = Vec::new();
    let mut passages = Vec::new();
    let mut document_ids = HashSet::new();
    let mut chunk_ids = HashSet::new();
    for metadata in document_values {
        let document: Document = safe(
            serde_json::from_value(metadata.clone()),
            "manifest document record is invalid",
        )?;
        check_document(&document)?;
        if !document_ids.insert(document.id.clone()) {
            bail!("manifest has duplicate document IDs");
        }
        let source_path = format!("{}/source.pdf", document.id);
        let source = artifacts
            .get(&source_path)
            .context("manifest lacks source snapshot")?;
        if source.sha256 != document.source_sha256 || source.bytes != document.source_bytes {
            bail!("document source does not match source artifact");
        }
        let document_path = format!("{}/document.json", document.id);
        let saved_document = read_json(&artifact_bytes(&root, &artifacts, &document_path)?)?;
        if saved_document != *metadata {
            bail!("manifest and document metadata disagree");
        }
        let chunk_path = format!("{}/chunks.jsonl", document.id);
        let chunk_bytes = artifact_bytes(&root, &artifacts, &chunk_path)?;
        let text = safe(
            std::str::from_utf8(&chunk_bytes),
            "chunk artifact is not UTF-8",
        )?;
        let mut count = 0;
        let mut previous_page = 0;
        for line in text.lines() {
            if line.trim().is_empty() {
                bail!("chunk artifact contains a blank record");
            }
            let metadata = read_json(line.as_bytes())?;
            let chunk: Chunk = safe(
                serde_json::from_value(metadata.clone()),
                "chunk record is invalid",
            )?;
            check_chunk(&chunk, &document, count, previous_page, max_chars as usize)?;
            if !chunk_ids.insert(chunk.id.clone()) {
                bail!("run has duplicate chunk IDs");
            }
            previous_page = chunk.page_end;
            passages.push(Passage { chunk, metadata });
            count += 1;
        }
        if count != document.draft_chunks {
            bail!("document chunk count does not match manifest");
        }
        documents.push(LoadedDocument {
            document,
            metadata: metadata.clone(),
        });
    }
    if documents.is_empty() || passages.is_empty() || passages.len() > i32::MAX as usize {
        bail!("run has invalid document or passage count");
    }
    print_notice(
        &format!(
            "Verified {} artifacts, {} documents, {} draft passages.",
            artifacts.len(),
            documents.len(),
            passages.len()
        ),
        false,
    );
    Ok(Corpus {
        manifest,
        manifest_hash,
        documents,
        passages,
    })
}

fn row_identity(row: &Row) -> Result<Identity> {
    let dimensions: i32 = safe(
        row.try_get("dimensions"),
        "collection dimensions are invalid",
    )?;
    let dimensions = safe(
        usize::try_from(dimensions),
        "collection dimensions are invalid",
    )?;
    Ok(Identity {
        model: safe(row.try_get("model"), "collection model is invalid")?,
        revision: safe(row.try_get("revision"), "collection revision is invalid")?,
        profile: safe(row.try_get("profile"), "collection profile is invalid")?,
        dimensions,
    })
}

fn already_loaded<C: GenericClient>(client: &mut C, name: &str, corpus: &Corpus) -> Result<bool> {
    let row = safe(client.query_opt("SELECT manifest_sha256,model,revision,profile,dimensions,chunk_count,
        (SELECT count(*) FROM retrieval.passages p WHERE p.collection_id=c.collection_id) AS actual_count,
        (SELECT count(*) FROM retrieval.documents d WHERE d.collection_id=c.collection_id) AS document_count
        FROM retrieval.collections c WHERE collection_id=$1", &[&name]), "database collection inspection failed")?;
    let row = match row {
        Some(value) => value,
        None => return Ok(false),
    };
    check_identity(&row_identity(&row)?)?;
    let manifest_hash: String = safe(
        row.try_get("manifest_sha256"),
        "collection manifest identity is invalid",
    )?;
    let count: i32 = safe(
        row.try_get("chunk_count"),
        "collection passage count is invalid",
    )?;
    let actual: i64 = safe(
        row.try_get("actual_count"),
        "collection actual passage count is invalid",
    )?;
    let document_count: i64 = safe(
        row.try_get("document_count"),
        "collection document count is invalid",
    )?;
    if manifest_hash != corpus.manifest_hash
        || count as usize != corpus.passages.len()
        || actual != corpus.passages.len() as i64
        || document_count != corpus.documents.len() as i64
    {
        bail!("collection conflicts with this run or stored counts; refusing to update");
    }
    Ok(true)
}

fn load(client: &mut Client, root: &Path, name: &str) -> Result<()> {
    let corpus = corpus(root)?;
    if already_loaded(client, name, &corpus)? {
        print_notice(
            &format!(
                "Collection already loaded: {} passages.",
                corpus.passages.len()
            ),
            false,
        );
        return Ok(());
    }
    let service = Service::connect()?;
    let mut vectors = Vec::with_capacity(corpus.passages.len());
    let mut tokens = Vec::with_capacity(corpus.passages.len());
    // Complete every model request before opening the insertion transaction.
    for batch in corpus.passages.chunks(BATCH_SIZE) {
        let texts: Vec<&str> = batch
            .iter()
            .map(|passage| passage.chunk.text.as_str())
            .collect();
        let embedded = service.embed(&texts, "passage")?;
        vectors.extend(embedded.embeddings);
        tokens.extend(embedded.token_counts);
        print_notice(
            &format!(
                "Embedded {}/{} passages.",
                vectors.len(),
                corpus.passages.len()
            ),
            false,
        );
    }
    let mut transaction = safe(client.transaction(), "database transaction start failed")?;
    safe(
        transaction.query_one(
            "SELECT pg_advisory_xact_lock(hashtextextended($1::text, 0))",
            &[&name],
        ),
        "database collection lock failed",
    )?;
    if already_loaded(&mut transaction, name, &corpus)? {
        safe(transaction.commit(), "database transaction commit failed")?;
        print_notice(
            &format!(
                "Collection already loaded by another process: {} passages.",
                corpus.passages.len()
            ),
            false,
        );
        return Ok(());
    }
    let dimensions = DIMENSIONS as i32;
    let count = corpus.passages.len() as i32;
    safe(transaction.execute("INSERT INTO retrieval.collections
        (collection_id,manifest_sha256,model,revision,profile,dimensions,source_manifest,chunk_count)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)", &[&name, &corpus.manifest_hash, &MODEL, &REVISION, &PROFILE, &dimensions, &corpus.manifest, &count]), "database collection insert failed")?;
    for loaded in &corpus.documents {
        let doc = &loaded.document;
        safe(
            transaction.execute(
                "INSERT INTO retrieval.documents
            (collection_id,document_id,source_file,source_sha256,metadata) VALUES ($1,$2,$3,$4,$5)",
                &[
                    &name,
                    &doc.id,
                    &doc.source_file,
                    &doc.source_sha256,
                    &loaded.metadata,
                ],
            ),
            "database document insert failed",
        )?;
    }
    for ((passage, vector), token_count) in corpus.passages.iter().zip(&vectors).zip(&tokens) {
        let chunk = &passage.chunk;
        let vector_text = safe(
            serde_json::to_string(vector),
            "cannot serialize validated vector",
        )?;
        let order = chunk.order as i32;
        let start = chunk.page_start as i32;
        let end = chunk.page_end as i32;
        let token_count = *token_count as i32;
        safe(
            transaction.execute(
                INSERT_PASSAGE,
                &[
                    &name,
                    &chunk.id,
                    &chunk.document_id,
                    &order,
                    &start,
                    &end,
                    &chunk.content_sha256,
                    &chunk.text,
                    &passage.metadata,
                    &token_count,
                    &vector_text,
                ],
            ),
            "database passage insert failed",
        )?;
    }
    safe(transaction.commit(), "database transaction commit failed")?;
    print_notice(
        &format!(
            "Loaded {} documents and {} draft passages transactionally.",
            corpus.documents.len(),
            corpus.passages.len()
        ),
        false,
    );
    Ok(())
}

fn query(client: &mut Client, name: &str, text: &str, top_k: i64) -> Result<()> {
    let row = safe(client.query_opt("SELECT model,revision,profile,dimensions FROM retrieval.collections WHERE collection_id=$1", &[&name]), "database collection inspection failed")?.context("collection not found")?;
    check_identity(&row_identity(&row)?)?;
    let service = Service::connect()?;
    let embedded = service.embed(&[text], "query")?;
    let vector = safe(
        serde_json::to_string(&embedded.embeddings[0]),
        "cannot serialize query vector",
    )?;
    let rows = safe(
        client.query(QUERY_SQL, &[&name, &vector, &top_k]),
        "database retrieval query failed",
    )?;
    let mut results = Vec::with_capacity(rows.len());
    for row in rows {
        let similarity: f64 = safe(row.try_get("similarity"), "retrieval similarity is invalid")?;
        if !similarity.is_finite() {
            bail!("retrieval similarity is nonfinite");
        }
        let chunk_id: String = safe(row.try_get("chunk_id"), "retrieval chunk ID is invalid")?;
        let document_id: String = safe(
            row.try_get("document_id"),
            "retrieval document ID is invalid",
        )?;
        let source_file: String = safe(
            row.try_get("source_file"),
            "retrieval source filename is invalid",
        )?;
        let source_sha256: String = safe(
            row.try_get("source_sha256"),
            "retrieval source checksum is invalid",
        )?;
        let content_sha256: String = safe(
            row.try_get("content_sha256"),
            "retrieval content checksum is invalid",
        )?;
        let page_start: i32 = safe(row.try_get("page_start"), "retrieval start page is invalid")?;
        let page_end: i32 = safe(row.try_get("page_end"), "retrieval end page is invalid")?;
        let passage: String = safe(row.try_get("passage"), "retrieval passage is invalid")?;
        let metadata: Value = safe(row.try_get("metadata"), "retrieval metadata is invalid")?;
        results.push(json!({ "chunk_id": chunk_id, "document_id": document_id, "source_file": source_file,
            "source_sha256": source_sha256, "page_start": page_start, "page_end": page_end,
            "content_sha256": content_sha256, "passage": passage, "metadata": metadata, "similarity": similarity }));
    }
    // stdout deliberately contains full private passages: callers must protect it.
    let output = json!({ "collection": name, "model": MODEL, "revision": REVISION, "profile": PROFILE, "results": results });
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    safe(
        serde_json::to_writer_pretty(&mut writer, &output),
        "query JSON output failed",
    )?;
    safe(writeln!(writer), "query JSON output failed")?;
    Ok(())
}

fn run() -> Result<()> {
    let command = command(env::args().skip(1).collect())?;
    let mut client = database()?;
    match command {
        Command::Init => {
            safe(
                client.batch_execute(SCHEMA_SQL),
                "database schema initialization failed",
            )?;
            print_notice("Retrieval schema initialized.", false);
            Ok(())
        }
        Command::Load { run, collection } => load(&mut client, &run, &collection),
        Command::Query {
            collection,
            text,
            top_k,
        } => query(&mut client, &collection, &text, top_k),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity() -> Identity {
        Identity {
            model: MODEL.to_string(),
            revision: REVISION.to_string(),
            profile: PROFILE.to_string(),
            dimensions: DIMENSIONS,
        }
    }

    fn runtime_values() -> (Value, Value, Value) {
        (
            json!({"status": "ok"}),
            json!({
                "build_info": LLAMA_BUILD, "model_alias": RUNTIME_MODEL,
                "model_ftype": "F16", "is_sleeping": false, "total_slots": 8,
                "default_generation_settings": {"n_ctx": 1024}
            }),
            json!({"data": [{"id": RUNTIME_MODEL, "meta": {
                "n_embd": DIMENSIONS, "ftype": "F16", "n_ctx": 1024,
                "n_ctx_train": MAX_TOKENS
            }}]}),
        )
    }

    fn openai_response() -> OpenAiEmbeddings {
        // Exercise the actual JSON decoding contract, not just constructed structs.
        serde_json::from_value(json!({
            "object": "list", "model": RUNTIME_MODEL,
            "data": [{"object": "embedding", "index": 0, "embedding": vec![1.0 / 32.0; DIMENSIONS]}],
            "usage": {"prompt_tokens": 5, "total_tokens": 5}
        })).unwrap()
    }

    #[test]
    fn query_prompt_is_official_and_passages_are_unchanged() {
        assert_eq!(
            prepared_input("Stop loss?", "query").unwrap(),
            format!("{}Stop loss?", QUERY_PREFIX)
        );
        assert_eq!(
            prepared_input("  Passage.\n", "passage").unwrap(),
            "  Passage.\n"
        );
        assert!(prepared_input(" \n", "query").is_err());
        assert!(prepared_input("x", "chat").is_err());
        assert_ne!(PROFILE, "harrier-hf-last-token-l2-web-search-v1");
        assert!(PROFILE.contains(GGUF_SHA256));
        assert!(PROFILE.contains(LLAMA_BUILD));
        let mut old = identity();
        old.profile = "harrier-hf-last-token-l2-web-search-v1".into();
        assert!(check_identity(&old).is_err());
    }

    #[test]
    fn context_preflight_uses_slot_limit_and_includes_special_tokens() {
        assert!(check_token_count(1024, 1024).is_ok());
        assert!(check_token_count(1025, 1024).is_err());
        assert!(check_token_count(1, 0).is_err());
        assert!(check_token_count(0, 1024).is_err());
        assert!(check_token_count(MAX_TOKENS + 1, MAX_TOKENS + 100).is_err());
        let tokenized: Tokenized =
            serde_json::from_value(json!({"tokens": [32, 13, 151643]})).unwrap();
        assert_eq!(tokenized.tokens.len(), 3); // EOS is not discarded.
        assert!(check_token_count(tokenized.tokens.len(), 2).is_err());
    }

    #[test]
    fn openai_contract_rejects_count_index_identity_usage_and_bad_vectors() {
        let mut response = openai_response();
        assert!(check_openai_embeddings(&response, &[5], 1024).is_ok());
        assert!(check_openai_embeddings(&response, &[5, 5], 1024).is_err());
        assert!(check_openai_embeddings(&response, &[5], 4).is_err());
        response.data[0].index = 1;
        assert!(check_openai_embeddings(&response, &[5], 1024).is_err());
        response.data[0].index = 0;
        response.usage.prompt_tokens = 4;
        assert!(check_openai_embeddings(&response, &[5], 1024).is_err());
        response.usage.prompt_tokens = 5;
        response.usage.total_tokens = 6;
        assert!(check_openai_embeddings(&response, &[5], 1024).is_err());
        response.usage.total_tokens = 5;
        response.model = MODEL.into();
        assert!(check_openai_embeddings(&response, &[5], 1024).is_err());
        response.model = RUNTIME_MODEL.into();
        response.data[0].embedding[0] = f32::NAN;
        assert!(check_openai_embeddings(&response, &[5], 1024).is_err());
        assert!(serde_json::from_value::<OpenAiEmbeddings>(
            json!({"data": [], "model": RUNTIME_MODEL})
        )
        .is_err());
    }

    #[test]
    fn openai_indices_must_be_unique_and_in_input_order() {
        let mut response = openai_response();
        response.data.push(OpenAiEmbedding {
            index: 1,
            embedding: vec![1.0 / 32.0; DIMENSIONS],
        });
        response.usage.prompt_tokens = 10;
        response.usage.total_tokens = 10;
        assert!(check_openai_embeddings(&response, &[5, 5], 1024).is_ok());
        response.data[1].index = 0;
        assert!(check_openai_embeddings(&response, &[5, 5], 1024).is_err());
        response.data[1].index = 1;
        response.data.swap(0, 1);
        assert!(check_openai_embeddings(&response, &[5, 5], 1024).is_err());
    }

    #[test]
    fn runtime_identity_requires_pinned_health_build_model_and_context() {
        let (health, props, models) = runtime_values();
        assert_eq!(check_runtime(&health, &props, &models).unwrap(), 1024);
        assert!(check_runtime(&json!({"status": "loading"}), &props, &models).is_err());
        for (key, value) in [
            ("build_info", json!("other")),
            ("model_alias", json!(MODEL)),
            ("model_ftype", json!("Q4_K_M")),
            ("is_sleeping", json!(true)),
            ("total_slots", json!(0)),
        ] {
            let mut wrong = props.clone();
            wrong[key] = value;
            assert!(check_runtime(&health, &wrong, &models).is_err());
        }
        for (key, value) in [
            ("n_embd", json!(768)),
            ("ftype", json!("Q8_0")),
            ("n_ctx", json!(8192)),
            ("n_ctx_train", json!(8192)),
        ] {
            let mut wrong = models.clone();
            wrong["data"][0]["meta"][key] = value;
            assert!(check_runtime(&health, &props, &wrong).is_err());
        }
        let mut wrong = models.clone();
        wrong["data"][0]["id"] = json!(MODEL);
        assert!(check_runtime(&health, &props, &wrong).is_err());
        assert!(check_runtime(&health, &props, &json!({"data": []})).is_err());
        let mut smaller_props = props.clone();
        let mut smaller_models = models.clone();
        smaller_props["default_generation_settings"]["n_ctx"] = json!(512);
        smaller_models["data"][0]["meta"]["n_ctx"] = json!(512);
        assert_eq!(
            check_runtime(&health, &smaller_props, &smaller_models).unwrap(),
            512
        );
    }

    #[test]
    fn runtime_file_errors_do_not_expose_private_paths() {
        for props in [
            json!({}),
            json!({"model_path": "/private/secret.gguf"}),
            json!({"model_path": "/private/missing/secret.gguf"}),
        ] {
            let error = verify_runtime_file(&props).unwrap_err().to_string();
            assert!(!error.contains("/private"));
            assert!(!error.contains("secret.gguf"));
        }
    }

    #[test]
    #[ignore = "requires the pinned running local llama.cpp runtime and GGUF"]
    fn live_llamacpp_synthetic_query_passage_and_context_rejection() {
        let service = Service::connect().unwrap();
        assert_eq!(service.slot_context, 1024);
        let query = service
            .embed(&["What is a stop-loss order?"], "query")
            .unwrap();
        let passage = service
            .embed(&["A stop-loss order may limit a planned loss."], "passage")
            .unwrap();
        assert_eq!(query.embeddings.len(), 1);
        assert_eq!(passage.embeddings.len(), 1);
        assert!(query.token_counts[0] > passage.token_counts[0]);
        let oversized = "synthetic ".repeat(2048);
        let error = match service.embed(&[&oversized], "passage") {
            Ok(_) => panic!("oversized input was accepted"),
            Err(error) => error.to_string(),
        };
        assert!(error.contains("active token limit"));
    }

    #[test]
    fn terminal_presentation_gating_and_plain_fallback() {
        assert!(style_enabled(true, false, "xterm"));
        assert!(!style_enabled(false, false, "xterm"));
        assert!(!style_enabled(true, true, "xterm"));
        assert!(!style_enabled(true, false, "dumb"));
        assert_eq!(styled("status", false, false), "status");
        assert_eq!(styled("status", false, true), "\x1b[32mstatus\x1b[0m");
        assert_eq!(styled("failure", true, true), "\x1b[1;31mfailure\x1b[0m");
    }

    #[test]
    fn vector_checks_dimensions_finiteness_zero_and_norm() {
        let vector = vec![1.0 / 32.0; DIMENSIONS];
        assert!(check_vector(&vector).is_ok());
        assert!(check_vector(&vector[..1023]).is_err());
        assert!(check_vector(&vec![0.0; DIMENSIONS]).is_err());
        let mut bad = vector.clone();
        bad[0] = f32::NAN;
        assert!(check_vector(&bad).is_err());
        bad[0] = f32::INFINITY;
        assert!(check_vector(&bad).is_err());
        assert!(check_vector(&vec![2.0 / 32.0; DIMENSIONS]).is_err());
    }

    #[test]
    fn response_checks_identity_counts_and_token_limit() {
        let mut response = Embedded {
            identity: identity(),
            embeddings: vec![vec![1.0 / 32.0; DIMENSIONS]],
            token_counts: vec![MAX_TOKENS],
        };
        assert!(check_embeddings(&response, 1).is_ok());
        assert!(check_embeddings(&response, 2).is_err());
        response.token_counts[0] = MAX_TOKENS + 1;
        assert!(check_embeddings(&response, 1).is_err());
        response.token_counts[0] = 0;
        assert!(check_embeddings(&response, 1).is_err());
        response.token_counts[0] = 5;
        response.identity.revision = "other".into();
        assert!(check_embeddings(&response, 1).is_err());
        response.identity = identity();
        response.identity.profile = "other".into();
        assert!(check_embeddings(&response, 1).is_err());
        response.identity = identity();
        response.identity.dimensions = 10;
        assert!(check_embeddings(&response, 1).is_err());
    }

    #[test]
    fn local_endpoints_reject_normalized_remote_credential_and_redirect_targets() {
        for url in [
            "http://127.0.0.1:8091",
            "http://localhost:8091/",
            "http://[::1]:8091",
        ] {
            assert!(endpoint(url).is_ok());
        }
        for url in [
            "http://127.1:8091",
            "http://2130706433",
            "http://127.0.0.2",
            "http://example.org",
            "http://localhost.evil",
            "http://user:secret@localhost",
            "http://localhost/path",
            "http://localhost?token=secret",
            "file://localhost",
            "http://localhost#fragment",
        ] {
            assert!(endpoint(url).is_err(), "{}", url);
        }
    }

    #[test]
    fn database_configuration_is_loopback_only_and_does_not_echo_credentials() {
        for raw in [
            "postgres://user:secret@127.0.0.1/db",
            "host=localhost user=test",
            "host=::1 user=test",
        ] {
            assert!(database_config(raw).is_ok());
        }
        for raw in [
            "postgres://user:secret@remote.invalid/db",
            "host=/tmp user=test password=secret",
            "user=test password=secret",
            "host=localhost hostaddr=192.0.2.1 password=secret",
            "invalid secret",
        ] {
            let error = database_config(raw).unwrap_err().to_string();
            assert!(!error.contains("secret"));
            assert!(!error.contains(raw));
        }
    }

    #[test]
    fn arguments_require_explicit_run_and_reject_duplicate_unknown_or_invalid_options() {
        let args = |values: &[&str]| values.iter().map(|value| value.to_string()).collect();
        assert_eq!(command(args(&["init"])).unwrap(), Command::Init);
        assert_eq!(
            command(args(&[
                "query",
                "--collection",
                "draft",
                "--text",
                "a question"
            ]))
            .unwrap(),
            Command::Query {
                collection: "draft".into(),
                text: "a question".into(),
                top_k: 5
            }
        );
        assert!(command(args(&[
            "load",
            "--collection",
            "draft",
            "--run",
            "private/run"
        ]))
        .is_ok());
        for values in [
            vec!["load", "--collection", "draft"],
            vec!["init", "extra"],
            vec!["query", "--collection", "draft", "--text", " "],
            vec![
                "query",
                "--collection",
                "draft",
                "--text",
                "x",
                "--top-k",
                "0",
            ],
            vec!["load", "--run", "x", "--run", "y", "--collection", "draft"],
            vec![
                "load",
                "--run",
                "x",
                "--collection",
                "draft",
                "--unknown",
                "x",
            ],
        ] {
            assert!(command(args(&values)).is_err());
        }
    }

    #[test]
    fn artifact_paths_hashes_and_sql_contract_are_strict() {
        assert!(relative_path("document/chunks.jsonl").is_ok());
        for path in [
            "",
            "/absolute",
            "../secret",
            "doc/../secret",
            "./file",
            "doc//file",
            "doc\\file",
        ] {
            assert!(relative_path(path).is_err());
        }
        assert!(check_hash(&sha256(b"sample")).is_ok());
        assert!(check_hash("invalid").is_err());
        assert!(INSERT_PASSAGE.contains("CAST($11::text AS halfvec(1024))"));
        assert!(SCHEMA_SQL.contains("retrieval.collections"));
        assert!(SCHEMA_SQL.contains("halfvec(1024)"));
        assert!(QUERY_SQL.contains("$1"));
        assert!(QUERY_SQL.contains("$2"));
        assert!(QUERY_SQL.contains("$3"));
    }

    #[test]
    fn source_and_chunk_identity_bind_content_order_and_pages() {
        let source_hash = sha256(b"source");
        let id = sha256(format!("{}\n{}", "source.pdf", source_hash).as_bytes());
        let doc = Document {
            id: id.clone(),
            source_file: "source.pdf".into(),
            source_sha256: source_hash,
            source_bytes: 6,
            pages: 2,
            draft_chunks: 1,
            chunk_policy: "isolated-layout-v1".into(),
        };
        assert!(check_document(&doc).is_ok());
        let block_id = sha256(b"block");
        let locations = format!("{}:0:4;", block_id);
        let parts = [
            "page-paragraph-v1",
            id.as_str(),
            "3000",
            locations.as_str(),
            "text",
            "text",
        ];
        let mut chunk = Chunk {
            id: sha256(&serde_json::to_vec(&parts).unwrap()),
            document_id: id,
            order: 0,
            page_start: 1,
            page_end: 1,
            text: "text".into(),
            content_sha256: sha256(b"text"),
            raw_excerpt: "text".into(),
            raw_fragments: vec![Fragment {
                block_id,
                byte_start: 0,
                byte_end: 4,
            }],
        };
        assert!(check_chunk(&chunk, &doc, 0, 0, 3000).is_ok());
        assert!(check_chunk(&chunk, &doc, 1, 0, 3000).is_err());
        assert!(check_chunk(&chunk, &doc, 0, 2, 3000).is_err());
        chunk.page_end = 3;
        assert!(check_chunk(&chunk, &doc, 0, 0, 3000).is_err());
        chunk.page_end = 1;
        chunk.text.push('x');
        assert!(check_chunk(&chunk, &doc, 0, 0, 3000).is_err());
    }

    #[test]
    fn artifact_verification_rejects_wrong_sizes_hashes_and_duplicate_path_spellings() {
        let root = env::temp_dir().join(format!("harrier-artifact-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("data"), b"authored sample").unwrap();
        let mut artifact = Artifact {
            path: "data".into(),
            bytes: 15,
            sha256: sha256(b"authored sample"),
        };
        assert!(verify_artifact(&root, &artifact).is_ok());
        artifact.bytes += 1;
        assert!(verify_artifact(&root, &artifact).is_err());
        artifact.bytes -= 1;
        artifact.sha256 = sha256(b"other sample");
        assert!(verify_artifact(&root, &artifact).is_err());
        artifact.path = "../data".into();
        assert!(verify_artifact(&root, &artifact).is_err());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    #[ignore = "requires explicitly selected private run via HARRIER_TEST_RUN"]
    fn real_run_manifest_artifacts_and_all_chunk_identities() {
        let root = env::var("HARRIER_TEST_RUN").expect("set explicit HARRIER_TEST_RUN");
        let corpus = corpus(Path::new(&root)).unwrap();
        assert_eq!(corpus.documents.len(), 3);
        assert_eq!(corpus.passages.len(), 511);
        for passage in &corpus.passages {
            assert_eq!(passage.metadata["text"], passage.chunk.text);
            assert_eq!(passage.metadata["raw_excerpt"], passage.chunk.raw_excerpt);
            assert!(passage.metadata.get("review_status").is_some());
        }
    }

    #[test]
    fn symlink_run_paths_are_rejected() {
        use std::os::unix::fs::symlink;
        let root = env::temp_dir().join(format!("harrier-path-test-{}", std::process::id()));
        fs::create_dir(&root).unwrap();
        fs::write(root.join("regular"), "data").unwrap();
        symlink(root.join("regular"), root.join("link")).unwrap();
        assert!(checked_path(&root.join("regular"), false).is_ok());
        assert!(checked_path(&root.join("link"), false).is_err());
        assert!(checked_path(&root, false).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
