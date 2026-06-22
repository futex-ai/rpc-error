# rpc-error-derive

`rpc-error-derive` provides the procedural macro used by `rpc-error` to convert
typed `thiserror` enums into standardized RPC errors. Depend on the public
`rpc-error` crate with the `derive` feature instead of depending on this macro
crate directly unless you are maintaining the macro itself.

## Responsibilities

- Parse error mapping attributes on enum variants.
- Generate conversions from typed errors into `rpc_error::RpcError`.
- Generate conversions from `RpcError` and `ErrorOutput` back into typed
  errors when the standard code, reason code, and data shape match.
- Generate direct and delegated reason-code catalog implementations.
- Reject invalid attributes and duplicate enum-local reason codes at compile
  time.

## What This Crate Does

- Supports the canonical `#[rpc_error(code = "...", reason = "...")]`
  attribute.
- Supports optional `message`, `severity = "warn"`, and `data = "omit"`
  arguments. Unknown `data` values are compile errors so typos cannot
  accidentally expose variant fields.
- Treats included variant `data` as client-safe and log-safe structured context.
- Redacts `internal` mappings by default with `"Internal server error"` and no
  structured data unless the mapping explicitly opts into `data = "include"`.
- Rejects variants with more than one RPC mapping or delegate attribute.
- Converts structured-data serialization failures into redacted internal
  errors while preserving the serialization failure as the server-side source.
- Preserves field-level `serde` attributes such as `rename`, `skip`, and
  `default` on generated named-field and multi-field tuple data helpers.
- Rejects field-level `serde` attributes on single unnamed data variants because
  those variants serialize the inner field value directly.
- Supports unit variants, single unnamed fields, multi-field tuple variants,
  named-field variants, and delegate variants.
- Rejects envelopes where a known reason is paired with the wrong standard code.
- Rejects structured `data` when decoding an envelope into a unit variant.
- Requires `#[delegate]` to be bare; arguments such as
  `#[delegate(reason = "...")]` are compile errors.
- Supports multiple delegate variants; decoding from an envelope tries each
  delegate in order and only continues past `UnknownReason`.
- Generates `reason_catalogs()` so applications can validate direct and
  delegated reason-code uniqueness with `validate_unique_reasons`.
- Retains useful legacy aliases such as `#[not_found("book-not-found")]` for
  migration from older RPC error code.

Deriving structured data variants requires callers to depend on `serde` directly
because their field types normally derive `Serialize` and `Deserialize`.

## Quick Start

Use the macro through `rpc-error`:

```rust
use rpc_error::{ErrorCode, RpcError, validate_unique_reasons};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[rpc_error(code = "not-found", reason = "book-not-found")]
    #[error("[api] book was not found")]
    BookNotFound,
}

let error: rpc_error::RpcError = ApiError::BookNotFound.into();
assert_eq!(error.code(), ErrorCode::NotFound);
validate_unique_reasons(&ApiError::reason_catalogs())?;
# Ok::<(), rpc_error::Error>(())
```

For structured data variants, add `serde` beside `rpc-error`:

```toml
serde = { version = "1", features = ["derive"] }
```

## Development

Run macro tests through the runtime crate because it re-exports the derive:

```bash
cargo test -p rpc-error --features derive
cargo test -p rpc-error --all-features
```

### Key Code

- `src/lib.rs` - proc-macro entrypoint.
- `src/attrs.rs` - attribute parsing.
- `src/derive.rs` - generated implementation orchestration.
- `src/to_rpc.rs` - typed error to `RpcError` generation.
- `src/from_rpc.rs` - `RpcError` and `ErrorOutput` decoding generation.

### Related Docs

- [Runtime crate](../rpc-error/README.md)
- [RPC error protocol](../../docs/protocol/rpc-error/README.md)
- [Implementation plan](../../plans/rpc-error-workspace-implementation.md)
