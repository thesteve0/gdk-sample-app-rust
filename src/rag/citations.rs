// Lesson 13
//! Lexical acknowledgment and selected-source membership; not semantic support validation.
use super::{retrieval, TurnGrounding};
use std::collections::HashSet;

pub fn inspect_citations(grounding: &TurnGrounding, answer: &str) -> Vec<String> {
    let allowed: HashSet<&str> = grounding
        .evidence
        .iter()
        .map(|passage| passage.citation_id.as_str())
        .collect();
    let mut warnings = Vec::new();
    let mut cited = false;
    let mut declined = false;
    for tail in answer.split("[source:").skip(1) {
        let id = match tail.split_once(']') {
            Some((id, _rest)) => id.to_string(),
            None => {
                warnings.push(
                    "Tutor output contains an incomplete source citation; inspect diagnostics."
                        .to_string(),
                );
                continue;
            }
        };
        if id == "none" {
            // The application-supplied decline sentinel is never an invalid ID.
            declined = true;
            if allowed.is_empty() {
                warnings.push("Tutor output used the decline marker [source:none] but no source excerpts were supplied this turn.".to_string());
            }
            continue;
        }
        cited = true;
        let marker = format!("source:{}", id);
        if !allowed.contains(marker.as_str()) {
            warnings.push("Tutor output cites an ID not selected as this turn's grounding; source support is unverified.".to_string());
        }
    }
    if cited && declined {
        warnings.push("Tutor output both cited source IDs and declared the excerpts unsupported ([source:none]); these statements contradict each other.".to_string());
    }
    // A grounded turn that cites nothing is NOT warned here: run_turn classifies
    // it as a silent ignore and escalates once, application-side.
    warnings
}

pub enum GroundingOutcome {
    // No excerpts were supplied for this turn (kickoff, no_evidence, unavailable).
    NotSupplied,
    // At least one grounding-selected source ID was cited.
    Cited,
    // The tutor explicitly declined the excerpts with [source:none].
    Declined,
    // Excerpts were supplied but the reply neither cited nor declined them.
    SilentIgnore,
}

impl GroundingOutcome {
    // Human-readable classification for diagnostics; the escalation decision
    // itself is made by run_turn, not by the model.
    pub fn summary(&self) -> String {
        match self {
            GroundingOutcome::NotSupplied => "no excerpts were supplied for this turn".to_string(),
            GroundingOutcome::Cited => "the tutor cited supplied excerpts".to_string(),
            GroundingOutcome::Declined => "the tutor declined the supplied excerpts".to_string(),
            GroundingOutcome::SilentIgnore => {
                "the tutor neither cited nor declined the supplied excerpts".to_string()
            }
        }
    }
}

pub fn grounding_outcome(grounding: &TurnGrounding, answer: &str) -> GroundingOutcome {
    if grounding.evidence.is_empty() {
        return GroundingOutcome::NotSupplied;
    }
    // extract_citations("[source:none]") yields the sentinel "source:none".
    let citations = retrieval::extract_citations(answer);
    let declined = citations.iter().any(|id| id == "source:none");
    let cited = citations.iter().any(|id| id != "source:none");
    if cited {
        // Citing still acknowledges the evidence even when the reply also
        // declines it; inspect_citations flags that contradiction separately.
        return GroundingOutcome::Cited;
    }
    if declined {
        return GroundingOutcome::Declined;
    }
    GroundingOutcome::SilentIgnore
}

pub fn citation_footer(grounding: &TurnGrounding, answer: &str) -> Option<String> {
    if grounding.evidence.is_empty() {
        return None;
    }
    let cited = retrieval::extract_citations(answer);
    if cited.is_empty() {
        return None;
    }
    let mut lines = Vec::new();
    for id in cited {
        let passage = grounding
            .evidence
            .iter()
            .find(|passage| passage.citation_id == id);
        let passage = match passage {
            Some(passage) => passage,
            None => continue,
        };
        lines.push(format!(
            "{}: {} pages {}-{} (similarity {:.2})",
            id,
            source_file_name(&passage.source_file),
            passage.page_start,
            passage.page_end,
            passage.similarity
        ));
    }
    if lines.is_empty() {
        return None;
    }
    Some(format!("Sources cited this turn:\n{}", lines.join("\n")))
}

pub fn source_file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}
