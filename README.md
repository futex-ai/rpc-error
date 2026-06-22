# rpc-error

`rpc-error` is a Rust workspace for typed HTTP, server, request, and RPC error
handling. It standardizes error responses, maps domain errors to stable reason
codes, and provides feature-gated Actix and Axum adapters so applications can
opt into only the web framework they use.

The active implementation plan is tracked in [plans](./plans/README.md).

## Key Features

- Framework-neutral core error types with a stable structured JSON envelope.
- A finite response code set that maps cleanly to HTTP status codes.
- Stable, unique reason codes for application-level error handling.
- Optional typed `data` payloads for structured client and log context.
- Feature-gated `axum` and `actix` response adapters.
- Logging middleware that emits request method, path, status, code, reason,
  severity, and server-side source-chain reports without leaking internal
  details to clients.
- A derive macro for converting `thiserror` enums into standardized RPC
  errors and back from serialized error envelopes.

## Interface

Git dependency usage:

```toml
[dependencies.rpc-error]
git = "https://github.com/futex-ai/rpc-error.git"
rev = "<commit>"
features = ["derive", "axum"]
```

Actix callers enable `actix` instead of `axum`.

Canonical error responses use this shape:

```json
{
  "error": {
    "code": "not-found",
    "reason": "book-not-found",
    "message": "Book was not found",
    "request_id": "018f7a98-9e0d-7a34-9e66-c43c8fdcf217",
    "data": {
      "book_id": "00000000-0000-0000-0000-000000000000"
    }
  }
}
```

## Developer Get Started

```bash
cargo build --workspace
cargo test --workspace
cargo xtask check
cargo xtask review
```

Documentation-only changes should validate Markdown and review the diff.

## Key Code

- `docs/protocol/rpc-error` - normative error contract and adapter behavior.
- `docs/integration` - downstream migration guidance.
- `plans` - active and completed implementation plans.
- `crates/rpc-error` - framework-neutral public crate and adapters.
- `crates/rpc-error-derive` - procedural macro crate.
- `xtask` - local verification and AI review commands.

## Links

- [Plans](./plans/README.md)
- [Documentation](./docs/README.md)
- [RPC error protocol](./docs/protocol/rpc-error/README.md)
- [Accounting integration](./docs/integration/accounting.md)
