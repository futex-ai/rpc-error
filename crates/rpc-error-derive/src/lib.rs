//! Derive macro entrypoint for `rpc-error`.

use proc_macro::TokenStream;

mod attrs;
mod data;
mod derive;
mod from_rpc;
mod to_rpc;
mod util;

/// Derives typed conversions into and from `rpc_error::RpcError`.
#[proc_macro_derive(
    RpcError,
    attributes(
        rpc_error,
        bad_request,
        invalid_argument,
        failed_precondition,
        out_of_range,
        unauthenticated,
        permission_denied,
        not_found,
        aborted,
        already_exists,
        resource_exhausted,
        cancelled,
        payload_too_large,
        unavailable,
        internal,
        deadline_exceeded,
        delegate
    )
)]
pub fn derive_rpc_error(input: TokenStream) -> TokenStream {
    derive::derive_rpc_error(input)
}
