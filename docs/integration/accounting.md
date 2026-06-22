# Accounting Axum Migration

`/Users/calummoore/projects/futex/accounting` is the first downstream target for
`rpc-error`. Its `bkf-api` crate currently uses Axum and renders manual
`(StatusCode, reason, message)` tuples into an envelope where `error.code`
contains the application reason string.

## Dependency

After this repository is committed, Accounting should pin the exact revision:

<!-- markdownlint-disable MD013 -->

```toml
[dependencies]
rpc-error = { git = "https://github.com/futex-ai/rpc-error.git", rev = "<commit>", features = ["derive", "axum"] }
serde = { version = "1", features = ["derive"] }
```

<!-- markdownlint-enable MD013 -->

This plan cannot validate a final git `rev` from inside the uncommitted
workspace. The path has been validated by compiling
`crates/rpc-error/examples/accounting_axum.rs` with `features = ["derive",
"axum"]`; the final downstream check is to replace the local path with the
commit hash produced from this workspace and run Accounting's API tests.

## Response Shape Change

Accounting should move from the current shape:

```json
{
  "error": {
    "code": "validation_failed",
    "message": "Validation failed",
    "request_id": "req_123"
  }
}
```

to the standardized shape:

```json
{
  "error": {
    "code": "bad-request",
    "reason": "validation-failed",
    "message": "Validation failed",
    "request_id": "req_123"
  }
}
```

Accounting is not production/live, so this breaking response-shape change is
acceptable and should be made directly instead of carrying a compatibility mode.

## Top-Level Error Pattern

Accounting can convert its current mapping helpers into a top-level enum:

```rust
use rpc_error::{RpcError, validate_unique_reasons};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, thiserror::Error, RpcError, Serialize, Deserialize)]
enum ApiError {
    #[rpc_error(
        code = "bad-request",
        reason = "validation-failed",
        message = "Validation failed"
    )]
    #[error("[bkf-api] validation failed")]
    ValidationFailed,

    #[rpc_error(
        code = "unauthenticated",
        reason = "unauthorized",
        message = "Authentication required"
    )]
    #[error("[bkf-api] authentication required")]
    Unauthorized,

    #[rpc_error(
        code = "internal",
        reason = "internal-error",
        message = "Internal server error"
    )]
    #[error("[bkf-api] internal error")]
    Internal,
}

fn assert_reason_catalog_is_unique() -> rpc_error::Result<()> {
    validate_unique_reasons(&ApiError::reason_catalogs())
}
```

The real migration can keep domain-specific mapping modules while having each
module return `RpcError` or a top-level derived enum variant instead of
`(StatusCode, &'static str, String)`.

## Targeted Tests

After Accounting pins a committed `rpc-error` revision, run:

```bash
cargo test -p bkf-api error
cargo test -p bkf-api api_smoke::core_metadata_tests
cargo test -p bkf-api --test pagination_tests
```

Those tests cannot be run as a meaningful validation from this workspace before
Accounting depends on the committed `rpc-error` revision. The local proof in
this repo is the compiled `accounting_axum` example plus the Axum adapter smoke
server test.
