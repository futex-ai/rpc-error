//! Derive orchestration.

use std::collections::BTreeSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Type, Variant, parse_macro_input};

use crate::attrs::{ErrorAttr, find_error_attr};

pub fn derive_rpc_error(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let enum_name = &input.ident;
    let Data::Enum(data) = &input.data else {
        return Err(syn::Error::new_spanned(
            input,
            "RpcError can only be derived for enums",
        ));
    };
    let attrs = data
        .variants
        .iter()
        .map(find_error_attr)
        .collect::<syn::Result<Vec<_>>>()?;
    reject_duplicate_reasons(&attrs)?;
    let data_structs = crate::data::generate_data_structs(enum_name, &data.variants, &attrs)?;
    let to_rpc = crate::to_rpc::generate_to_rpc(enum_name, &data.variants, &attrs)?;
    let from_rpc = crate::from_rpc::generate_from_rpc(enum_name, &data.variants, &attrs)?;
    let catalog = generate_catalog(enum_name, &data.variants, &attrs)?;
    Ok(quote! {
        #data_structs
        #to_rpc
        #from_rpc
        #catalog
    })
}

fn reject_duplicate_reasons(attrs: &[Option<ErrorAttr>]) -> syn::Result<()> {
    let mut seen = BTreeSet::new();
    for attr in attrs {
        let Some(ErrorAttr::Mapping(mapping)) = attr else {
            continue;
        };
        if !seen.insert(mapping.reason.clone()) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!("duplicate rpc_error reason `{}`", mapping.reason),
            ));
        }
    }
    Ok(())
}

fn generate_catalog(
    enum_name: &syn::Ident,
    variants: &syn::punctuated::Punctuated<Variant, syn::token::Comma>,
    attrs: &[Option<ErrorAttr>],
) -> syn::Result<proc_macro2::TokenStream> {
    let definitions = attrs.iter().filter_map(|attr| {
        let Some(ErrorAttr::Mapping(mapping)) = attr else {
            return None;
        };
        let code = &mapping.code;
        let reason = &mapping.reason;
        Some(quote! {
            ::rpc_error::ReasonDefinition {
                code: ::rpc_error::ErrorCode::#code,
                reason: #reason,
            }
        })
    });
    let delegates = variants
        .iter()
        .zip(attrs.iter())
        .filter_map(|(variant, attr)| delegate_inner_type(variant, attr).transpose())
        .collect::<syn::Result<Vec<_>>>()?;
    let delegated_catalogs = delegates.iter().map(|delegate| {
        quote! {
            catalogs.extend(<#delegate as ::rpc_error::ReasonCatalogs>::reason_catalogs());
        }
    });
    Ok(quote! {
        impl ::rpc_error::ReasonCatalog for #enum_name {
            fn reason_definitions() -> &'static [::rpc_error::ReasonDefinition] {
                const DEFINITIONS: &[::rpc_error::ReasonDefinition] = &[
                    #(#definitions),*
                ];
                DEFINITIONS
            }
        }

        impl ::rpc_error::ReasonCatalogs for #enum_name {
            fn reason_catalogs() -> ::std::vec::Vec<&'static [::rpc_error::ReasonDefinition]> {
                let mut catalogs = ::std::vec![
                    <#enum_name as ::rpc_error::ReasonCatalog>::reason_definitions()
                ];
                #(#delegated_catalogs)*
                catalogs
            }
        }

        impl #enum_name {
            /// Returns direct and delegated reason-code catalogs for this error contract.
            pub fn reason_catalogs() -> ::std::vec::Vec<&'static [::rpc_error::ReasonDefinition]> {
                <Self as ::rpc_error::ReasonCatalogs>::reason_catalogs()
            }
        }
    })
}

fn delegate_inner_type<'a>(
    variant: &'a Variant,
    attr: &Option<ErrorAttr>,
) -> syn::Result<Option<&'a Type>> {
    if !matches!(attr, Some(ErrorAttr::Delegate)) {
        return Ok(None);
    }
    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let Some(field) = fields.unnamed.first() else {
                return Err(syn::Error::new_spanned(variant, "expected one field"));
            };
            Ok(Some(&field.ty))
        }
        _ => Err(syn::Error::new_spanned(
            variant,
            "delegate attribute requires one unnamed field",
        )),
    }
}
