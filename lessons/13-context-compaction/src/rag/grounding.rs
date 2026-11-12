// Lesson 13
//! Per-turn evidence selection, independent of Session storage and presentation.
use super::{citations::source_file_name, retrieval};
use std::fmt::Write;

/// Application-owned selection threshold, statement cap and search-response budget.
pub struct GroundingPolicy {
    pub min_similarity: f64,
    pub max_statements: usize,
    pub token_budget: usize,
}

#[derive(Clone, Debug)]
pub struct TurnGrounding {
    // "none" before preparation (kickoff), "grounded", "no_evidence", "unavailable"
    pub status: String,
    pub query: String,
    pub threshold: f64,
    pub notice: Option<String>,
    pub evidence: Vec<retrieval::Evidence>,
}

impl Default for TurnGrounding {
    // Kickoff starts explicitly ungrounded; the comment above must match the
    // state value, so Default carries "none" rather than an empty string.
    fn default() -> Self {
        Self {
            status: "none".to_string(),
            query: String::new(),
            threshold: 0.0,
            notice: None,
            evidence: Vec::new(),
        }
    }
}

impl TurnGrounding {
    // Injected as a prompt part on each inference request this turn; never saved
    // into conversation history, so compaction and learner-quote provenance are
    // unaffected.
    pub fn prompt_part(&self, contract: &str) -> Option<(String, String)> {
        if self.evidence.is_empty() {
            return None;
        }
        // The application supplies the authored contract; excerpts remain untrusted data.
        let mut body = String::from(contract);
        for passage in &self.evidence {
            let _ = writeln!(
                body,
                "\n{} pages {}-{} similarity {:.2}\n{}\n{}",
                source_file_name(&passage.source_file),
                passage.page_start,
                passage.page_end,
                passage.similarity,
                passage.citation_id,
                passage.excerpt
            );
        }
        Some(("source-grounding".to_string(), body))
    }

    // Human-readable one-line grounding summary for the diagnostics log; the
    // complete Debug value is already captured in session_fields.
    pub fn grounding_summary(&self) -> String {
        let mut summary = format!(
            "status: {}, threshold: {:.2}, candidates selected: {}, query: {:?}",
            self.status,
            self.threshold,
            self.evidence.len(),
            self.query,
        );
        if let Some(notice) = &self.notice {
            summary.push_str(&format!("\nnotice: {}", notice));
        }
        summary
    }
}

pub fn select_grounding(
    query: &str,
    response: &retrieval::SearchResponse,
    policy: &GroundingPolicy,
) -> TurnGrounding {
    // SQL returns candidates ranked best-first; keep the top matches above threshold.
    let evidence: Vec<retrieval::Evidence> = response
        .evidence
        .iter()
        .filter(|passage| passage.similarity >= policy.min_similarity)
        .take(policy.max_statements)
        .cloned()
        .collect();
    let status = if evidence.is_empty() {
        "no_evidence"
    } else {
        "grounded"
    };
    TurnGrounding {
        status: status.to_string(),
        query: query.to_string(),
        threshold: policy.min_similarity,
        notice: None,
        evidence,
    }
}

/// Retains the full search result alongside the per-turn selection, without logging it.
pub struct GroundingPreparation {
    pub grounding: TurnGrounding,
    pub response: Result<retrieval::SearchResponse, String>,
}

pub async fn prepare_grounding(
    retrieval: &retrieval::Retrieval,
    query: &str,
    policy: &GroundingPolicy,
) -> GroundingPreparation {
    let response = retrieval.search(query, policy.token_budget).await;
    let grounding = match &response {
        Ok(response) => select_grounding(query, response, policy),
        Err(failure) => TurnGrounding {
            status: "unavailable".to_string(),
            query: query.to_string(),
            threshold: policy.min_similarity,
            notice: Some(failure.clone()),
            evidence: Vec::new(),
        },
    };
    GroundingPreparation {
        grounding,
        response,
    }
}
