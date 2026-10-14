mod chunk;
mod cleanup;

use std::{
    collections::HashMap,
    env, fs,
    io::{self, IsTerminal, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{bail, Context, Result};
use serde::Serialize;
use sha2::{Digest, Sha256};

use chunk::{build_blocks, build_chunks, build_chunks_with_layout_policy, Page, CHUNK_VERSION};

use cleanup::{clean_pages, CLEANUP_VERSION};

const INITIAL_PDFS: [&str; 3] = [
    "DayTrading.com-PDF-Day-Trading-For-Beginners.pdf",
    "The-Complete-Guide-To-Day-Trading.pdf",
    "The-Complete-Guide-to-Trading.pdf",
];
const EXTRACTION_VERSION: &str = "poppler-layout-pages-v1";
const MAX_PDF_BYTES: u64 = 256 * 1024 * 1024;

pub fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug)]
struct Options {
    books: PathBuf,
    output: PathBuf,
    pdfs: Vec<String>,
    max_chars: usize,
}

#[derive(Serialize)]
struct Document {
    id: String,
    source_file: String,
    source_sha256: String,
    source_bytes: usize,
    duplicate_content_of: Option<String>,
    title: Option<String>,
    author: Option<String>,
    publication: Option<String>,
    pages: usize,
    blocks: usize,
    draft_chunks: usize,
    excluded_lines: usize,
    chunk_policy: &'static str,
    warnings: Vec<String>,
}

#[derive(Serialize)]
struct Artifact {
    path: String,
    bytes: usize,
    sha256: String,
}

#[derive(Serialize)]
struct Manifest {
    format_version: &'static str,
    status: &'static str,
    extraction_version: &'static str,
    chunk_version: &'static str,
    cleanup_version: &'static str,
    pdftotext_version: String,
    pdfinfo_version: String,
    max_chunk_chars: usize,
    tokenizer_validated: bool,
    layout_reviewed: bool,
    created_unix_seconds: u64,
    documents: Vec<Document>,
    artifacts: Vec<Artifact>,
}

fn main() {
    if let Err(error) = run() {
        let message = format!("Error: {:#}", error);
        eprintln!("{}", styled(&message, "1;31", true));
        std::process::exit(1);
    }
}

fn styled(text: &str, code: &str, stderr: bool) -> String {
    let terminal = if stderr {
        io::stderr().is_terminal()
    } else {
        io::stdout().is_terminal()
    };
    let enabled = style_enabled(
        terminal,
        env::var_os("NO_COLOR").is_some(),
        &env::var("TERM").unwrap_or_default(),
    );
    if enabled {
        format!("\x1b[{}m{}\x1b[0m", code, text)
    } else {
        text.to_string()
    }
}

fn style_enabled(terminal: bool, no_color: bool, term: &str) -> bool {
    terminal && !no_color && term != "dumb"
}

fn options(args: Vec<String>) -> Result<Options> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("missing repository root")?;
    let mut options = Options {
        books: root.join("day_trading/books"),
        output: root.join("day_trading/embeddings"),
        pdfs: Vec::new(),
        max_chars: 3000,
    };
    let mut values = args.into_iter();
    while let Some(flag) = values.next() {
        let value = values.next().context("each option requires a value")?;
        match flag.as_str() {
            "--books" => options.books = PathBuf::from(value),
            "--output" => options.output = PathBuf::from(value),
            "--pdf" => {
                validate_filename(&value)?;
                options.pdfs.push(value);
            }
            "--max-chars" => options.max_chars = value.parse().context("invalid --max-chars")?,
            _ => bail!("unknown option: {}", flag),
        }
    }
    if !(200..=20_000).contains(&options.max_chars) {
        bail!("--max-chars must be 200..20000; this is NOT a token limit");
    }
    if options.pdfs.is_empty() {
        options.pdfs = INITIAL_PDFS.iter().map(|name| name.to_string()).collect();
    }
    let mut unique = std::collections::HashSet::new();
    for name in &options.pdfs {
        if !unique.insert(name) {
            bail!("duplicate --pdf: {}", name);
        }
    }
    Ok(options)
}

fn validate_filename(name: &str) -> Result<()> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name == "."
        || name == ".."
        || !name.to_ascii_lowercase().ends_with(".pdf")
    {
        bail!("--pdf requires a PDF basename, not a path");
    }
    Ok(())
}

// Refuse symlinks in the artifact directory chain; newly created directories are private.
fn private_directory(path: &Path) -> Result<()> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir()?.join(path)
    };
    let mut current = PathBuf::new();
    for component in absolute.components() {
        if matches!(component, std::path::Component::ParentDir) {
            bail!("artifact paths cannot contain '..'");
        }
        current.push(component);
        match fs::symlink_metadata(&current) {
            Ok(metadata) => {
                if metadata.file_type().is_symlink() || !metadata.is_dir() {
                    bail!(
                        "artifact path is not a real directory: {}",
                        current.display()
                    );
                }
            }
            Err(error) => {
                if error.kind() != io::ErrorKind::NotFound {
                    return Err(error.into());
                }
                fs::DirBuilder::new().mode(0o700).create(&current)?;
            }
        }
    }
    Ok(())
}

fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn save_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    save(path, &bytes)
}

fn save_jsonl<T: Serialize>(path: &Path, values: &[T]) -> Result<()> {
    let mut bytes = Vec::new();
    for value in values {
        serde_json::to_writer(&mut bytes, value)?;
        bytes.push(b'\n');
    }
    save(path, &bytes)
}

fn tool_version(tool: &str) -> Result<String> {
    let result = Command::new("timeout")
        .args(["10s", tool, "-v"])
        .output()
        .with_context(|| {
            format!(
                "cannot launch {}; install poppler-utils and coreutils",
                tool
            )
        })?;
    if !result.status.success() {
        bail!("{} version check failed", tool);
    }
    Ok(format!(
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    ))
}

fn run() -> Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.iter().any(|arg| arg == "--help") {
        println!("PDF ingestion/chunking only. No embedding calls.\nUsage: cargo run --manifest-path embedding-code/Cargo.toml -- [--books DIR] [--output DIR] [--pdf BASENAME ...] [--max-chars 3000]\nDefaults: three PLAN PDFs; private day_trading/embeddings; draft chunks requiring layout/tokenizer review.");
        return Ok(());
    }
    let options = options(args)?;
    let text_version = tool_version("pdftotext")?;
    let info_version = tool_version("pdfinfo")?;
    private_directory(&options.output)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let run_dir = options.output.join(format!(
        "pdf-draft-{}-{}",
        now.as_secs(),
        now.subsec_nanos()
    ));
    fs::DirBuilder::new().mode(0o700).create(&run_dir)?;
    println!(
        "{}",
        styled(
            "Local PDF inventory, extraction and draft chunking",
            "1;36",
            false
        )
    );
    let result = prepare(
        &options,
        &run_dir,
        text_version,
        info_version,
        now.as_secs(),
    );
    if let Err(error) = &result {
        let failure = format!("Incomplete run; do not consume artifacts.\n{:#}\n", error);
        save(&run_dir.join("FAILED.txt"), failure.as_bytes())?;
        eprintln!(
            "{}",
            styled(
                &format!("Partial artifacts retained at {}", run_dir.display()),
                "33",
                true
            )
        );
    }
    result?;
    println!(
        "{}",
        styled(
            &format!("Draft artifacts saved: {}", run_dir.display()),
            "32",
            false
        )
    );
    println!("{}", styled("STOP: layout review and candidate-tokenizer safeguards still required. No embeddings or database writes performed.", "33", false));
    Ok(())
}

fn prepare(
    options: &Options,
    run_dir: &Path,
    text_version: String,
    info_version: String,
    created: u64,
) -> Result<()> {
    let mut manifest = Manifest {
        format_version: "pdf-corpus-draft-v2",
        status: "draft_pending_layout_and_tokenizer_review",
        extraction_version: EXTRACTION_VERSION,
        chunk_version: CHUNK_VERSION,
        cleanup_version: CLEANUP_VERSION,
        pdftotext_version: text_version,
        pdfinfo_version: info_version,
        max_chunk_chars: options.max_chars,
        tokenizer_validated: false,
        layout_reviewed: false,
        created_unix_seconds: created,
        documents: Vec::new(),
        artifacts: Vec::new(),
    };
    let mut hashes = HashMap::new();
    for name in &options.pdfs {
        let source = options.books.join(name);
        let metadata = fs::symlink_metadata(&source)
            .with_context(|| format!("missing PDF: {}", source.display()))?;
        if !metadata.is_file()
            || metadata.file_type().is_symlink()
            || metadata.len() > MAX_PDF_BYTES
        {
            bail!("PDF must be a regular nonsymlink file <=256 MiB: {}", name);
        }
        let bytes = fs::read(&source)?;
        if bytes.len() as u64 > MAX_PDF_BYTES || !bytes.starts_with(b"%PDF-") {
            bail!("invalid/oversized PDF: {}", name);
        }
        let hash = sha256(&bytes);
        let id = sha256(format!("{}\n{}", name, hash).as_bytes());
        let duplicate_content_of = hashes.get(&hash).cloned();
        hashes.entry(hash.clone()).or_insert_with(|| id.clone());
        let directory = run_dir.join(&id);
        fs::DirBuilder::new().mode(0o700).create(&directory)?;
        // Extraction reads a hashed snapshot, not a source that can change during the run.
        let snapshot = directory.join("source.pdf");
        save(&snapshot, &bytes)?;
        let info = Command::new("timeout")
            .args(["120s", "pdfinfo"])
            .arg(&snapshot)
            .env("LC_ALL", "C")
            .stdin(Stdio::null())
            .output()?;
        save(&directory.join("pdfinfo.txt"), &info.stdout)?;
        save(&directory.join("pdfinfo-stderr.txt"), &info.stderr)?;
        if !info.status.success() {
            bail!("pdfinfo failed for {}; see private stderr", name);
        }
        let page_count = parse_page_count(&String::from_utf8(info.stdout)?)?;
        let text_file = directory.join("extracted.txt");
        save(&text_file, b"")?;
        let result = Command::new("timeout")
            .args([
                "120s",
                "pdftotext",
                "-layout",
                "-enc",
                "UTF-8",
                "-eol",
                "unix",
            ])
            .arg(&snapshot)
            .arg(&text_file)
            .stdin(Stdio::null())
            .output()?;
        // pdftotext creates this file itself; restrict its permissions immediately.
        if text_file.exists() {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&text_file, fs::Permissions::from_mode(0o600))?;
        }
        save(&directory.join("pdftotext-stderr.txt"), &result.stderr)?;
        if !result.status.success() {
            bail!("pdftotext failed for {}; see private stderr", name);
        }
        let extracted = fs::read_to_string(&text_file)?;
        let pages = parse_pages(&extracted, page_count)?;
        let (cleaned_pages, excluded_lines) = clean_pages(&hash, &pages);
        let blocks = build_blocks(&id, &cleaned_pages);
        let group_layout = hash
            == "74296415db1e5d94b64352751df2a7b4d559b227fc0916f31d545e2ad35efeb0"
            || hash == "50f6e16003682e96f7bd5fc05d5947fcbcc3efba5e8c37afdca908f802251049";
        let chunk_policy = if group_layout {
            "page-block-groups-v1"
        } else {
            "isolated-layout-v1"
        };
        let (chunks, mut warnings) = if group_layout {
            build_chunks_with_layout_policy(&id, &blocks, options.max_chars, true)
        } else {
            build_chunks(&id, &blocks, options.max_chars)
        };
        warnings.push("All pages require reading-order/visual review; only checksum-bound verified margin rules applied. No OCR or chart interpretation performed.".to_string());
        if !result.stderr.is_empty() {
            warnings.push(
                "Poppler emitted extraction diagnostics; inspect pdftotext-stderr.txt.".to_string(),
            );
        }
        for page in &pages {
            for warning in &page.warnings {
                warnings.push(format!("Page {}: {}", page.number, warning));
            }
        }
        save_jsonl(&directory.join("pages.jsonl"), &pages)?;
        save_jsonl(&directory.join("cleaned-pages.jsonl"), &cleaned_pages)?;
        save_jsonl(&directory.join("excluded-lines.jsonl"), &excluded_lines)?;
        save_jsonl(&directory.join("blocks.jsonl"), &blocks)?;
        save_jsonl(&directory.join("chunks.jsonl"), &chunks)?;
        let document = Document {
            id: id.clone(),
            source_file: name.clone(),
            source_sha256: hash,
            source_bytes: bytes.len(),
            duplicate_content_of,
            title: None,
            author: None,
            publication: None,
            pages: pages.len(),
            blocks: blocks.len(),
            draft_chunks: chunks.len(),
            excluded_lines: excluded_lines.len(),
            chunk_policy,
            warnings,
        };
        save_json(&directory.join("document.json"), &document)?;
        for entry in fs::read_dir(&directory)? {
            let path = entry?.path();
            let data = fs::read(&path)?;
            manifest.artifacts.push(Artifact {
                path: path.strip_prefix(run_dir)?.to_string_lossy().into_owned(),
                bytes: data.len(),
                sha256: sha256(&data),
            });
        }
        println!(
            "{}: {} pages, {} draft chunks",
            name, document.pages, document.draft_chunks
        );
        manifest.documents.push(document);
    }
    manifest
        .artifacts
        .sort_by(|left, right| left.path.cmp(&right.path));
    // A manifest marks completed draft preparation; failed/partial runs have no manifest.
    save_json(&run_dir.join("manifest.json"), &manifest)?;
    fs::File::open(run_dir)?.sync_all()?;
    Ok(())
}

fn parse_page_count(info: &str) -> Result<usize> {
    for line in info.lines() {
        if let Some(value) = line.strip_prefix("Pages:") {
            let count: usize = value.trim().parse().context("invalid pdfinfo page count")?;
            if count == 0 {
                bail!("zero-page PDF");
            }
            return Ok(count);
        }
    }
    bail!("pdfinfo did not report physical page count")
}

fn parse_pages(text: &str, expected: usize) -> Result<Vec<Page>> {
    let mut parts: Vec<&str> = text.split('\u{c}').collect();
    if parts.last() == Some(&"") {
        parts.pop();
    }
    if parts.len() != expected {
        bail!(
            "page break count {} differs from pdfinfo {}; refuse guessed page provenance",
            parts.len(),
            expected
        );
    }
    let mut pages = Vec::new();
    for (index, part) in parts.into_iter().enumerate() {
        let mut warnings = Vec::new();
        if part.split_whitespace().count() < 20 {
            warnings.push(
                "low text: inspect for front matter, chart, blank or extraction failure"
                    .to_string(),
            );
        }
        if part.contains('\u{fffd}') {
            warnings.push("Unicode replacement character: possible encoding loss".to_string());
        }
        pages.push(Page {
            number: index + 1,
            raw_text: part.to_string(),
            warnings,
        });
    }
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminal_style_gating() {
        assert!(style_enabled(true, false, "xterm"));
        assert!(!style_enabled(false, false, "xterm"));
        assert!(!style_enabled(true, true, "xterm"));
        assert!(!style_enabled(true, false, "dumb"));
    }

    #[test]
    fn page_provenance_keeps_empty_physical_pages() {
        let pages = parse_pages("First\u{c}\u{c}Third\u{c}", 3).unwrap();
        assert_eq!(pages[1].number, 2);
        assert_eq!(pages[1].raw_text, "");
        assert_eq!(pages[2].raw_text, "Third");
        assert!(parse_pages("One\u{c}", 2).is_err());
    }

    #[test]
    fn input_validation() {
        assert!(validate_filename("../book.pdf").is_err());
        assert!(validate_filename("book.epub").is_err());
        assert!(options(vec!["--max-chars".into(), "0".into()]).is_err());
        assert!(options(vec![
            "--pdf".into(),
            "a.pdf".into(),
            "--pdf".into(),
            "a.pdf".into()
        ])
        .is_err());
        assert!(parse_page_count("Pages: 0").is_err());
        assert_eq!(parse_page_count("Pages: 18\n").unwrap(), 18);
    }

    #[test]
    fn ids_preserve_case_sensitive_source_identity() {
        let first = sha256(b"The-Complete-Guide-To-Day-Trading.pdf\nhash");
        let second = sha256(b"The-Complete-Guide-to-Trading.pdf\nhash");
        assert_ne!(first, second);
        assert_eq!(
            sha256(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
