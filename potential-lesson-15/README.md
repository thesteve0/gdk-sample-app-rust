# Potential Lesson 15 — Encapsulated historical practice simulator

**Documentation-only draft — 2026-10-11. Not part of the committed 14-lesson course. No source or implementation is authorized.** The instructor approved the initial scope below as a draft, not a calibrated market model or settled data/engine design. Context compaction remains a separate required Lesson 13; do not combine them.

The draft explores a transferable architecture lesson: **conventional software owns the simulated world and its rules; the model helps a learner understand it.** It is not a plan for live trading, brokerage integration, a profitable strategy, or certification.

## Plain-language vocabulary

- **Historical replay:** reveal recorded observations in chronological order, as if time were advancing through that period.
- **Intraday:** observations within a trading day, not just one closing price per day.
- **Instrument:** the asset being studied; initially one stock.
- **Long-only:** practice buying shares and later selling shares already held; no betting on declines through short selling.
- **No leverage:** no borrowing to take a larger position than the simulated cash allows.
- **Order:** a request to buy or sell under specified conditions. A **fill** is a simulated execution of all or part of that request.
- **Spread:** the difference between quoted buying and selling prices. **Slippage:** execution at a price different from the reference/anticipated price.
- **Liquidity:** availability of willing trading counterparts; simplified liquidity rules do not reproduce a real exchange.
- **Bar:** an interval's opening, highest, lowest and closing prices, usually with traded volume. It does not show the order of events inside the interval.
- **Probability distribution:** a rule describing possible random outcomes and their relative chances.
- **Random seed:** the initial value used by a repeatable random-number generator.
- **Encapsulation:** keep a component's state and rules behind a narrow interface so other components cannot bypass them.

## Proposed initial scope accepted for drafting

One instrument, one short supplied historical intraday dataset, long-only positions, no leverage, explicit fees/execution assumptions, and seeded reproducible randomness. No live data, brokerage, margin, options, or multi-instrument portfolio. Order types, partial fills, time handling, data granularity and probability distributions are **unresolved**, not implicitly approved.

This is a standalone potential architecture lesson, not a prerequisite for retrieval/compaction/evaluation. Supply the engine for attendees to integrate; do not turn it into an exchange-microstructure programming course.

## Separation of concerns

| Component | Owns | Must not do |
| --- | --- | --- |
| Data adapter | Validated chronology, asset identity, source metadata and available replay observations | Let the model access future replay records through a generic query |
| Simulation engine | Simulated clock, orders, validation, positions, cash, fees, execution decisions, seeded draws and authoritative results | Ask an LLM to decide fills or calculate account balances |
| Application boundary | Explicit learner commands/confirmation, capability allowlist, errors, resource limits and state/result delivery | Interpret tutor prose as authorization or add real trading access |
| Model tutor | Explanations, questions, source-grounded feedback and discussion of alternatives | Invent fills, change state through prose, or equate profit with demonstrated understanding |
| Learner memory | Separately approved learning notes | Persist account state or certify skill from simulated outcomes |

A proposed command boundary is inspect current observations/account, submit/cancel a simulated order, and advance one replay step. The exact API is unsettled. Prefer direct explicit learner controls for state-changing commands; any model proposal needs separate learner confirmation. Results are structured, authoritative engine output; the tutor cannot override them. Waiting is valid and trading must not be forced.

## Probabilistic but reproducible execution

The engine needs no model. Given the same dataset/version, simulator code/version, configuration, initial account, action sequence, PRNG algorithm/version and seed, outcomes should repeat. Record sampled outcomes/event ordering to explain results; parallelism and draw ordering can otherwise change a seeded run.

Historical observations stay fixed. Randomness represents selected execution uncertainties **under teaching assumptions**, not fictional changes to the historical prices. Candidate mechanisms include bounded slippage, probabilistic eligibility-to-fill and simplified partial fills. None is selected or calibrated yet. Specify when a draw occurs, its units/range, dependencies on available observations, and what happens at extreme values. Preserve cash/share invariants with exact checked arithmetic; distributions cannot bypass validation.

A reproducible outcome is not a realistic one. Probabilities need evidence/calibration before being described as real-world estimates; arbitrary settings must be labeled hypothetical. Sensitivity across seeds/configurations can demonstrate uncertainty, not establish strategy performance or a confidence level for actual trading. Distinguish maximum planned loss from realized simulated loss.

## Data needed before implementation

- A licensed/permitted dataset and redistributable classroom subset or lawful private access; provenance, acquisition date, provider/version and permitted caching/sharing.
- Unambiguous instrument identity, venue/currency, chronological timestamps/time zone, market-session calendar, observation frequency and units.
- Explicit treatment of splits, other corporate actions, adjusted versus unadjusted prices, missing/duplicate/out-of-order records, halts and session boundaries.
- At minimum interval price/volume data for a simplified bar simulator. Credible spread/quote-based rules need bid/ask observations; precise fill claims may require trades/quotes or order-book information not present in bars. Do not invent missing fields as historical facts.
- Initial simulated cash, minimum share/price increments, order eligibility/timing, fees, liquidity assumptions, and executable price rules.
- Versioned probability settings with evidence or clearly labeled hypothetical status; fixed seed/action replay for tests and varied seeds for exploration.

No source, data license, schema, frequency, calibration evidence or distribution has been chosen. Daily closing prices alone are not sufficient to portray intraday execution. Research source terms before downloading or committing data.

## Ambiguity and hindsight controls

An order submitted after viewing a bar cannot fill retrospectively inside that already revealed bar. Define the next eligible execution point. If both a stop and a target fall within one later bar, their order is unknown. Options include finer data, an explicit conservative rule, or labeled probabilistic resolution; choose and explain one before coding. A limit price appearing in a bar does not establish that the learner's order would have filled.

Engine APIs must not expose future replay observations to the tutor. However, the base model may already know historical events; tool isolation does not guarantee freedom from hindsight. Label that limitation and consider less famous periods without falsifying provenance. Keep unrevealed records out of tutor prompts and learner-facing results.

## Potential classroom activity

Inspect current observations; choose an action or wait; explain a plan; calculate maximum planned loss where applicable; advance; inspect the engine's result and assumptions; discuss with retrieved evidence. Compare the same action script/seed for repeatability, then another seed to see the effect of specified uncertainties. Learner decisions should not be made by the tutor.

Any exercise must make clear that profitable results can come from weak reasoning or luck, and losses can occur despite a defensible process. The instructor lacks domain expertise; feedback is tentative/source-grounded, not a professionally validated trading assessment. The app may help explain this glossary over time but cannot validate its own simulator's realism.

## Implementation questions and future acceptance gates

Settle dataset/license/granularity, supported orders, clock advancement, intrabar resolution, fills/fees, distributions, PRNG, precision, state storage/replay, local confirmation, crash/idempotency semantics and evaluation before implementation. No recovery/exactly-once guarantee is implied. Use domain-specific review if realistic execution is a goal.

Test engine independently without an LLM: deterministic replay, no future access, order/account invariants, exact arithmetic, draw boundaries, cancellation, and invalid commands. Then test the model boundary: results quoted faithfully, no unauthorized action, no invented fills, honest assumptions and no mastery claims. Never make statistical realism a claim based on a few successful examples.

Potential deliverables, only if separately authorized: complete supplied engine/adapter and reference, data-preparation contract, conceptual sequence diagram, instructor notes, reproducibility tests, and truthful evaluation evidence. This directory intentionally contains **documentation only and no src directory**.
