# Lesson 2 Instructor Notes: Provider Smoke Test

## Teaching objective

Connect Lesson 1's architecture vocabulary to code while separating provider configuration and connectivity from inference. Learners should understand that the application constructs a provider as its interface to a model service, that a provider can expose models, and that construction, discovery, and generating a response are separate checks.

## Suggested pacing

- Reconnect the application → provider → model path from Lesson 1: 5 minutes
- Review provider JSON: 10 minutes
- Explain provider construction and the provider abstraction: 10 minutes
- Walk through `fetch_supported_models()`: 10 minutes
- Run the smoke test and interpret output: 10 minutes
- Review and questions: 5 minutes

## Before class

- Ensure the shared endpoint or local server is available when using a dynamically discovered provider.
- Test the JSON against the endpoint and confirm the configured model ID matches a returned ID when `dynamic_models` enables discovery.
- Have an alternate hardware-appropriate configuration ready.
- Check the actual root checkpoint before teaching commands.

## Discussion prompts

- Which component owns provider configuration, and which component is the configured interface to the model service?
- Why is model discovery not an inference request?
- What does `from_json` validate and configure locally?
- Why does a successful model-discovery request not prove inference works?
- Why should the application use the provider abstraction instead of rebuilding its request URL, headers, and authentication?
- How does `dynamic_models` determine whether the configured list is static or discovered?

## Live-demo cautions

- Provider availability is part of validation when the configuration dynamically discovers models. Do not replace it with a mock without approval.
- The exact pinned public `goose-providers` API exposes `fetch_supported_models()`; use it rather than manually calling `/v1/models`.
- The bundled configuration leaves `dynamic_models` unset. Its OpenAI-compatible provider attempts model discovery and falls back to configured models only for a missing models endpoint (404); other errors, including authentication failures, remain errors.
- Never display or commit a real key.

## Terminal presentation guidance

- Use bold cyan headings, green successful connection/model checks, yellow model mismatch, and bold red stderr errors as visual cues; model IDs and metadata stay normal.
- The mismatch remains a stdout warning, not a failed run. The supplied `main` wrapper prints fatal errors once to stderr and exits 1; discovery behavior is unchanged.
- stdout and stderr are gated independently by `IsTerminal`; redirected output, a present `NO_COLOR` (including empty), or `TERM=dumb` disables styling. Plain output has the same strings and layout.
- Treat the local presentation helpers and error wrapper as supplied support, not a Rust fill-in exercise. There are no new printed user-input sites or payload fences; color is not protocol data or a validation criterion.

## Checkpoint

Learners can distinguish provider parsing, provider-managed model discovery, and inference, and can explain why configuration-specific request details belong in the provider library.


## Terminal styling validation — 2026-10-06

- Using this lesson's complete source with the shared pinned manifest in temporary staging, `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets`, and `cargo test` passed (2 tests). Root source and dependencies were not replaced.
- Successful terminal-colored, redirected/plain, and empty-`NO_COLOR` runs passed. `TERM=dumb` and independent stdout/stderr gating were also verified with error-output probes; errors retained exit status 1 and terminal stderr was bold red.
- Live model discovery returned the configured model and printed the successful match.
