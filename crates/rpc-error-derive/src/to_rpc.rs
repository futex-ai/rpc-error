//! Typed error to `RpcError` generation.

use quote::{format_ident, quote};
use syn::{Fields, Variant, punctuated::Punctuated, token::Comma};

use crate::attrs::{ErrorAttr, ErrorMapping};
use crate::util::named_field_idents;

pub fn generate_to_rpc(
    enum_name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>,
    attrs: &[Option<ErrorAttr>],
) -> syn::Result<proc_macro2::TokenStream> {
    let arms = variants
        .iter()
        .zip(attrs.iter())
        .map(|(variant, attr)| to_rpc_arm(enum_name, variant, attr))
        .collect::<syn::Result<Vec<_>>>()?;
    Ok(quote! {
        impl ::std::convert::From<#enum_name> for ::rpc_error::RpcError {
            fn from(err: #enum_name) -> Self {
                let message = ::std::string::ToString::to_string(&err);
                match err {
                    #(#arms)*
                }
            }
        }
    })
}

fn to_rpc_arm(
    enum_name: &syn::Ident,
    variant: &Variant,
    attr: &Option<ErrorAttr>,
) -> syn::Result<proc_macro2::TokenStream> {
    let variant_name = &variant.ident;
    match attr {
        Some(ErrorAttr::Mapping(mapping)) => {
            mapping_arm(enum_name, variant_name, &variant.fields, mapping)
        }
        Some(ErrorAttr::Delegate) => delegate_arm(enum_name, variant),
        None => Ok(internal_arm(enum_name, variant_name, &variant.fields)),
    }
}

fn mapping_arm(
    enum_name: &syn::Ident,
    variant_name: &syn::Ident,
    fields: &Fields,
    mapping: &ErrorMapping,
) -> syn::Result<proc_macro2::TokenStream> {
    let pattern = pattern(enum_name, variant_name, fields)?;
    let base = base_error(mapping);
    let source = source_expr(enum_name, variant_name, fields);
    let data = data_tokens(enum_name, variant_name, fields, mapping, source)?;
    Ok(quote! {
        #pattern => {
            #base
            #data
        }
    })
}

fn base_error(mapping: &ErrorMapping) -> proc_macro2::TokenStream {
    let code = &mapping.code;
    let reason = &mapping.reason;
    let message = match &mapping.message {
        Some(message) => quote! { #message },
        None if mapping.is_internal => quote! { "Internal server error" },
        None => quote! { message.as_str() },
    };
    let severity = mapping.severity.as_ref().map(|severity| {
        quote! {
            rpc_error = rpc_error.with_severity(#severity);
        }
    });
    quote! {
        let mut rpc_error = match ::rpc_error::RpcError::new(
            ::rpc_error::ErrorCode::#code,
            #reason,
            #message,
        ) {
            ::std::result::Result::Ok(error) => error,
            ::std::result::Result::Err(source) => return ::rpc_error::RpcError::internal(source),
        };
        #severity
    }
}

fn data_tokens(
    enum_name: &syn::Ident,
    variant_name: &syn::Ident,
    fields: &Fields,
    mapping: &ErrorMapping,
    source: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    if !mapping.include_data {
        return Ok(quote! { rpc_error.with_source(#source) });
    }
    match fields {
        Fields::Unit => Ok(quote! { rpc_error.with_source(#source) }),
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            Ok(attach_data_or_internal(quote! { &data }, source))
        }
        Fields::Unnamed(fields) => {
            let data_struct_name = crate::data::data_struct_path(enum_name, variant_name);
            let names = (0..fields.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect::<Vec<_>>();
            Ok(attach_data_or_internal(
                quote! {
                    #data_struct_name(#(&#names),*)
                },
                source,
            ))
        }
        Fields::Named(fields) => {
            let data_struct_name = crate::data::data_struct_path(enum_name, variant_name);
            let names = named_field_idents(fields)?;
            Ok(attach_data_or_internal(
                quote! {
                    #data_struct_name {
                        #(#names: &#names),*
                    }
                },
                source,
            ))
        }
    }
}

fn attach_data_or_internal(
    data: proc_macro2::TokenStream,
    source: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        let rpc_error = match rpc_error.with_data(#data) {
            ::std::result::Result::Ok(error) => error,
            ::std::result::Result::Err(::rpc_error::Error::DataSerialization { source }) => {
                return ::rpc_error::RpcError::internal(source);
            }
            ::std::result::Result::Err(source) => {
                return ::rpc_error::RpcError::internal(source);
            }
        };
        rpc_error.with_source(#source)
    }
}

fn pattern(
    enum_name: &syn::Ident,
    variant_name: &syn::Ident,
    fields: &Fields,
) -> syn::Result<proc_macro2::TokenStream> {
    match fields {
        Fields::Unit => Ok(quote! { #enum_name::#variant_name }),
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            Ok(quote! { #enum_name::#variant_name(data) })
        }
        Fields::Unnamed(fields) => {
            let names = (0..fields.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect::<Vec<_>>();
            Ok(quote! { #enum_name::#variant_name(#(#names),*) })
        }
        Fields::Named(fields) => {
            let names = named_field_idents(fields)?;
            Ok(quote! { #enum_name::#variant_name { #(#names),* } })
        }
    }
}

fn source_expr(
    enum_name: &syn::Ident,
    variant_name: &syn::Ident,
    fields: &Fields,
) -> proc_macro2::TokenStream {
    match fields {
        Fields::Unit => quote! { #enum_name::#variant_name },
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            quote! { #enum_name::#variant_name(data) }
        }
        Fields::Unnamed(fields) => {
            let names = (0..fields.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect::<Vec<_>>();
            quote! { #enum_name::#variant_name(#(#names),*) }
        }
        Fields::Named(fields) => {
            let names = match named_field_idents(fields) {
                Ok(names) => names,
                Err(_source) => return quote! { #enum_name::#variant_name },
            };
            quote! { #enum_name::#variant_name { #(#names),* } }
        }
    }
}

fn delegate_arm(
    enum_name: &syn::Ident,
    variant: &Variant,
) -> syn::Result<proc_macro2::TokenStream> {
    let variant_name = &variant.ident;
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => Ok(quote! {
            #enum_name::#variant_name(inner) => inner.into(),
        }),
        _ => Err(syn::Error::new_spanned(
            variant,
            "delegate attribute requires one unnamed field",
        )),
    }
}

fn internal_arm(
    enum_name: &syn::Ident,
    variant_name: &syn::Ident,
    fields: &Fields,
) -> proc_macro2::TokenStream {
    match fields {
        Fields::Unit => quote! {
            #enum_name::#variant_name => ::rpc_error::RpcError::internal(#enum_name::#variant_name),
        },
        Fields::Unnamed(fields) => {
            let names = (0..fields.unnamed.len())
                .map(|index| format_ident!("field_{index}"))
                .collect::<Vec<_>>();
            quote! {
                #enum_name::#variant_name(#(#names),*) => {
                    ::rpc_error::RpcError::internal(#enum_name::#variant_name(#(#names),*))
                },
            }
        }
        Fields::Named(fields) => {
            let names = match named_field_idents(fields) {
                Ok(names) => names,
                Err(_source) => return quote! {},
            };
            quote! {
                #enum_name::#variant_name { #(#names),* } => {
                    ::rpc_error::RpcError::internal(#enum_name::#variant_name { #(#names),* })
                },
            }
        }
    }
}
