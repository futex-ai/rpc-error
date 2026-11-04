# rpc-error

`rpc-error` is the public runtime crate for standardized HTTP and RPC error
responses. Depend on it when a Rust service needs typed domain errors to render
as a stable JSON envelope with standard codes, unique reason codes, optional
structured data, request ids, and framework-specific response adapters.

## Responsibilities

- Own the framework-neutral error model.
- Serialize every client-facing error into the standard envelope.
- Keep standard code to HTTP status mapping consistent.
- Provide request-id, reason-catalog, and source-chain helpers.
- Expose optional Actix and Axum adapters without enabling either framework by
  default.
- Re-export `rpc-error-derive` when the `derive` feature is enabled.

## What This Crate Does

- Defines `RpcError`, `ErrorCode`, `Severity`, `ErrorOutput`, and
  `ErrorDetail`.
- Maps standard codes such as `bad-request`, `not-found`, and `internal` to
  HTTP status numbers.
- Redacts internal errors by default while preserving server-side source chains
  for logs.
- Derive mappings for `internal` keep that redaction default unless callers
  explicitly provide safe public output.
- Validates reason codes as lowercase kebab-case ASCII.
- Reports typed decode failures for unknown reasons, code/reason mismatches, and
  data shape mismatches.
- Provides `ReasonCatalog`, `ReasonCatalogs`, and `validate_unique_reasons` so
  applications can test top-level reason-code uniqueness, including delegated
  error enums.
- Provides feature-gated `axum` and `actix` modules for response rendering and
  middleware.
- Reads request IDs from a `RequestId` request extension or `x-request-id`
  header in framework middleware, generating a UUID when neither is present.
- Treats structured `data` as client-safe and log-safe context.

Standard code mapping:

| Code | HTTP status |
| --- | --- |
| `bad-request` | `400` |
| `invalid-argument` | `400` |
| `failed-precondition` | `400` |
| `out-of-range` | `400` |
| `unauthenticated` | `401` |
| `permission-denied` | `403` |
| `not-found` | `404` |
| `aborted` | `409` |
| `already-exists` | `409` |
| `resource-exhausted` | `429` |
| `cancelled` | `406` |
| `payload-too-large` | `413` |
| `unavailable` | `503` |
| `internal` | `500` |
| `deadline-exceeded` | `504` |

## Quick Start

Use the framework-neutral core without a web framework:

```rust
use rpc_error::RpcError;

let error = RpcError::not_found("book-not-found", "Book was not found")?
    .with_request_id("req_123");

let body = error.output();
# Ok::<(), rpc_error::Error>(())
```

Attach public structured data to the response and logs:

```rust
use rpc_error::RpcError;
use serde_json::json;

let error = RpcError::bad_request("token-rejected", "Token was rejected")?
    .with_data(json!({ "token_hint": "abc..." }))?;

assert!(error.output().error.data.is_some());
assert!(error.log_record().data.is_some());
# Ok::<(), rpc_error::Error>(())
```

Use Axum through a private git pin:

```toml
[dependencies.rpc-error]
git = "https://github.com/futex-ai/rpc-error.git"
rev = "<commit>"
features = ["derive", "axum"]
```

Actix callers enable `features = ["derive", "actix"]`.

Derive users with structured data fields should also depend on `serde` directly:

```toml
serde = { version = "1", features = ["derive"] }
```

Axum callers typically install the middleware so returned `RpcError` values use
the request ID chosen for the request:

```rust
use axum::{middleware, Router};

let app: Router<()> = Router::new()
    .layer(middleware::from_fn(rpc_error::axum::log_errors));
```

Actix callers use `RpcErrorLogger`:

```rust
use actix_web::App;

let app = App::new().wrap(rpc_error::actix::RpcErrorLogger);
```

Both adapters read a `rpc_error::RequestId` request extension first, then the
`x-request-id` header, and generate a UUID when neither exists.

## Development

Run the local checks from the workspace root:

```bash
cargo build --workspace
cargo test --workspace
cargo xtask check
```

### Key Code

- `src/code.rs` - standard code definitions and HTTP status mapping.
- `src/runtime.rs` - `RpcError` and server-side log metadata.
- `src/output.rs` - serializable JSON envelope types.
- `src/catalog.rs` - reason-code catalog validation.
- `src/axum.rs` - Axum adapter behind the `axum` feature.
- `src/actix.rs` - Actix adapter behind the `actix` feature.

### Related Docs

- [Workspace README](../../README.md)
- [RPC error protocol](../../docs/protocol/rpc-error/README.md)
