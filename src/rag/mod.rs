// Lesson 13
//! Reusable local retrieval and per-turn source grounding; application policy stays in main.
pub mod citations;
pub mod grounding;
pub mod retrieval;
pub mod transport;

pub use citations::{citation_footer, grounding_outcome, inspect_citations, GroundingOutcome};
pub use grounding::{
    prepare_grounding, select_grounding, GroundingPolicy, GroundingPreparation, TurnGrounding,
};
pub use retrieval::RetrievalDefaults;
pub use transport::{local_tutor_transport, validate_local_tutor};

#[cfg(test)]
mod tests;
