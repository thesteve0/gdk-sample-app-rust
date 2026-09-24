# Python-to-Rust Course Migration Plan

> **Temporary migration document**
>
> This records the migration decisions that remain relevant while the Rust
> course is reviewed. Delete it after the course materials are independently
> approved.

## Decision: native Rust CLI, not UniFFI bindings

This repository builds a standalone Rust command-line application. It does not
need to be called by Kotlin or Python and therefore must **not** enable the
`uniffi` feature of `goose-sdk`.

Inspection of the exact pinned `goose-sdk` `0.1.0-alpha.10` source establishes
that:

- the crate has no default features;
- its `uniffi` feature enables the UniFFI dependency and the in-process binding
  modules;
- the crate documentation describes that feature as exposing an in-process API
  to Python and Kotlin; and
- the native provider implementation is separately available in the public
  `goose-providers` crate.

The course therefore pins and uses:

```toml
[dependencies]
futures = "0.3"
goose-providers = { version = "=0.1.0-alpha.10", features = ["rustls-tls"] }
```

`rustls-tls` provides the provider library's selected HTTPS transport.
`futures::StreamExt` supplies native Rust stream consumption. `goose-sdk` is not
a dependency of the CLI.

## Native API translation

The course uses these public `goose-providers` APIs:

| Course need | Native Rust API |
|---|---|
| Construct configured provider | `declarative::from_json(&json, None, EnvKeyResolver {})` |
| Provider abstraction | `base::Provider` |
| Discover models | `Provider::fetch_supported_models()` |
| Select a request model | `model::ModelConfig::new(model_name)` |
| User/assistant turns | `conversation::message::Message::{user, assistant}().with_text(...)` |
| Start inference | `Provider::stream(&model, system, &messages, &tools).await` |
| Consume partial responses | `StreamExt::next()` over the returned message stream |

A stream item is a `Result` containing an optional partial `Message` and
optional usage metadata. The lessons use `.transpose()?` to propagate a stream
error. A text fragment comes from `message.as_concat_text()`. A non-`None`
usage value is the structural completion signal used by the reference code.

## Source snapshot and constraints

- Python course source commit: `ca6d277b4ff4e4498601d8a67c8a6a44294bf06e`
- Rust provider library: `goose-providers = "=0.1.0-alpha.10"`
- Rust toolchain: `1.94.1`
- Live-validation configuration: `custom_aa_llama_qwen3_6-35b.json` at
  `http://127.0.0.1:8080`
- Migration owner: course instructor

Keep the following constraints:

1. Pin an exact released crate version and commit `Cargo.lock` with
   `Cargo.toml`.
2. Do not depend on a moving branch or the internal Goose application crate.
3. Do not construct provider HTTP requests, authorization headers, or endpoint
   paths in application code; pass the declarative JSON to the provider
   constructor.
4. Load `.env` with `dotenvy` before provider construction, and never commit
   `.env` or credentials.
5. Keep lessons small, instructor-led, and synchronized across reference
   source, learner prose, and instructor notes.
6. Validate provider lessons against a live provider and assert structural
   outcomes rather than generated wording.

## Migration status

- [x] Native standalone-CLI architecture selected; UniFFI bindings removed.
- [x] Exact `goose-providers` version and Rust toolchain pinned.
- [x] Lesson 1 explains the dependency and architecture decision.
- [x] Lesson 2 constructs a provider and discovers models through the native
  provider abstraction.
- [x] Lessons 3 and 4 use native messages, model configuration, and streams.
- [x] Root manifest and lockfile no longer contain `goose-sdk` or UniFFI.
- [ ] Provider-calling lessons remain Draft until the instructor independently
  approves their live validations.
- [ ] Tool definition and tool-result round trip remain future lessons.

## Sources checked

- [`goose-sdk` `0.1.0-alpha.10` manifest](https://github.com/aaif-goose/goose/blob/gdk-v0.1.0-alpha.10/crates/goose-sdk/Cargo.toml)
- [`goose-sdk` `0.1.0-alpha.10` crate source](https://github.com/aaif-goose/goose/blob/gdk-v0.1.0-alpha.10/crates/goose-sdk/src/lib.rs)
- [`goose-providers` `0.1.0-alpha.10`](https://crates.io/crates/goose-providers/0.1.0-alpha.10)
- [GDK SDK overview](https://goose-docs.ai/docs/gdk/sdk)
- [GDK SDK API reference](https://goose-docs.ai/docs/gdk/sdk/api-reference)
