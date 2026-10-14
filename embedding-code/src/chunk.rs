//! Page-local draft chunks. Character limits are not model-token limits.
//! Raw excerpts concatenate exact source fragments (not invented blank separators).
use serde::{Deserialize, Serialize};

pub const CHUNK_VERSION: &str = "page-paragraph-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    pub number: usize,
    pub raw_text: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub id: String,
    pub page: usize,
    pub start_line: usize,
    pub end_line: usize,
    pub raw_text: String,
    pub text: String,
    pub kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawFragment {
    pub block_id: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub document_id: String,
    pub order: usize,
    pub page_start: usize,
    pub page_end: usize,
    pub block_ids: Vec<String>,
    pub overlap_block_ids: Vec<String>,
    pub raw_excerpt: String,
    pub raw_fragments: Vec<RawFragment>,
    pub text: String,
    pub content_sha256: String,
    pub review_status: String,
    pub section: Option<String>,
}

// Length-prefixed JSON avoids ambiguous concatenations in identity inputs.
fn identity(parts: &[&str]) -> String {
    let encoded = serde_json::to_vec(parts).expect("string arrays serialize");
    crate::sha256(&encoded)
}

fn is_layout(raw: &str) -> bool {
    for line in raw.lines() {
        let text = line.trim_start();
        let numbered = text.split_once(['.', ')']);
        if let Some((prefix, suffix)) = numbered {
            if !prefix.is_empty()
                && prefix.chars().all(|ch| ch.is_ascii_digit())
                && suffix.starts_with(char::is_whitespace)
            {
                return true;
            }
        }
        if line.starts_with(char::is_whitespace)
            || text.starts_with(['-', '*', '•', '‣'])
            || text.contains("  ")
            || text.contains('\t')
            || text.contains('|')
            || text.starts_with("```")
        {
            return true;
        }
    }
    false
}

fn normalize(raw: &str) -> String {
    let mut text = String::new();
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !text.is_empty() {
            // Keep ambiguous hyphenated wraps visible; never dehyphenate them.
            if text.ends_with(['-', '\u{00ad}', '\u{2010}', '\u{2011}']) {
                text.push('\n');
            } else {
                text.push(' ');
            }
        }
        text.push_str(trimmed);
    }
    text
}

pub fn build_blocks(document_id: &str, pages: &[Page]) -> Vec<Block> {
    let mut blocks = Vec::new();
    for page in pages {
        let lines: Vec<&str> = page.raw_text.split_inclusive('\n').collect();
        let mut index = 0;
        while index < lines.len() {
            if lines[index].trim().is_empty() {
                index += 1;
                continue;
            }
            let start = index;
            while index < lines.len() && !lines[index].trim().is_empty() {
                index += 1;
            }
            let raw = lines[start..index].concat();
            let layout = is_layout(&raw);
            let kind = if layout { "layout" } else { "paragraph" };
            let text = if layout { raw.clone() } else { normalize(&raw) };
            let location = format!("{}:{}:{}", page.number, start + 1, index);
            let id = identity(&[CHUNK_VERSION, document_id, &location, kind, &raw]);
            blocks.push(Block {
                id,
                page: page.number,
                start_line: start + 1,
                end_line: index,
                raw_text: raw,
                text,
                kind: kind.to_string(),
            });
        }
    }
    blocks
}

#[derive(Clone)]
struct Fragment<'a> {
    block: &'a Block,
    raw: String,
    text: String,
    whole: bool,
    offset: usize,
}

// Only obvious punctuation followed by whitespace and an uppercase start is a
// candidate. Ambiguous periods/abbreviations use source line boundaries instead.
fn split_paragraph(block: &Block, max_chars: usize) -> Option<Vec<Fragment<'_>>> {
    let raw = &block.raw_text;
    let mut cuts = vec![0];
    for (offset, ch) in raw.char_indices() {
        if ch == '\n' {
            cuts.push(offset + 1);
        }
        if !['.', '?', '!'].contains(&ch) {
            continue;
        }
        let prefix = &raw[..offset];
        // Keep the optional word extraction visible for course readers.
        #[allow(clippy::manual_unwrap_or_default, clippy::manual_unwrap_or)]
        let word = match prefix.split_whitespace().last() {
            Some(value) => value,
            None => "",
        };
        let ambiguous = ch == '.'
            && (word.chars().count() < 3
                || !word.chars().all(char::is_lowercase)
                || ["dr", "mr", "mrs", "ms", "prof", "etc", "vs", "e.g", "i.e"]
                    .contains(&word.to_lowercase().as_str()));
        if ambiguous {
            continue;
        }
        let after = offset + ch.len_utf8();
        let tail = &raw[after..];
        let next = tail.trim_start();
        if next.len() < tail.len() && next.starts_with(char::is_uppercase) {
            cuts.push(raw.len() - next.len());
        }
    }
    cuts.push(raw.len());
    cuts.sort_unstable();
    cuts.dedup();
    let mut fragments = Vec::new();
    let mut start = 0;
    while start < raw.len() {
        let mut end = start;
        for &candidate in &cuts {
            if candidate > start {
                let candidate_text = normalize(&raw[start..candidate]);
                if candidate_text.chars().count() > max_chars {
                    break;
                }
                end = candidate;
            }
        }
        if end == start {
            return None;
        }
        let fragment_raw = raw[start..end].to_string();
        fragments.push(Fragment {
            block,
            text: normalize(&fragment_raw),
            raw: fragment_raw,
            whole: false,
            offset: start,
        });
        start = end;
    }
    Some(fragments)
}

fn emit(
    document_id: &str,
    max_chars: usize,
    parts: &[Fragment<'_>],
    overlap: bool,
    group_layout: bool,
    chunks: &mut Vec<Chunk>,
) {
    let first = &parts[0];
    let mut ids = Vec::new();
    let mut locations = String::new();
    let mut raw = String::new();
    let mut texts = Vec::new();
    let mut raw_fragments = Vec::new();
    for part in parts {
        raw_fragments.push(RawFragment {
            block_id: part.block.id.clone(),
            byte_start: part.offset,
            byte_end: part.offset + part.raw.len(),
        });
        if !ids.contains(&part.block.id) {
            ids.push(part.block.id.clone());
        }
        locations.push_str(&format!(
            "{}:{}:{};",
            part.block.id,
            part.offset,
            part.raw.len()
        ));
        raw.push_str(&part.raw);
        texts.push(part.text.as_str());
    }
    let text = texts.join("\n\n");
    let limit = max_chars.to_string();
    let id = if group_layout {
        identity(&[
            CHUNK_VERSION,
            "page-block-groups-v1",
            document_id,
            &limit,
            &locations,
            &raw,
            &text,
        ])
    } else {
        identity(&[CHUNK_VERSION, document_id, &limit, &locations, &raw, &text])
    };
    let overlap_ids = if overlap {
        vec![first.block.id.clone()]
    } else {
        Vec::new()
    };
    chunks.push(Chunk {
        id,
        document_id: document_id.to_string(),
        order: chunks.len(),
        page_start: first.block.page,
        page_end: first.block.page,
        block_ids: ids,
        overlap_block_ids: overlap_ids,
        raw_excerpt: raw,
        raw_fragments,
        content_sha256: crate::sha256(text.as_bytes()),
        text,
        review_status: "draft_pending_layout_and_tokenizer_review".to_string(),
        section: None,
    });
}

pub fn build_chunks(
    document_id: &str,
    blocks: &[Block],
    max_chars: usize,
) -> (Vec<Chunk>, Vec<String>) {
    build_chunks_with_layout_policy(document_id, blocks, max_chars, false)
}

pub fn build_chunks_with_layout_policy(
    document_id: &str,
    blocks: &[Block],
    max_chars: usize,
    group_layout: bool,
) -> (Vec<Chunk>, Vec<String>) {
    let mut chunks = Vec::new();
    let mut warnings = Vec::new();
    let mut pending: Vec<Fragment<'_>> = Vec::new();
    let mut overlap = false;
    for block in blocks {
        let count = block.text.chars().count();
        let fragments = if count <= max_chars && max_chars > 0 {
            Some(vec![Fragment {
                block,
                raw: block.raw_text.clone(),
                text: block.text.clone(),
                whole: true,
                offset: 0,
            }])
        } else if block.kind == "paragraph" && max_chars > 0 {
            split_paragraph(block, max_chars)
        } else {
            None
        };
        let fragments = match fragments {
            Some(value) => value,
            None => {
                if !pending.is_empty() {
                    emit(
                        document_id,
                        max_chars,
                        &pending,
                        overlap,
                        group_layout,
                        &mut chunks,
                    );
                    pending.clear();
                }
                overlap = false;
                warnings.push(format!("Quarantined block {} (page {}, lines {}-{}): {} characters cannot fit limit {} without unsafe splitting; source block retained.",
                    block.id, block.page, block.start_line, block.end_line, count, max_chars));
                continue;
            }
        };
        if fragments.len() > 1 {
            warnings.push(format!("Split block {} at source sentence/line boundaries; layout and tokenizer review required.", block.id));
        }
        for fragment in fragments {
            let size = pending
                .iter()
                .map(|part| part.text.chars().count())
                .sum::<usize>()
                + pending.len().saturating_sub(1) * 2;
            let boundary = pending.last().is_some_and(|last| {
                last.block.page != block.page
                    || (!group_layout && (last.block.kind == "layout" || block.kind == "layout"))
                    || !last.whole
                    || !fragment.whole
            });
            let overflow =
                !pending.is_empty() && size + 2 + fragment.text.chars().count() > max_chars;
            if !pending.is_empty() && (boundary || overflow) {
                emit(
                    document_id,
                    max_chars,
                    &pending,
                    overlap,
                    group_layout,
                    &mut chunks,
                );
                let previous = pending.last().cloned().expect("pending is nonempty");
                pending.clear();
                overlap = false;
                // A whole previous block may repeat only alongside new progress.
                if !boundary
                    && previous.block.kind == "paragraph"
                    && fragment.block.kind == "paragraph"
                    && previous.whole
                    && fragment.whole
                    && previous.text.chars().count() + 2 + fragment.text.chars().count()
                        <= max_chars
                {
                    pending.push(previous);
                    overlap = true;
                }
            }
            pending.push(fragment);
        }
    }
    if !pending.is_empty() {
        emit(
            document_id,
            max_chars,
            &pending,
            overlap,
            group_layout,
            &mut chunks,
        );
    }
    (chunks, warnings)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(number: usize, text: &str) -> Page {
        Page {
            number,
            raw_text: text.to_string(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn grouped_layout_preserves_format_provenance_bounds_and_page_boundaries() {
        let blocks = build_blocks(
            "doc",
            &[
                page(
                    1,
                    "Intro\n\n    Indented quotation.\n\n- First item\n- Second item\n\nConclusion",
                ),
                page(2, "Next page"),
            ],
        );
        let (isolated, _) = build_chunks("doc", &blocks, 200);
        let (grouped, warnings) = build_chunks_with_layout_policy("doc", &blocks, 200, true);
        assert!(warnings.is_empty());
        assert!(grouped.len() < isolated.len());
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].block_ids.len(), 4);
        assert!(grouped[0].text.contains("    Indented quotation.\n"));
        assert!(grouped[0].text.contains("- First item\n- Second item\n"));
        assert_eq!(grouped[1].page_start, 2);
        let (bounded, _) = build_chunks_with_layout_policy("doc", &blocks, 45, true);
        assert!(bounded.iter().all(|c| c.text.chars().count() <= 45));
        assert!(bounded
            .iter()
            .all(|c| c.overlap_block_ids.iter().all(|id| blocks
                .iter()
                .find(|b| &b.id == id)
                .unwrap()
                .kind
                == "paragraph")));
        let single = build_blocks("doc", &[page(1, "Intro")]);
        assert_ne!(
            build_chunks("doc", &single, 200).0[0].id,
            build_chunks_with_layout_policy("doc", &single, 200, true).0[0].id
        );
    }

    #[test]
    fn preserves_pages_lines_raw_headers_and_hyphens() {
        let pages = vec![
            page(
                3,
                "HEADER\r\n\r\nA wrapped\r\nparagraph.\r\n\r\ninter-\r\nnational\r\n",
            ),
            page(4, "HEADER\n"),
        ];
        let blocks = build_blocks("doc", &pages);
        assert_eq!(blocks.len(), 4);
        assert_eq!((blocks[1].start_line, blocks[1].end_line), (3, 4));
        assert_eq!(blocks[1].raw_text, "A wrapped\r\nparagraph.\r\n");
        assert_eq!(blocks[1].text, "A wrapped paragraph.");
        assert_eq!(blocks[2].text, "inter-\nnational");
        assert_eq!(blocks[3].text, "HEADER");
        let (chunks, warnings) = build_chunks("doc", &blocks, 100);
        assert!(warnings.is_empty());
        assert_eq!(chunks.len(), 2);
        assert_eq!((chunks[0].page_start, chunks[0].page_end), (3, 3));
        assert_eq!(chunks[1].page_start, 4);
        assert_eq!(
            chunks[0].block_ids,
            blocks[..3].iter().map(|b| b.id.clone()).collect::<Vec<_>>()
        );
        assert_eq!(
            chunks[0].raw_excerpt,
            blocks[..3]
                .iter()
                .map(|b| b.raw_text.as_str())
                .collect::<String>()
        );
        assert!(chunks.iter().all(|c| c.section.is_none()
            && c.review_status == "draft_pending_layout_and_tokenizer_review"));
        assert_eq!(
            chunks[0].content_sha256,
            crate::sha256(chunks[0].text.as_bytes())
        );
        let encoded = serde_json::to_string(&chunks).unwrap();
        let decoded: Vec<Chunk> = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded[0].id, chunks[0].id);
    }

    #[test]
    fn preserves_and_isolates_layout() {
        let blocks = build_blocks("doc", &[page(1,
            "Intro\n\n- item\n- next\n\nName  Value\nA     B\n\n    code();\n\n1. numbered\n\nEnd")]);
        assert_eq!(
            blocks.iter().map(|b| b.kind.as_str()).collect::<Vec<_>>(),
            vec![
                "paragraph",
                "layout",
                "layout",
                "layout",
                "layout",
                "paragraph"
            ]
        );
        let (chunks, warnings) = build_chunks("doc", &blocks, 100);
        assert!(warnings.is_empty());
        assert_eq!(chunks.len(), 6);
        for (chunk, block) in chunks.iter().zip(&blocks) {
            assert!(chunk.overlap_block_ids.is_empty());
            assert_eq!(chunk.raw_excerpt, block.raw_text);
            if block.kind == "layout" {
                assert_eq!(chunk.text, block.raw_text);
            }
        }
    }

    #[test]
    fn unicode_limits_and_whole_overlap_make_progress() {
        let blocks = build_blocks("doc", &[page(1, "éé\n\n界界\n\nöö\n\n終終")]);
        let (chunks, warnings) = build_chunks("doc", &blocks, 6);
        assert!(warnings.is_empty());
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].text, "éé\n\n界界");
        assert_eq!(chunks[1].text, "界界\n\nöö");
        assert_eq!(chunks[2].text, "öö\n\n終終");
        assert_eq!(chunks[1].overlap_block_ids, vec![blocks[1].id.clone()]);
        assert!(chunks.iter().all(|c| c.text.chars().count() <= 6));
        for chunk in chunks.iter().skip(1) {
            assert!(chunk
                .block_ids
                .iter()
                .any(|id| !chunk.overlap_block_ids.contains(id)));
        }
        let (tight, _) = build_chunks("doc", &blocks, 3);
        assert_eq!(tight.len(), 4);
        assert!(tight.iter().all(|c| c.overlap_block_ids.is_empty()));
    }

    #[test]
    fn oversized_sentence_and_line_fragments_reconstruct_raw_exactly() {
        for raw in [
            "Alpha ends. Beta ends. Gamma ends.\r\n",
            "alpha wraps\r\nbeta wraps\r\ngamma wraps\r\n",
            "inter-\nnational text\n",
        ] {
            let blocks = build_blocks("doc", &[page(2, raw)]);
            let (chunks, warnings) = build_chunks("doc", &blocks, 14);
            assert!(chunks.len() >= 2);
            assert_eq!(warnings.len(), 1);
            assert_eq!(
                chunks
                    .iter()
                    .map(|c| c.raw_excerpt.as_str())
                    .collect::<String>(),
                raw
            );
            assert!(chunks.iter().all(|c| c.text.chars().count() <= 14
                && c.block_ids == vec![blocks[0].id.clone()]
                && c.overlap_block_ids.is_empty()));
            assert_ne!(chunks[0].id, chunks[1].id);
            for chunk in &chunks {
                let reconstructed: String = chunk
                    .raw_fragments
                    .iter()
                    .map(|fragment| {
                        let block = blocks
                            .iter()
                            .find(|block| block.id == fragment.block_id)
                            .unwrap();
                        block.raw_text[fragment.byte_start..fragment.byte_end].to_string()
                    })
                    .collect();
                assert_eq!(reconstructed, chunk.raw_excerpt);
            }
        }
    }

    #[test]
    fn quarantine_is_atomic_and_never_truncates_or_bridges() {
        let blocks = build_blocks("doc", &[page(1,
            "aa\n\nbb\n\n- a very long layout row\n\ncc\n\nunbreakablewordtoolong\n\nValid. Anotherunbreakablewordtoolong\n\ndd")]);
        let (chunks, warnings) = build_chunks("doc", &blocks, 6);
        assert_eq!(blocks.len(), 7);
        assert_eq!(warnings.len(), 3);
        assert_eq!(chunks.len(), 3);
        assert!(chunks.iter().all(|c| c.overlap_block_ids.is_empty()));
        for index in [2, 4, 5] {
            assert!(warnings.iter().any(|w| w.contains(&blocks[index].id)));
            assert!(chunks
                .iter()
                .all(|c| !c.block_ids.contains(&blocks[index].id)));
        }
        let (none, zero_warnings) = build_chunks("doc", &blocks, 0);
        assert!(none.is_empty());
        assert_eq!(zero_warnings.len(), blocks.len());
        assert!(build_chunks("doc", &[], 10).0.is_empty());
    }

    #[test]
    fn ids_are_stable_and_bound_to_content_document_location_and_limit() {
        let blocks = build_blocks("doc", &[page(1, "Original.")]);
        let first = build_chunks("doc", &blocks, 50).0;
        assert_eq!(first[0].id, build_chunks("doc", &blocks, 50).0[0].id);
        assert_ne!(first[0].id, build_chunks("doc", &blocks, 51).0[0].id);
        assert_ne!(first[0].id, build_chunks("other", &blocks, 50).0[0].id);
        for pages in [
            vec![page(2, "Original.")],
            vec![page(1, "Changed.")],
            vec![page(1, "\nOriginal.")],
        ] {
            let changed = build_blocks("doc", &pages);
            assert_ne!(blocks[0].id, changed[0].id);
            assert_ne!(first[0].id, build_chunks("doc", &changed, 50).0[0].id);
        }
        let other = build_blocks("other", &[page(1, "Original.")]);
        assert_ne!(blocks[0].id, other[0].id);
    }
}
