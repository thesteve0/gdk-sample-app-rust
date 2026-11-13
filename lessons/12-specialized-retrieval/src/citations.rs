// Lesson 12
//! Lexical acknowledgment and selected-source membership; not semantic support validation.
use crate::{retrieval, TurnGrounding};
use std::collections::{HashMap, HashSet};

// Provenance for one source ID the application supplied in some turn's
// grounding, kept for the whole conversation so a citation the tutor reuses
// from an earlier turn still resolves to its reference.
pub struct CitationOrigin {
    pub source_file: String,
    pub page_start: i32,
    pub page_end: i32,
    pub similarity: f64,
}

#[derive(Default)]
pub struct CitationRegistry {
    origins: HashMap<String, CitationOrigin>,
}

impl CitationRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    // First origin wins: the ID keeps the reference it was first shown with.
    pub fn record(&mut self, evidence: &retrieval::Evidence) {
        self.origins
            .entry(evidence.citation_id.clone())
            .or_insert_with(|| CitationOrigin {
                source_file: evidence.source_file.clone(),
                page_start: evidence.page_start,
                page_end: evidence.page_end,
                similarity: evidence.similarity,
            });
    }
    pub fn get(&self, citation_id: &str) -> Option<&CitationOrigin> {
        self.origins.get(citation_id)
    }
    // Every ID ever recorded, for application reminders that list the exact
    // markers the tutor is allowed to cite.
    pub fn citation_ids(&self) -> Vec<&str> {
        self.origins.keys().map(|id| id.as_str()).collect()
    }
}

// Only excerpt IDs the application supplied in this conversation authorize
// [source:<id>] citations. The [source:none] sentinel is the sanctioned
// decline marker on grounded turns.
pub fn inspect_citations(
    grounding: &TurnGrounding,
    registry: &CitationRegistry,
    answer: &str,
) -> Vec<String> {
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
        if !allowed.contains(marker.as_str()) && registry.get(&marker).is_none() {
            warnings.push("Tutor output cites an ID the application never supplied in this conversation; no reference is available and support is unverified.".to_string());
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
    // At least one cited marker was never supplied in this conversation
    // (fabricated, stale, or malformed); no reference can exist for it.
    InvalidAcknowledgment,
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
            GroundingOutcome::InvalidAcknowledgment => {
                "the tutor cited a source ID the application never supplied; no reference exists for it"
                    .to_string()
            }
        }
    }
}

// Markers cited in the reply that the application never supplied in this
// conversation (fabricated, stale, or malformed). The caller decides whether
// to withhold the reply or escalate.
pub fn invalid_citation_markers(
    grounding: &TurnGrounding,
    registry: &CitationRegistry,
    answer: &str,
) -> Vec<String> {
    let mut invalid = Vec::new();
    for marker in retrieval::extract_citations(answer) {
        if marker == "source:none" {
            continue;
        }
        let supplied = grounding
            .evidence
            .iter()
            .any(|passage| passage.citation_id == marker)
            || registry.get(&marker).is_some();
        if !supplied && !invalid.contains(&marker) {
            invalid.push(marker);
        }
    }
    invalid
}

pub fn grounding_outcome(
    grounding: &TurnGrounding,
    registry: &CitationRegistry,
    answer: &str,
) -> GroundingOutcome {
    if grounding.evidence.is_empty() {
        return GroundingOutcome::NotSupplied;
    }
    // extract_citations("[source:none]") yields the sentinel "source:none".
    let citations = retrieval::extract_citations(answer);
    let declined = citations.iter().any(|id| id == "source:none");
    let cited: Vec<&str> = citations
        .iter()
        .map(|id| id.as_str())
        .filter(|id| *id != "source:none")
        .collect();
    if !cited.is_empty() {
        // Membership, not marker presence, decides compliance: a fabricated
        // or stale ID has no reference and must not count as acknowledgment.
        // Current-turn IDs and every ID the application supplied in an
        // earlier turn of this conversation are both valid.
        if invalid_citation_markers(grounding, registry, answer).is_empty() {
            // Citing still acknowledges the evidence even when the reply also
            // declines it; inspect_citations flags that contradiction separately.
            return GroundingOutcome::Cited;
        }
        return GroundingOutcome::InvalidAcknowledgment;
    }
    if declined {
        return GroundingOutcome::Declined;
    }
    GroundingOutcome::SilentIgnore
}

// Learner-readable provenance for the IDs the tutor actually cited this turn.
// Current-turn IDs resolve from the turn's grounding; IDs supplied in an
// earlier turn resolve through the registry so no citation the application
// ever supplied goes without one. Never-supplied IDs stay unexplained here;
// inspect_citations warns about them instead.
pub fn citation_footer(
    grounding: &TurnGrounding,
    registry: &CitationRegistry,
    answer: &str,
) -> Option<String> {
    let cited = retrieval::extract_citations(answer);
    if cited.is_empty() {
        return None;
    }
    let mut lines = Vec::new();
    for id in cited {
        if id == "source:none" {
            continue;
        }
        if let Some(passage) = grounding
            .evidence
            .iter()
            .find(|passage| passage.citation_id == id)
        {
            lines.push(format!(
                "{}: {} pages {}-{} (similarity {:.2})",
                id,
                source_file_name(&passage.source_file),
                passage.page_start,
                passage.page_end,
                passage.similarity
            ));
            continue;
        }
        if let Some(origin) = registry.get(&id) {
            lines.push(format!(
                "{}: {} pages {}-{} (similarity {:.2}) (from an earlier turn's excerpts)",
                id,
                source_file_name(&origin.source_file),
                origin.page_start,
                origin.page_end,
                origin.similarity
            ));
        }
    }
    if lines.is_empty() {
        return None;
    }
    Some(format!("Sources cited this turn:\n{}", lines.join("\n")))
}

pub fn source_file_name(path: &str) -> String {
    path.rsplit('/').next().unwrap_or(path).to_string()
}
