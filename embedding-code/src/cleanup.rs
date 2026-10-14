//! Only inspected source revisions receive automatic margin cleanup.
use serde::Serialize;

use crate::chunk::Page;

pub const CLEANUP_VERSION: &str = "verified-margins-v2";
const BEGINNERS_SHA256: &str = "3e32d4176274c8ffcabd432235bdac313160ee77afbc15592e4cf5f9124793ec";

const CFI_SHA256: &str = "50f6e16003682e96f7bd5fc05d5947fcbcc3efba5e8c37afdca908f802251049";

#[derive(Debug, Serialize)]
pub struct ExcludedLine {
    pub page: usize,
    pub line: usize,
    pub raw_text: String,
    pub reason: String,
    pub rule: &'static str,
}

pub fn clean_pages(source_sha256: &str, pages: &[Page]) -> (Vec<Page>, Vec<ExcludedLine>) {
    if source_sha256 == BEGINNERS_SHA256 {
        return clean_verified_margins(pages, false);
    }
    if source_sha256 == CFI_SHA256 {
        return clean_verified_margins(pages, true);
    }
    (pages.to_vec(), Vec::new())
}

fn footer(text: &str, page: usize) -> bool {
    let rest = match text.strip_prefix("DayTrading.com | Day Trading For Beginners") {
        Some(value) => value,
        None => return false,
    };
    let expected = format!("{} / 17", page.saturating_sub(1));
    rest.split_whitespace().collect::<Vec<_>>().join(" ") == expected
}

fn clean_verified_margins(pages: &[Page], cfi: bool) -> (Vec<Page>, Vec<ExcludedLine>) {
    let mut cleaned = Vec::new();
    let mut excluded = Vec::new();
    for page in pages {
        let lines: Vec<&str> = page.raw_text.split_inclusive('\n').collect();
        let last_nonblank = lines.iter().rposition(|line| !line.trim().is_empty());
        let first_nonblank = lines.iter().position(|line| !line.trim().is_empty());
        let mut text = String::new();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            let compact = trimmed.split_whitespace().collect::<Vec<_>>().join(" ");
            let cfi_footer = format!("corporatefinanceinstitute.com {}", page.number);
            let reason = if cfi {
                if (1..=116).contains(&page.number)
                    && Some(index) == first_nonblank
                    && compact == "The Corporate Finance Institute The Complete Guide to Trading"
                {
                    Some("verified CFI running header")
                } else if (1..=116).contains(&page.number)
                    && Some(index) == last_nonblank
                    && compact == cfi_footer
                {
                    Some("verified CFI website footer and physical page counter")
                } else {
                    None
                }
            } else if page.number == 1 && !trimmed.is_empty() {
                Some("instructor-excluded cover page")
            } else if (2..=18).contains(&page.number)
                && Some(index) == first_nonblank
                && trimmed == "DAYTRADING.COM"
            {
                Some("verified running brand header")
            } else if (2..=18).contains(&page.number)
                && Some(index) == last_nonblank
                && footer(trimmed, page.number)
            {
                Some("verified running footer and printed page counter")
            } else {
                None
            };
            match reason {
                Some(value) => {
                    excluded.push(ExcludedLine {
                        page: page.number,
                        line: index + 1,
                        raw_text: line.to_string(),
                        reason: value.to_string(),
                        rule: CLEANUP_VERSION,
                    });
                    // Keep original physical line numbers for surviving blocks.
                    if line.ends_with('\n') {
                        text.push('\n');
                    }
                }
                None => text.push_str(line),
            }
        }
        cleaned.push(Page {
            number: page.number,
            raw_text: text,
            warnings: page.warnings.clone(),
        });
    }
    (cleaned, excluded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::{build_blocks, build_chunks};

    fn page(number: usize, text: &str) -> Page {
        Page {
            number,
            raw_text: text.to_string(),
            warnings: Vec::new(),
        }
    }

    #[test]
    fn cfi_cleanup_is_margin_and_revision_bound() {
        let header = "The Corporate Finance Institute   The Complete Guide to Trading";
        let raw = format!("  {}\n\nSynthetic body.\n{}\ncorporatefinanceinstitute.com 2\n\ncorporatefinanceinstitute.com    3\n", header, header);
        let pages = vec![page(3, &raw)];
        let (cleaned, audit) = clean_pages(CFI_SHA256, &pages);
        assert_eq!(audit.len(), 2);
        assert_eq!((audit[0].line, audit[1].line), (1, 7));
        assert!(cleaned[0].raw_text.contains(header));
        assert!(cleaned[0]
            .raw_text
            .contains("corporatefinanceinstitute.com 2"));
        let blocks = build_blocks("cfi", &cleaned);
        assert_eq!(blocks[0].start_line, 3);
        let (untouched, excluded) = clean_pages("unreviewed-cfi-revision", &pages);
        assert!(excluded.is_empty());
        assert_eq!(untouched[0].raw_text, raw);
        let (_, wrong_counter) = clean_pages(
            CFI_SHA256,
            &[page(4, "Synthetic body\ncorporatefinanceinstitute.com 3\n")],
        );
        assert!(wrong_counter.is_empty());
    }

    #[test]
    fn only_verified_hash_activates_cleanup() {
        let pages = vec![page(2, "DAYTRADING.COM\n\nSynthetic body.\n")];
        let (untouched, audit) = clean_pages("another-revision", &pages);
        assert_eq!(untouched[0].raw_text, pages[0].raw_text);
        assert!(audit.is_empty());
        let (cleaned, audit) = clean_pages(BEGINNERS_SHA256, &pages);
        assert_eq!(cleaned[0].raw_text, "\n\nSynthetic body.\n");
        assert_eq!(audit.len(), 1);
    }

    #[test]
    fn margin_only_and_cover_exclusion_preserve_body_and_original_lines() {
        let pages = vec![
            page(1, "Synthetic cover\n\nSynthetic publisher\n"),
            page(2, " DAYTRADING.COM\nSynthetic Section\n\nSynthetic body.\nDAYTRADING.COM\n\nDayTrading.com | Day Trading For Beginners   1 / 17\n"),
        ];
        let (cleaned, audit) = clean_verified_margins(&pages, false);
        assert_eq!(audit.len(), 4);
        assert_eq!(audit[3].line, 7);
        assert_eq!(
            audit[3].raw_text,
            "DayTrading.com | Day Trading For Beginners   1 / 17\n"
        );
        let blocks = build_blocks("doc", &cleaned);
        assert_eq!(blocks.len(), 2);
        assert_eq!((blocks[0].page, blocks[0].start_line), (2, 2));
        assert_eq!(blocks[0].text, "Synthetic Section");
        assert!(blocks[1].text.contains("DAYTRADING.COM"));
        let (chunks, _) = build_chunks("doc", &blocks, 3000);
        assert!(chunks.iter().all(|chunk| chunk.page_start == 2));
        assert!(!chunks.iter().any(|chunk| chunk.text.contains("1 / 17")));
        // Raw page evidence is never modified in place.
        assert!(pages[1].raw_text.starts_with(" DAYTRADING.COM"));
    }

    #[test]
    fn footer_requires_exact_label_counter_and_last_position() {
        assert!(footer(
            "DayTrading.com | Day Trading For Beginners  2 / 17",
            3
        ));
        assert!(!footer(
            "DayTrading.com | Day Trading For Beginners  3 / 17",
            3
        ));
        assert!(!footer(
            "DayTrading.com | Day Trading For Beginners commentary",
            3
        ));
        let pages = vec![page(
            3,
            "Synthetic content\nDayTrading.com | Day Trading For Beginners 2 / 17\nMore content\n",
        )];
        let (cleaned, audit) = clean_verified_margins(&pages, false);
        assert!(audit.is_empty());
        assert_eq!(cleaned[0].raw_text, pages[0].raw_text);
    }
}
