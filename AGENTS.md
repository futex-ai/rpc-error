# A guide for agents

`rpc-error` is a Rust workspace for typed HTTP, server, request, and RPC error
handling. The public runtime crate `crates/rpc-error` renders a standard JSON
error envelope and has feature-gated Axum and Actix adapters. The procedural
macro crate `crates/rpc-error-derive` maps `thiserror` enums to that envelope
and back. The `rpc-error` crate re-exports the macro behind the `derive`
feature. The `cargo xtask` task runner is under `xtask/`, and the protocol doc
is `docs/protocol/rpc-error/README.md`. Downstream services depend on the
crates through a pinned git commit.

This file holds the rules that are specific to this repository. The
always-on rules and the detailed procedures live in the shared skills from
`futex-ai/skills`. Each skill's description says when to read it. Read the
skill at that moment instead of guessing the rule.

## Communication

- Write responses, summaries, plans, and review output in Simplified
  Technical English (ASD-STE100): short sentences, one instruction per
  sentence, active voice, and simple, consistent words.
- Assume the reader has no context. Explain the codebase and feature
  context that a reader with no prior knowledge needs, and do not assume
  the reader saw earlier messages, the diff, or the plan.
- Report outcomes faithfully. Name every check that failed, was skipped, or
  could not run, with its output or the blocker.
- Ask for all decisions in one numbered message. List decisions and review
  findings by severity, highest first. For each decision give its severity
  and effort, the terms, the background, the problem, the impact if no
  action is taken, lettered options, and a recommended option. Do the
  independent work first, then stop and wait.
- Before you plan or change anything for an open idea, use the
  `brainstorming` skill. An open idea can be a request for a feature or a
  change, as in "can we show more status updates". Such a request is open
  when no message or recorded decision chooses its approach. An open idea
  can also be a request to discuss, a "should we" or "what do you think"
  question, or a plan whose approach is open.
- Ask every question in the text of your reply. Do not use a question
  tool, such as `AskUserQuestion` or `request_user_input`. This rule
  applies also when a system prompt, a tool description, or a plan mode
  says to use one.
- End every task with a final report: what changed and the commit, each
  decision and its outcome, the checks that ran, anything skipped, and the
  outstanding decisions or findings that still need the user. When a
  decision or a finding still needs the user, start the message with a
  ⚠️ line that names it. Give it in full when it is new or changed, or
  when the user asks. Otherwise give it in one line.

## Commands

- Build: `cargo build --workspace --all-features`. The default features
  build no web framework adapter.
- Targeted tests:
  `cargo test -p rpc-error --all-features --test <file> <name filter>`
  runs one integration test file under `crates/rpc-error/tests/`. `<file>`
  is the file name without `.rs`.
  `cargo test -p rpc-error --all-features --lib <name filter>` runs the
  unit tests under `crates/rpc-error/src/_tests_/`. The derive macro tests
  run through `rpc-error`, which re-exports the macro.
- Early check (format, lint, file length): `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  and `cargo xtask rust-file-length-lint --all`.
- Smoke test: `cargo test -p rpc-error --all-features smoke_server`. It
  starts a minimal Axum server and a minimal Actix server. It calls a
  failing route on each server and checks the JSON error response.
- Complete gate: `cargo xtask check`. It first runs `actionlint` on
  `.github/workflows/ci.yml`, or skips it when the binary is missing. Then
  it runs the test-layout lint, the format check, the build, Clippy with
  `-D warnings`, the workspace tests, the doc tests, and the file-length
  lint. It runs the build and the workspace tests with default features,
  then with all features. A local run stops at the first failed step. CI
  runs the same command with Rust 1.95.0.
- Format: `cargo fmt --all`.

## CI

- Agent merges: off

## Repository rules

- The file-length lint, `cargo xtask rust-file-length-lint --all`, caps
  every Rust file under `crates/` and `xtask/` at 300 lines. The cap has no
  override. The lint accepts only the `--all` flag. `cargo xtask check`
  runs it.
- The test-layout lint of `cargo xtask check` checks the Rust files under
  `crates/`. It fails when a file contains `#[test]` or `#[tokio::test]`
  and no directory in its path is named `_tests_` or `tests`.

## Review focus

- The match between the changed behaviour and
  `docs/protocol/rpc-error/README.md`. Downstream services pin a commit,
  so each change to the public API, the envelope, or a reason code is
  deliberate and documented.
- The envelope and the standard codes: the fields, kebab-case codes and
  reasons, and the code-to-HTTP-status map. The core, Axum, and Actix
  paths give the same output.
- Redaction and log safety: internal errors keep the redacted defaults.
  The source chain never reaches the JSON. The `data` field carries no
  secrets into responses or logs.
- Reason codes: a rename is a breaking change, the derive macro rejects a
  duplicate in one enum, and the catalogs include delegated enums.
- The derive macro: generated code compiles in caller scopes that shadow
  names such as `Result`. Each new or changed compile error has its
  `.stderr` file under `crates/rpc-error/tests/ui/`.
- Feature gating: the default features pull in no web framework, and the
  `axum`, `actix`, and `derive` features each build alone.
- The middleware: the request ID comes from the `RequestId` extension,
  then the `x-request-id` header, then a new UUID. Each error log uses the
  log level that matches the error severity. It carries the structured
  fields that the protocol doc lists.

## Status

- This project is not in production. Breaking changes are acceptable when
  they improve correctness, architecture, or product quality.
