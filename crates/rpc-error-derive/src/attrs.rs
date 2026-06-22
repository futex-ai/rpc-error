//! Error mapping attribute parsing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::ParseStream;
use syn::{Attribute, Ident, LitStr, Meta, Variant};

/// Parsed variant mapping.
#[derive(Clone)]
pub struct ErrorMapping {
    /// Standard code token path.
    pub code: TokenStream,
    /// Stable reason code.
    pub reason: String,
    /// Optional message override.
    pub message: Option<String>,
    /// Optional severity override.
    pub severity: Option<TokenStream>,
    /// Whether variant fields should be serialized as data.
    pub include_data: bool,
    /// Whether this mapping uses the internal standard code.
    pub is_internal: bool,
}

/// Parsed variant attribute.
#[derive(Clone)]
pub enum ErrorAttr {
    /// Concrete error mapping.
    Mapping(ErrorMapping),
    /// Delegate to wrapped error.
    Delegate,
}

pub fn find_error_attr(variant: &Variant) -> syn::Result<Option<ErrorAttr>> {
    let mut parsed = None;
    for attr in &variant.attrs {
        let Some(ident) = attr.path().get_ident() else {
            continue;
        };
        let attr_value = match ident.to_string().as_str() {
            "rpc_error" => Some(parse_rpc_error_attr(attr)?),
            "bad_request" => Some(parse_alias_attr(attr, quote! { BadRequest }, false)?),
            "invalid_argument" => Some(parse_alias_attr(attr, quote! { InvalidArgument }, false)?),
            "failed_precondition" => Some(parse_alias_attr(
                attr,
                quote! { FailedPrecondition },
                false,
            )?),
            "out_of_range" => Some(parse_alias_attr(attr, quote! { OutOfRange }, false)?),
            "unauthenticated" => Some(parse_alias_attr(attr, quote! { Unauthenticated }, false)?),
            "permission_denied" => {
                Some(parse_alias_attr(attr, quote! { PermissionDenied }, false)?)
            }
            "not_found" => Some(parse_alias_attr(attr, quote! { NotFound }, false)?),
            "aborted" => Some(parse_alias_attr(attr, quote! { Aborted }, false)?),
            "already_exists" => Some(parse_alias_attr(attr, quote! { AlreadyExists }, false)?),
            "resource_exhausted" => {
                Some(parse_alias_attr(attr, quote! { ResourceExhausted }, false)?)
            }
            "cancelled" => Some(parse_alias_attr(attr, quote! { Cancelled }, false)?),
            "payload_too_large" => Some(parse_alias_attr(attr, quote! { PayloadTooLarge }, false)?),
            "unavailable" => Some(parse_alias_attr(attr, quote! { Unavailable }, false)?),
            "internal" => Some(parse_alias_attr(attr, quote! { Internal }, true)?),
            "deadline_exceeded" => {
                Some(parse_alias_attr(attr, quote! { DeadlineExceeded }, false)?)
            }
            "delegate" => Some(parse_delegate_attr(attr)?),
            _ => None,
        };
        if let Some(attr_value) = attr_value {
            if parsed.is_some() {
                return Err(syn::Error::new_spanned(
                    attr,
                    "only one rpc error attribute is allowed per variant",
                ));
            }
            parsed = Some(attr_value);
        }
    }
    Ok(parsed)
}

fn parse_delegate_attr(attr: &Attribute) -> syn::Result<ErrorAttr> {
    match &attr.meta {
        Meta::Path(_) => Ok(ErrorAttr::Delegate),
        _ => Err(syn::Error::new_spanned(
            attr,
            "delegate attribute does not accept arguments",
        )),
    }
}

fn parse_rpc_error_attr(attr: &Attribute) -> syn::Result<ErrorAttr> {
    let mut code = None;
    let mut details = AttrDetails::default();
    attr.parse_args_with(|input: ParseStream<'_>| {
        while !input.is_empty() {
            let name = input.parse::<Ident>()?;
            input.parse::<syn::Token![=]>()?;
            let value = input.parse::<LitStr>()?;
            match name.to_string().as_str() {
                "code" => code = Some(code_tokens(&value)?),
                "reason" => details.reason = Some(value.value()),
                "message" => details.message = Some(value.value()),
                "severity" => details.severity = Some(severity_tokens(&value)?),
                "data" => details.include_data = Some(include_data(&value)?),
                _ => {
                    return Err(syn::Error::new_spanned(
                        name,
                        "unsupported rpc_error argument",
                    ));
                }
            }
            parse_optional_comma(input)?;
        }
        Ok(())
    })?;
    let (code, is_internal) =
        code.ok_or_else(|| syn::Error::new_spanned(attr, "missing code argument"))?;
    mapping_attr(attr, code, is_internal, details)
}

fn parse_alias_attr(
    attr: &Attribute,
    code: TokenStream,
    is_internal: bool,
) -> syn::Result<ErrorAttr> {
    let mut details = AttrDetails::default();
    attr.parse_args_with(|input: ParseStream<'_>| {
        while !input.is_empty() {
            if input.peek(LitStr) && details.reason.is_none() {
                details.reason = Some(input.parse::<LitStr>()?.value());
            } else {
                let name = input.parse::<Ident>()?;
                input.parse::<syn::Token![=]>()?;
                let value = input.parse::<LitStr>()?;
                match name.to_string().as_str() {
                    "message" => details.message = Some(value.value()),
                    "severity" => details.severity = Some(severity_tokens(&value)?),
                    "data" => details.include_data = Some(include_data(&value)?),
                    _ => return Err(syn::Error::new_spanned(name, "unsupported alias argument")),
                }
            }
            parse_optional_comma(input)?;
        }
        Ok(())
    })?;
    mapping_attr(attr, code, is_internal, details)
}

#[derive(Default)]
struct AttrDetails {
    reason: Option<String>,
    message: Option<String>,
    severity: Option<TokenStream>,
    include_data: Option<bool>,
}

fn mapping_attr(
    attr: &Attribute,
    code: TokenStream,
    is_internal: bool,
    details: AttrDetails,
) -> syn::Result<ErrorAttr> {
    let reason = details
        .reason
        .ok_or_else(|| syn::Error::new_spanned(attr, "missing reason code"))?;
    validate_reason(attr, &reason)?;
    Ok(ErrorAttr::Mapping(ErrorMapping {
        code,
        reason,
        message: details.message,
        severity: details.severity,
        include_data: details.include_data.unwrap_or(!is_internal),
        is_internal,
    }))
}

fn code_tokens(value: &LitStr) -> syn::Result<(TokenStream, bool)> {
    match value.value().as_str() {
        "bad-request" => Ok((quote! { BadRequest }, false)),
        "invalid-argument" => Ok((quote! { InvalidArgument }, false)),
        "failed-precondition" => Ok((quote! { FailedPrecondition }, false)),
        "out-of-range" => Ok((quote! { OutOfRange }, false)),
        "unauthenticated" => Ok((quote! { Unauthenticated }, false)),
        "permission-denied" => Ok((quote! { PermissionDenied }, false)),
        "not-found" => Ok((quote! { NotFound }, false)),
        "aborted" => Ok((quote! { Aborted }, false)),
        "already-exists" => Ok((quote! { AlreadyExists }, false)),
        "resource-exhausted" => Ok((quote! { ResourceExhausted }, false)),
        "cancelled" => Ok((quote! { Cancelled }, false)),
        "payload-too-large" => Ok((quote! { PayloadTooLarge }, false)),
        "unavailable" => Ok((quote! { Unavailable }, false)),
        "internal" => Ok((quote! { Internal }, true)),
        "deadline-exceeded" => Ok((quote! { DeadlineExceeded }, false)),
        _ => Err(syn::Error::new_spanned(value, "unsupported rpc_error code")),
    }
}

fn severity_tokens(value: &LitStr) -> syn::Result<TokenStream> {
    match value.value().as_str() {
        "warn" => Ok(quote! { ::rpc_error::Severity::Warn }),
        "error" => Ok(quote! { ::rpc_error::Severity::Error }),
        _ => Err(syn::Error::new_spanned(value, "unsupported severity")),
    }
}

fn include_data(value: &LitStr) -> syn::Result<bool> {
    match value.value().as_str() {
        "include" | "true" => Ok(true),
        "omit" | "none" | "skip" | "false" => Ok(false),
        _ => Err(syn::Error::new_spanned(
            value,
            "unsupported data value; use include, true, omit, none, skip, or false",
        )),
    }
}

fn parse_optional_comma(input: ParseStream<'_>) -> syn::Result<()> {
    if input.is_empty() {
        return Ok(());
    }
    input.parse::<syn::Token![,]>()?;
    Ok(())
}

fn validate_reason(attr: &Attribute, value: &str) -> syn::Result<()> {
    if !value.is_empty()
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        && !value.contains("--")
    {
        return Ok(());
    }
    Err(syn::Error::new_spanned(
        attr,
        "reason code must be lowercase kebab-case ASCII",
    ))
}
