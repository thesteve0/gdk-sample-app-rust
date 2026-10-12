# Lesson 12 plan — Integrate specialized educational retrieval

**Planning only — 2026-10-11. No implementation authorized or supplied.** Read [the shared handoff](../final-lessons-handoff.md) first. This is the first of the final three required lessons.

## Goal and transferable concepts

Connect the existing learning partner to a supplied PostgreSQL database using **pgvector**, an extension that stores vectors and supports similarity search. A **vector** is a numeric representation of text produced by an embedding model. **Retrieval** selects relevant source passages to accompany a model request; it does not retrain the model. **Provenance** identifies where a passage came from so a reader can inspect its support.

Teach bounded access to external evidence, source-grounded answers, absence of evidence, and the separation of trusted instructions from untrusted retrieved content. Day-trading books provide the example, not a new trading feature.

## Agreed scope and corpus

The instructor has private EPUB/PDF books, not expert-authored learning rubrics. The datastore and data will be prepared for learners outside the lesson. Learners integrate retrieval; they do not install PostgreSQL, ingest documents, choose chunking algorithms, build indexes, or train embeddings in class.

Retrieve specialized explanations, book-specific examples, and available exercises. Compare a learner's answer to identified passages and ask for clarification where evidence is limited. Do not invent expert assessment criteria or infer mastery/readiness to trade. Book claims may conflict or be outdated; disagreement is a result to explain, not average away. Demonstrate visible use of source-specific details, not merely a generic plausible answer.

## Prerequisites to settle before implementation

No schema, embedding model, embedding service, or database client has been selected. Ask the instructor to settle a preparation contract, with implementation assistance if needed, before coding against it:

- PostgreSQL/pgvector versions, reachable host/database, TLS/authentication, read-only role, connection limits and timeouts; credentials supplied privately through environment/configuration, never committed or logged.
- Query-embedding model identifier/version, dimensions, preprocessing and any query/document prefixes, service/local execution, cost/privacy, failure behavior. Query vectors must match the preparation pipeline, not just have the same length. The chat provider need not be the embedding provider.
- Vector distance metric and index behavior; scores must not be described as probabilities of truth. Threshold/top-k tuning is evaluated, not assumed universal.
- Actual tables/views and column mappings; data availability and collection restrictions. Database contents are not yet verified ready.
- Citation/location quality and corpus licensing/provider-disclosure permissions. Decide whether any specialized text can be distributed to learners or used only privately.
- A supplied nonsecret setup example and small legally redistributable corpus for repeatable integration checks, distinct from private-book validation.

## Proposed integration contract (conceptual, not finalized SQL)

Application input: a bounded search query and application-owned collection restrictions. Application validates length, result count, and scope; the model cannot supply SQL, a table name, credentials, or arbitrary database identifiers.

Returned passages need: stable document/passage IDs; title and author when known; edition/publication information when available; exact excerpt; useful source location; corpus/preparation version; and a similarity value labeled according to the selected metric. Distinguish source publication date from ingestion date. PDF physical page index may differ from printed page numbers; EPUB chapter/anchor locations must not be invented as pages. Unknown metadata stays unknown.

Flow: validate request → embed query → parameterized read-only vector query → return bounded excerpts/provenance → correlated tool response → tutor answer with citations. No write privileges or general SQL tool. Keep all-request/correlation safeguards and bound database wait, embedding wait, excerpt size, and aggregate prompt contribution. The retrieved text is data, not a source of instructions.

Return distinguishable outcomes for no relevant evidence, unavailable service, malformed data, and successful retrieval. Do not turn a database failure into a false claim that the books contain no answer. Citation IDs must map to passages actually returned; label unsupported additions or ask a narrower question. Prompt instructions alone cannot guarantee support or resist injected text.

## Supplied code and classroom rhythm when authorized

Supply the full reference, database/embedding adapters, minimal configuration example, tests, lesson prose, and instructor notes. Choose the narrowest viable Rust dependencies using current official docs; do not guess provider embedding APIs or upgrade the GDK for convenience.

Predict whether a course/book-specific question needs retrieval; run; inspect retrieved source and tutor claim in private diagnostics; explain query/result boundaries; tweak the question to ask about a different source-supported detail; run again. An additional no-evidence question demonstrates uncertainty without deliberately breaking infrastructure. Keep source inspection out of ordinary learner dialogue except readable citations.

## Acceptance and validation

- Real supplied datastore and matching query embeddings exercised with the live provider; evidence and actual tool-selection pattern recorded.
- Source-specific answer demonstrably depends on retrieved passages; citations resolve and support the associated claims. An answer merely resembling the book is not proof of dependence.
- No-evidence and source-conflict cases handled honestly; unavailable services are reported separately.
- Strict arguments, parameterized query, read-only access, bounds, unknown/excess/correlation behavior, plain UI, memory consent and independent approval preserved.
- Unit/integration tests cover shape, mapping, bounds, empty results, failure classification, and citation membership. Similarity and semantic support need additional inspection; a citation-membership test cannot establish truth.
- Standard repository Rust checks and the documented live command pass; document private-data handling, actual setup, outcomes and unvalidated behavior.

Next: Lesson 13 manages the working context as retrieved evidence and conversation accumulate. Full ingestion and historical simulation remain outside this lesson.
