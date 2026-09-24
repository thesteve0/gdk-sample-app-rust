# Lesson 2 Instructor Notes: Provider Smoke Test

## Teaching objective

Separate provider configuration and connectivity from inference. Learners should understand that constructing a declarative provider, discovering its supported models, and generating a response are separate checks.

## Suggested pacing

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

- What does `from_json` validate and configure locally?
- Why does a successful model-discovery request not prove inference works?
- Why should the application use the provider abstraction instead of rebuilding its request URL, headers, and authentication?
- How does `dynamic_models` determine whether the configured list is static or discovered?

## Live-demo cautions

- Provider availability is part of validation when the configuration dynamically discovers models. Do not replace it with a mock without approval.
- The exact pinned public `goose-providers` API exposes `fetch_supported_models()`; use it rather than manually calling `/v1/models`.
- The bundled configuration leaves `dynamic_models` unset. Its OpenAI-compatible provider attempts model discovery and falls back to configured models only for a missing models endpoint (404); other errors, including authentication failures, remain errors.
- Never display or commit a real key.

## Checkpoint

Learners can distinguish provider parsing, provider-managed model discovery, and inference, and can explain why configuration-specific request details belong in the provider library.
