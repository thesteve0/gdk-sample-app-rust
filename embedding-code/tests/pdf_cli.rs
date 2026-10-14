use std::{
    fs,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

// Course-authored one-page PDF, generated without copyrighted book content.
fn synthetic_pdf() -> Vec<u8> {
    let stream = "BT /F1 12 Tf 72 720 Td (Synthetic risk lesson.) Tj 0 -20 Td (A plan is not a guarantee.) Tj ET\n";
    let objects = [
        "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
        "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
        "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_string(),
        "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        format!("<< /Length {} >>\nstream\n{}endstream", stream.len(), stream),
    ];
    let mut pdf = "%PDF-1.4\n".to_string();
    let mut offsets = Vec::new();
    for (index, object) in objects.iter().enumerate() {
        offsets.push(pdf.len());
        pdf.push_str(&format!("{} 0 obj\n{}\nendobj\n", index + 1, object));
    }
    let xref = pdf.len();
    pdf.push_str("xref\n0 6\n0000000000 65535 f \n");
    for offset in offsets {
        pdf.push_str(&format!("{:010} 00000 n \n", offset));
    }
    pdf.push_str(&format!(
        "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{}\n%%EOF\n",
        xref
    ));
    pdf.into_bytes()
}

fn invoke(books: &Path, output: &Path, name: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_pdf-corpus-preparation"))
        .args([
            "--books",
            books.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
            "--pdf",
            name,
        ])
        .env("NO_COLOR", "")
        .output()
        .unwrap()
}

#[test]
fn real_poppler_synthetic_pipeline_and_failure_marker() {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("pdf-prep-test-{}-{}", std::process::id(), stamp));
    let books = root.join("books");
    let output = root.join("outputs");
    fs::create_dir_all(&books).unwrap();
    fs::write(books.join("synthetic.pdf"), synthetic_pdf()).unwrap();
    let result = invoke(&books, &output, "synthetic.pdf");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    assert!(!result.stdout.contains(&27));
    let run = fs::read_dir(&output)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(run.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest["documents"][0]["pages"], 1);
    assert_eq!(manifest["tokenizer_validated"], false);
    assert_eq!(manifest["layout_reviewed"], false);
    let doc_id = manifest["documents"][0]["id"].as_str().unwrap();
    let chunks = fs::read_to_string(run.join(doc_id).join("chunks.jsonl")).unwrap();
    assert!(chunks.contains("Synthetic risk lesson."));
    for artifact in manifest["artifacts"].as_array().unwrap() {
        let path = run.join(artifact["path"].as_str().unwrap());
        let bytes = fs::read(&path).unwrap();
        assert_eq!(bytes.len() as u64, artifact["bytes"].as_u64().unwrap());
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    let failure_output = root.join("failure");
    let failure = invoke(&books, &failure_output, "missing.pdf");
    assert!(!failure.status.success());
    assert!(!failure.stderr.contains(&27));
    let failed_run = fs::read_dir(&failure_output)
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    assert!(failed_run.join("FAILED.txt").is_file());
    assert!(!failed_run.join("manifest.json").exists());
    fs::remove_dir_all(root).unwrap();
}
