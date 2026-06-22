# Plan: RPC Error Workspace Implementation

Build a standalone Rust workspace for standardized HTTP, server, request, and
RPC error handling. The library will be consumed by private git commit pins and
must support both Axum and Actix through opt-in features.

## Investigation Summary

Reference implementations:

- `/Users/calummoore/projects/polybase/zk-rollup-noir/pkg/rpc` provides an
  Actix-first `HTTPError`, standard response codes, JSON error output, and
  logging middleware.
- `/Users/calummoore/projects/polybase/zk-rollup-noir/pkg/rpc-error-convert`
  provides a derive macro for mapping `thiserror` enums to HTTP errors,
  including reason-code attributes, optional severity, structured data, and
  delegate variants.
- `/Users/calummoore/projects/futex/redloop` provides the desired standalone
  workspace shape: root README, protocol docs, plans, minimal Rust workspace,
  `xtask check`, `xtask review`, and CI.
- `/Users/calummoore/projects/futex/accounting` is the first downstream target.
  Its API crate is Axum-based and currently maintains manual
  `(StatusCode, reason, message)` mappings in `bkf-api`.

## Scope

Implement:

- `crates/rpc-error` as the public framework-neutral runtime crate.
- `crates/rpc-error-derive` as the procedural macro crate.
- Feature-gated Axum and Actix adapters in `rpc-error`.
- Structured JSON error envelopes with standard code, unique reason, message,
  optional request id, and optional data.
- Logging middleware for both frameworks.
- Error catalog helpers that let applications test uniqueness across composed
  top-level error contracts.
- Minimal workspace infrastructure: Cargo workspace, READMEs, protocol docs,
  `xtask`, GitHub Actions CI, and private git dependency guidance.

Do not implement in this repo:

- Accounting-specific API migration.
- OpenAPI generation.
- crates.io publishing.
- A hosted service.

## Decisions

- Use one public runtime crate named `rpc-error`.
- Keep web framework dependencies behind `axum` and `actix` features.
- Re-export the derive macro from `rpc-error` behind a `derive` feature.
- Default features must not enable Actix or Axum.
- Keep the canonical response envelope as `error.code`, `error.reason`,
  `error.message`, optional `error.request_id`, and optional `error.data`.
- Treat reason codes as stable client-facing API identifiers.
- Guarantee enum-local reason uniqueness in the derive macro and provide
  runtime/catalog helpers for application-level uniqueness tests.
- Redact internal errors by default.
- Preserve request IDs supplied by application middleware and generate one when
  the adapter is configured to do so.
- Include mapped variant fields as structured `data` by default, with
  `data = "omit"` for variants that should not expose fields.
- Migrate downstream responses to canonical `code` plus kebab-case `reason`
  fields rather than preserving Accounting's current snake_case reason strings.
- Preserve the useful legacy derive aliases where practical, but document the
  explicit `#[rpc_error(...)]` attribute as canonical.

## Milestone 1: Workspace Skeleton And Docs

Summary: create the standalone Rust workspace and repository automation shell.

At the end of this milestone, the repo has compiling workspace scaffolding,
protocol docs, crate READMEs, and local verification commands.

TODOs:

- [x] Add root `Cargo.toml` with members for `crates/rpc-error`,
  `crates/rpc-error-derive`, and `xtask`.
- [x] Add workspace package metadata for Rust 2024, private git consumption,
  license, repository, and shared dependency versions.
- [x] Add `Cargo.lock` and a Rust toolchain pin aligned with current Futex
  workspace practice.
- [x] Create `crates/rpc-error` with a thin `lib.rs`, module-level docs,
  feature declarations, and a crate README following the required Rust crate
  README section order.
- [x] Create `crates/rpc-error-derive` with a proc-macro manifest, thin
  `lib.rs`, and crate README following the required section order.
- [x] Create `xtask` with `check`, `review`, `rust-file-length-lint`, and Rust
  source/test layout lint commands.
- [x] Add `.cargo/config.toml` aliases for `cargo xtask`.
- [x] Add GitHub Actions CI that runs `cargo xtask check`.
- [x] Keep `README.md`, `docs/README.md`, and protocol docs linked to
  `plans/README.md`.
- [x] Run `cargo build --workspace` after adding the skeleton and fix manifest
  or dependency errors.

## Milestone 2: Core Error Contract

Summary: implement the framework-neutral error model and serialization contract.

At the end of this milestone, callers can construct standardized errors and
serialize the JSON envelope without enabling Actix or Axum.

TODOs:

- [x] Define `ErrorCode` with the protocol code set and kebab-case serde
  representation.
- [x] Implement code-to-HTTP-status mapping without depending on Actix or Axum
  concrete status types in the core model.
- [x] Define `Severity`, `RpcError`, `ErrorOutput`, `ErrorDetail`,
  `ReasonCode`, and crate-local `Error`/`Result` types.
- [x] Implement constructors for common cases: bad request, unauthenticated,
  permission denied, not found, conflict/already exists, unavailable, and
  internal.
- [x] Ensure internal errors default to redacted public output while preserving
  server-side source chains for logs.
- [x] Implement safe structured `data` serialization with a typed internal
  error path when serialization fails.
- [x] Add request-id support that lets adapters attach an existing request id
  or generate one from an injected provider.
- [x] Add catalog types for reason-code definitions and uniqueness validation.
- [x] Add unit tests for code serialization, status mapping, constructors,
  internal redaction, source-chain reports, data serialization failure, and
  catalog uniqueness checks.
- [x] Add file-based snapshot tests for representative JSON envelopes.
- [x] Update `crates/rpc-error/README.md` with the real public API examples.

## Milestone 3: Derive Macro And Typed Conversion

Summary: port and harden the typed conversion macro from the legacy
`rpc-error-convert` crate.

At the end of this milestone, `thiserror` enums can derive conversions into the
core `RpcError` and back from an error envelope when reason/data match.

TODOs:

- [x] Implement canonical `#[rpc_error(code = "...", reason = "...")]`
  parsing.
- [x] Support optional `message`, `severity`, and `data = "omit"` arguments.
- [x] Default internal mappings to redacted message and omitted data unless the
  variant explicitly opts into safe public data.
- [x] Reject variants with more than one RPC mapping or delegate attribute.
- [x] Support legacy aliases such as `#[bad_request("reason-code")]` where
  they do not compromise the new contract.
- [x] Generate conversion from the enum into `RpcError`.
- [x] Generate conversion from `RpcError` and `ErrorOutput` back into the enum
  for mapped variants.
- [x] Reject typed decoding when a known reason is paired with the wrong
  standard code.
- [x] Generate data structs for named fields and multi-field tuple variants.
- [x] Support single unnamed fields as direct typed data.
- [x] Support delegate variants with one unnamed field.
- [x] Generate a reason catalog implementation for each derived enum.
- [x] Reject duplicate reason codes within one enum at compile time.
- [x] Add `trybuild` or equivalent compile-fail tests for bad attributes,
  duplicate reason codes, invalid delegate fields, and unsupported code names.
- [x] Add runtime derive tests for unit, single-field, multi-field, named-field,
  omitted-data, warn severity, internal fallback, and delegate conversions.
- [x] Update `crates/rpc-error-derive/README.md` with runnable examples.

## Milestone 4: Axum Adapter And Middleware

Summary: add the first downstream framework adapter for Accounting's Axum API.

At the end of this milestone, Axum users can return `RpcError` from handlers,
receive the standard envelope, and attach logging middleware.

TODOs:

- [x] Add optional `axum` and `tower` dependencies behind the `axum` feature.
- [x] Implement `IntoResponse` for `RpcError`.
- [x] Ensure the Axum response uses the exact protocol envelope and status
  mapping.
- [x] Attach response extensions or equivalent metadata so middleware can log
  standardized errors after response conversion.
- [x] Implement Axum/Tower logging middleware with method, path, status, code,
  reason, severity, request id, and source-chain report fields.
- [x] Add tests for Axum `IntoResponse` status/body/header behavior.
- [x] Add middleware tests that prove warn and error severities are routed to
  the intended tracing levels where practical.
- [x] Add a minimal Axum smoke app test that starts a server and calls a failing
  route.
- [x] Add an Accounting-style example that shows replacing manual
  `(StatusCode, reason, message)` mappings with the canonical `code`,
  kebab-case `reason`, and `message` envelope.
- [x] Document `features = ["derive", "axum"]` git dependency usage.

## Milestone 5: Actix Adapter And Middleware

Summary: add Actix support without making Actix a default dependency.

At the end of this milestone, legacy Actix callers can use the same core error
contract and middleware through `features = ["derive", "actix"]`.

TODOs:

- [x] Add optional `actix-web` dependency behind the `actix` feature.
- [x] Implement Actix `ResponseError` for `RpcError`.
- [x] Ensure the Actix response uses the exact protocol envelope and status
  mapping.
- [x] Implement Actix middleware that logs standardized errors from responses.
- [x] Add a not-found fallback helper for unmatched routes.
- [x] Add Actix response and middleware tests.
- [x] Add a minimal Actix smoke app test that starts a server and calls a
  failing route.
- [x] Compare behavior against the legacy `pkg/rpc` tests and port useful
  coverage into this workspace.
- [x] Document `features = ["derive", "actix"]` git dependency usage.

## Milestone 6: Downstream Migration Proof

Summary: prove that the library can be consumed by Accounting and can preserve
current error semantics while moving to the standardized response contract.

At the end of this milestone, there is a tested migration path for
`/Users/calummoore/projects/futex/accounting`.

TODOs:

- [x] Create a local Accounting migration note under `.context/` or a checked-in
  docs page if the guidance is generally useful.
- [x] Build an example top-level `ApiError` enum that derives or delegates to
  `rpc-error` while mapping existing Accounting reason strings such as
  `validation_failed`, `unauthorized`, and `internal_error` to kebab-case
  canonical reasons.
- [x] Document the response-shape breaking change Accounting will make when it
  moves from `error.code = "validation_failed"` to `error.code =
  "bad-request"` and `error.reason = "validation-failed"`.
- [x] Add a catalog uniqueness test pattern that Accounting can copy into
  `bkf-api`.
- [x] Validate the pinned git dependency form from Accounting using
  `features = ["derive", "axum"]`.
- [x] Run targeted Accounting API error tests in a local branch or document any
  blockers that prevent running them from this workspace.
- [x] Update `rpc-error` docs with Accounting integration notes based on the
  proof.

## Milestone 7: CI, Smoke Tests, And Review

Summary: finish verification, documentation alignment, and AI review.

At the end of this milestone, the workspace is ready to pin from downstream
repositories by commit.

TODOs:

- [x] Run `cargo fmt --all -- --check`; if it fails, run `cargo fmt --all` and
  re-run the check.
- [x] Run `cargo build --workspace`.
- [x] Run `cargo build --workspace --all-features`.
- [x] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [x] Run `cargo test --workspace`.
- [x] Run `cargo test --workspace --all-features`.
- [x] Run `cargo test --doc --workspace`.
- [x] Run Actix and Axum smoke tests by starting each minimal server and calling
  failing routes.
- [x] Run `cargo xtask rust-file-length-lint --all`.
- [x] Run `cargo xtask check` after all targeted tests pass.
- [x] Run `cargo xtask review` after tests and `cargo xtask check`; do not
  automatically fix reviewer findings.
- [x] Record each `cargo xtask review` finding in the final handoff with a
  numbered item, codebase context, solution options, and a recommended option.
- [x] Update root and crate READMEs with the final public API and all feature
  flags.
- [x] Confirm the commit hash intended for downstream private git pins.

## Milestone 8: Review Follow-Up Hardening

Summary: implement the user-selected fixes from the required AI review.

At the end of this milestone, the derive macro is more hygienic in downstream
caller scopes, serialization failures preserve the correct server-side source,
and the review command honors repository instructions.

TODOs:

- [x] Fully qualify generated standard-library conversion and result paths in
  `RpcError` derive output.
- [x] Add a compile-pass regression for downstream scopes that define their own
  `Result` alias.
- [x] Preserve structured-data serialization failures as the server-side source
  instead of overwriting them with the domain error.
- [x] Add a runtime regression for derived structured-data serialization
  failures.
- [x] Remove `--ignore-rules` from `cargo xtask review`.
- [x] Update protocol and derive crate documentation with the serialization
  failure behavior.
- [x] Run `cargo xtask check` after review follow-up fixes.
- [x] Run `cargo xtask review` after `cargo xtask check`; do not automatically
  fix reviewer findings.

## Milestone 9: Serde Data Attribute Support

Summary: implement the user-selected fix for serde field attributes on derived
structured data.

At the end of this milestone, generated data helpers honor field-level serde
shape attributes where helpers exist and reject the direct single-field path
when the attribute cannot be honored.

TODOs:

- [x] Preserve field-level `serde` attributes on generated named-field data
  helper fields.
- [x] Preserve field-level `serde` attributes on generated multi-field tuple
  data helper fields.
- [x] Reject field-level `serde` attributes on single unnamed data variants.
- [x] Add runtime regressions for `serde(rename)`, `serde(skip)`, and
  `serde(default)` behavior.
- [x] Add a compile-fail regression for unsupported single-field serde
  attributes.
- [x] Update protocol and derive crate documentation with the serde attribute
  behavior.
- [x] Run `cargo xtask check` after serde attribute support.
- [x] Run `cargo xtask review` after `cargo xtask check`; do not automatically
  fix reviewer findings.

Notes:

- The current workspace changes are not committed. Downstream repositories
  should pin the commit produced from this completed change set; the current
  pre-change `HEAD` is not the intended pin.
- The first `cargo xtask review` findings were handled according to the user's
  selected options.
- The second `cargo xtask review` finding was handled according to the user's
  selected option.

## Remaining Questions

No open questions. The initial implementation should use the canonical
`code` plus kebab-case `reason` envelope, support supplied or generated request
IDs, and include mapped variant data by default unless a variant opts out.
