# RPC Error

`rpc-error` standardizes HTTP, server, request, and RPC error responses for Rust
services. This page defines the public contract the implementation must follow.

## Scope

- Expose a framework-neutral Rust error model.
- Render every client-facing error as structured JSON.
- Assign every mapped application error a stable reason code.
- Support Actix and Axum through optional crate features.
- Provide simple request-aware logging middleware for standardized errors.
- Support private git dependency installation by pinned commit.

Out of scope:

- Generating OpenAPI schemas.
- Owning application-specific error catalogs.
- Publishing to crates.io in the initial release.
- Replacing application domain error enums.

## Workspace Shape

The workspace must include:

- `crates/rpc-error` as the public runtime crate.
- `crates/rpc-error-derive` as the procedural macro crate.
- `xtask` for local checks and AI review.

The `rpc-error` crate owns the public API. The derive crate is re-exported from
`rpc-error` behind the `derive` feature.

Framework dependencies are optional:

- `axum` enables Axum `IntoResponse` support and Tower middleware.
- `actix` enables Actix `ResponseError` support and middleware.
- `derive` enables the derive macro re-export.

Default features must not pull in a web framework.

## Error Model

The core error type must contain:

- `code`: a finite standard code such as `bad-request`, `not-found`, or
  `internal`.
- `reason`: a stable application reason code such as `book-not-found`.
- `message`: a client-safe message.
- `data`: optional structured JSON context.
- `severity`: `warn` for expected request failures or `error` for operational
  failures.
- `source`: optional server-side error source chain that is never serialized.

Internal errors must default to:

- HTTP status `500`.
- `code = "internal"`.
- `reason = "internal-error"`.
- `message = "Internal server error"`.
- `severity = "error"`.

Applications may override the internal reason only when it remains client-safe
and stable.

## Standard Codes

The standard code set must include:

- `bad-request`
- `invalid-argument`
- `failed-precondition`
- `out-of-range`
- `unauthenticated`
- `permission-denied`
- `not-found`
- `aborted`
- `already-exists`
- `resource-exhausted`
- `cancelled`
- `payload-too-large`
- `unavailable`
- `internal`
- `deadline-exceeded`

Each code maps to one HTTP status. The mapping must be documented in the
`rpc-error` crate README and tested for Actix and Axum adapters.

## JSON Envelope

Every rendered error response must be JSON with this shape:

```json
{
  "error": {
    "code": "bad-request",
    "reason": "validation-failed",
    "message": "Validation failed",
    "request_id": "018f7a98-9e0d-7a34-9e66-c43c8fdcf217",
    "data": {
      "field": "email"
    }
  }
}
```

Rules:

- `error.code`, `error.reason`, and `error.message` are always present.
- `error.request_id` is present when a request ID provider is configured.
- `error.data` is omitted or `null` when no structured context is provided.
- Serialized responses must use kebab-case code and reason strings.
- The source chain must be available for logs but never included in JSON.
- Framework middleware must accept a request ID supplied as a `RequestId`
  request extension or `x-request-id` header and must generate one when no
  application ID is present.

## Reason Codes

Reason codes are stable client-facing identifiers.

Rules:

- Reason codes use lowercase kebab-case ASCII.
- A reason code identifies one semantic error condition in a top-level API
  contract.
- Renaming a reason code is a breaking API change.
- The derive macro must reject duplicate reason codes within one enum.
- The runtime crate must provide catalog helpers so applications can test that
  composed enums do not reuse reason codes accidentally.
- Derived top-level enums must provide a helper that includes directly mapped
  and delegated reason catalogs so validation cannot miss delegated enums.
- Generic fallback reasons such as `validation-failed` are allowed only when
  the application intentionally groups multiple equivalent validation failures.

## Derive Conversion

The derive macro must support:

- Mapping enum variants to standard codes and reason codes.
- Optional public message overrides.
- Optional `warn` severity.
- Optional omission of variant data from the serialized response.
- The `data` argument must reject unsupported values at compile time.
- Variant data is included by default when a mapped variant carries fields, and
  included data is treated as safe for both client responses and server logs.
- Variants mapped to `internal` default to `message = "Internal server error"`
  and omitted `data`; callers must opt into public internal data explicitly.
- Structured-data serialization failures must produce redacted internal errors
  and preserve the serialization failure as the server-side source.
- Generated named-field and multi-field tuple data helpers must preserve
  field-level `serde` attributes such as `rename`, `skip`, and `default`.
- Single unnamed data variants serialize the inner field value directly and must
  reject field-level `serde` attributes that cannot be honored.
- Each variant may have only one RPC mapping or delegate attribute.
- Unit variants.
- Single unnamed fields.
- Multiple unnamed fields through generated tuple data structs.
- Named fields through generated named data structs.
- Delegating conversion to one or more wrapped error enums.
- Conversion from a typed error into the core RPC error.
- Conversion from the core RPC error or JSON envelope back into a typed error
  when the standard code, reason, and data shape match.
- Envelopes with a known reason and the wrong standard code must fail typed
  decoding with an explicit conversion error.
- Unit variants must reject envelopes that carry structured `data`; otherwise
  clients and servers could silently discard fields that are not part of the
  typed contract.
- Conversion into the core RPC error must not require the typed error enum or
  its fields to implement `Clone`.

The canonical attribute should be explicit, for example:

```rust
#[rpc_error(code = "not-found", reason = "book-not-found")]
```

Compatibility aliases such as `#[not_found("book-not-found")]` may be retained
to make migration from the existing `rpc-error-convert` crate cheaper.

Derive users with structured data fields must depend on `serde` directly so
their caller-owned field types can derive `Serialize` and `Deserialize`.

## Framework Adapters

Axum support must provide:

- `IntoResponse` for the core error type.
- A Tower-compatible middleware layer that logs standardized error responses.
- A way to attach request IDs from request extensions or headers and generate
  them when absent.

Actix support must provide:

- `ResponseError` for the core error type.
- Actix middleware that logs standardized error responses.
- A not-found fallback helper for unmatched routes.

Adapters must not change the JSON envelope or reason-code behavior.

## Logging Middleware

Middleware logs must include:

- HTTP method.
- Request path.
- HTTP status.
- Standard code.
- Reason code.
- Severity.
- Request ID when available.
- Structured data when the error carries data.
- A server-side source-chain report.

Middleware must log `warn` severity at warning level and `error` severity at
error level and include a structured `severity` field on each RPC error log
event. It must avoid `println!`, `eprintln!`, or string-only diagnostics in
library/runtime paths.

Response data must not contain secrets, protected identifiers, or other values
that should stay out of centralized logs. Such fields should be omitted from
`data` entirely, including with derive `data = "omit"` when variant fields are
not safe to expose.

## Downstream Compatibility

The first downstream target is `/Users/calummoore/projects/futex/accounting`,
which currently uses Axum and manual `(StatusCode, reason, message)` mappings.
The implementation must include an Axum migration example and tests that prove
the existing mappings can move to the standard `code` plus kebab-case `reason`
fields. Accounting is not production/live, so breaking response-shape changes
are acceptable when they improve the contract.

Legacy behavior from `/Users/calummoore/projects/polybase/zk-rollup-noir/pkg/rpc`
and `/Users/calummoore/projects/polybase/zk-rollup-noir/pkg/rpc-error-convert`
is reference material, not an exact compatibility mandate.

## Verification

The implementation must provide:

- Unit tests for code-to-status mapping.
- Serialization tests for every envelope field.
- Redaction tests for internal errors.
- Derive tests for unit, unnamed, named, omitted-data, severity, and delegate
  variants.
- Compile-fail tests for invalid attributes and duplicate enum-local reasons.
- Actix adapter tests with the `actix` feature.
- Axum adapter tests with the `axum` feature.
- Middleware tests that verify structured log fields where practical.
- Smoke tests that start minimal Actix and Axum apps and call failing routes.

Before implementation is considered complete, run tests, then
`cargo xtask check`, then `cargo xtask review`.
