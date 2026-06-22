//! `RpcError` to typed error generation.

use quote::{format_ident, quote};
use syn::{Fields, Type, Variant, punctuated::Punctuated, token::Comma};

use crate::attrs::{ErrorAttr, ErrorMapping};
use crate::util::named_field_idents;

pub fn generate_from_rpc(
    enum_name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>,
    attrs: &[Option<ErrorAttr>],
) -> syn::Result<proc_macro2::TokenStream> {
    let mut arms = Vec::new();
    let mut mismatch_arms = Vec::new();
    for (variant, attr) in variants.iter().zip(attrs.iter()) {
        if let Some(generated) = from_rpc_arms(enum_name, variant, attr) {
            let generated = generated?;
            arms.push(generated.success);
            mismatch_arms.push(generated.mismatch);
        }
    }
    let fallback = fallback_arm(enum_name, variants, attrs)?;
    Ok(quote! {
        impl ::std::convert::TryFrom<::rpc_error::RpcError> for #enum_name {
            type Error = ::rpc_error::Error;

            fn try_from(
                rpc_error: ::rpc_error::RpcError,
            ) -> ::std::result::Result<Self, Self::Error> {
                match (rpc_error.code(), rpc_error.reason()) {
                    #(#arms)*
                    #(#mismatch_arms)*
                    #fallback
                }
            }
        }

        impl ::std::convert::TryFrom<::rpc_error::ErrorOutput> for #enum_name {
            type Error = ::rpc_error::Error;

            fn try_from(
                output: ::rpc_error::ErrorOutput,
            ) -> ::std::result::Result<Self, Self::Error> {
                let rpc_error = match ::rpc_error::RpcError::new(
                    output.error.code,
                    output.error.reason,
                    output.error.message,
                ) {
                    ::std::result::Result::Ok(error) => error,
                    ::std::result::Result::Err(source) => {
                        return ::std::result::Result::Err(source);
                    }
                };
                let rpc_error = match output.error.data {
                    Some(data) => rpc_error.with_data_or_internal(data),
                    None => rpc_error,
                };
                let rpc_error = match output.error.request_id {
                    Some(request_id) => rpc_error.with_request_id(request_id),
                    None => rpc_error,
                };
                <Self as ::std::convert::TryFrom<::rpc_error::RpcError>>::try_from(rpc_error)
            }
        }
    })
}

struct FromRpcArms {
    success: proc_macro2::TokenStream,
    mismatch: proc_macro2::TokenStream,
}

fn from_rpc_arms(
    enum_name: &syn::Ident,
    variant: &Variant,
    attr: &Option<ErrorAttr>,
) -> Option<syn::Result<FromRpcArms>> {
    let Some(ErrorAttr::Mapping(mapping)) = attr else {
        return None;
    };
    Some(from_mapping_arms(enum_name, variant, mapping))
}

fn from_mapping_arms(
    enum_name: &syn::Ident,
    variant: &Variant,
    mapping: &ErrorMapping,
) -> syn::Result<FromRpcArms> {
    let code = &mapping.code;
    let reason = &mapping.reason;
    let variant_name = &variant.ident;
    let body = match &variant.fields {
        Fields::Unit => unit_variant_body(enum_name, variant_name, reason),
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 && mapping.include_data => {
            let Some(field) = fields.unnamed.first() else {
                return Err(syn::Error::new_spanned(variant, "expected one field"));
            };
            let ty = &field.ty;
            deserialize_single(enum_name, variant_name, reason, ty)
        }
        Fields::Unnamed(fields) if mapping.include_data => {
            let data_struct_name = crate::data::data_struct_path(enum_name, variant_name);
            let names = (0..fields.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect::<Vec<_>>();
            deserialize_wrapped(
                reason,
                quote! { #data_struct_name },
                quote! {
                    let #data_struct_name(#(#names),*) = decoded;
                    ::std::result::Result::Ok(#enum_name::#variant_name(#(#names),*))
                },
            )
        }
        Fields::Named(fields) if mapping.include_data => {
            let data_struct_name = crate::data::data_struct_path(enum_name, variant_name);
            let names = named_field_idents(fields)?;
            deserialize_wrapped(
                reason,
                quote! { #data_struct_name },
                quote! {
                    let #data_struct_name { #(#names),* } = decoded;
                    ::std::result::Result::Ok(#enum_name::#variant_name { #(#names),* })
                },
            )
        }
        _ => quote! {
            ::std::result::Result::Err(::rpc_error::Error::MissingData {
                reason: #reason.to_owned(),
            })
        },
    };
    Ok(FromRpcArms {
        success: quote! {
            (::rpc_error::ErrorCode::#code, #reason) => { #body },
        },
        mismatch: quote! {
            (actual, #reason) => ::std::result::Result::Err(::rpc_error::Error::CodeMismatch {
                reason: #reason.to_owned(),
                expected: ::rpc_error::ErrorCode::#code,
                actual,
            }),
        },
    })
}

fn unit_variant_body(
    enum_name: &syn::Ident,
    variant_name: &syn::Ident,
    reason: &str,
) -> proc_macro2::TokenStream {
    quote! {
        match rpc_error.data().cloned() {
            Some(data) => ::std::result::Result::Err(::rpc_error::Error::UnexpectedData {
                reason: #reason.to_owned(),
                data,
            }),
            None => ::std::result::Result::Ok(#enum_name::#variant_name),
        }
    }
}

fn deserialize_single(
    enum_name: &syn::Ident,
    variant_name: &syn::Ident,
    reason: &str,
    ty: &Type,
) -> proc_macro2::TokenStream {
    deserialize_wrapped(
        reason,
        quote! { #ty },
        quote! {
            ::std::result::Result::Ok(#enum_name::#variant_name(decoded))
        },
    )
}

fn deserialize_wrapped(
    reason: &str,
    ty: proc_macro2::TokenStream,
    success: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        let Some(data) = rpc_error.data().cloned() else {
            return ::std::result::Result::Err(::rpc_error::Error::MissingData {
                reason: #reason.to_owned(),
            });
        };
        let decoded = match ::rpc_error::__private::serde_json::from_value::<#ty>(data.clone()) {
            ::std::result::Result::Ok(decoded) => decoded,
            ::std::result::Result::Err(source) => {
                return ::std::result::Result::Err(::rpc_error::Error::DataDeserialization {
                    reason: #reason.to_owned(),
                    data,
                    source,
                });
            }
        };
        #success
    }
}

fn fallback_arm(
    enum_name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>,
    attrs: &[Option<ErrorAttr>],
) -> syn::Result<proc_macro2::TokenStream> {
    let delegates = variants
        .iter()
        .zip(attrs.iter())
        .filter_map(|(variant, attr)| delegate_inner_type(variant, attr).transpose())
        .collect::<syn::Result<Vec<_>>>()?;
    if delegates.is_empty() {
        return Ok(quote! {
            (_, reason) => ::std::result::Result::Err(::rpc_error::Error::UnknownReason {
                reason: reason.to_owned(),
            }),
        });
    }
    let attempts = delegates.iter().map(|(variant_name, inner_type)| {
        quote! {
            match <#inner_type as ::std::convert::TryFrom<::rpc_error::RpcError>>::try_from(
                rpc_error.clone(),
            ) {
                ::std::result::Result::Ok(inner) => {
                    return ::std::result::Result::Ok(#enum_name::#variant_name(inner));
                }
                ::std::result::Result::Err(::rpc_error::Error::UnknownReason { .. }) => {}
                ::std::result::Result::Err(source) => {
                    return ::std::result::Result::Err(source);
                }
            }
        }
    });
    Ok(quote! {
        (_, reason) => {
            #(#attempts)*
            ::std::result::Result::Err(::rpc_error::Error::UnknownReason {
                reason: reason.to_owned(),
            })
        }
    })
}

fn delegate_inner_type<'a>(
    variant: &'a Variant,
    attr: &Option<ErrorAttr>,
) -> syn::Result<Option<(&'a syn::Ident, &'a Type)>> {
    if !matches!(attr, Some(ErrorAttr::Delegate)) {
        return Ok(None);
    }
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let Some(field) = fields.unnamed.first() else {
                return Err(syn::Error::new_spanned(variant, "expected one field"));
            };
            let ty = &field.ty;
            Ok(Some((&variant.ident, ty)))
        }
        _ => Err(syn::Error::new_spanned(
            variant,
            "delegate attribute requires one unnamed field",
        )),
    }
}
