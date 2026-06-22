//! Data structure generation for multi-field variants.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Attribute, Field, Fields, Variant, punctuated::Punctuated, token::Comma};

use crate::attrs::ErrorAttr;

pub fn generate_data_structs(
    enum_name: &syn::Ident,
    variants: &Punctuated<Variant, Comma>,
    attrs: &[Option<ErrorAttr>],
) -> syn::Result<proc_macro2::TokenStream> {
    let structs = variants
        .iter()
        .zip(attrs.iter())
        .filter_map(|(variant, attr)| data_struct(variant, attr))
        .collect::<syn::Result<Vec<_>>>()?;
    if structs.is_empty() {
        return Ok(quote! {});
    }
    let module_name = data_module_name(enum_name);
    Ok(quote! {
        #[doc(hidden)]
        #[allow(non_snake_case)]
        mod #module_name {
            #[allow(unused_imports)]
            use super::*;

            #(#structs)*
        }
    })
}

pub fn data_struct_path(enum_name: &syn::Ident, variant_name: &syn::Ident) -> TokenStream {
    let module_name = data_module_name(enum_name);
    let data_struct_name = data_struct_name(variant_name);
    quote! { #module_name::#data_struct_name }
}

fn data_module_name(enum_name: &syn::Ident) -> syn::Ident {
    format_ident!("__rpc_error_data_for_{}", enum_name)
}

fn data_struct_name(variant_name: &syn::Ident) -> syn::Ident {
    format_ident!("{}Data", variant_name)
}

fn data_struct(
    variant: &Variant,
    attr: &Option<ErrorAttr>,
) -> Option<syn::Result<proc_macro2::TokenStream>> {
    let include_data = matches!(attr, Some(ErrorAttr::Mapping(mapping)) if mapping.include_data);
    if !include_data {
        return None;
    }
    let variant_name = &variant.ident;
    let data_struct_name = data_struct_name(variant_name);
    match &variant.fields {
        Fields::Named(fields) => Some(named_data_struct(variant_name, &data_struct_name, fields)),
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let field = fields.unnamed.first()?;
            if has_serde_attrs(field) {
                return Some(Err(syn::Error::new_spanned(
                    field,
                    "serde field attributes are not supported on single-field data variants",
                )));
            }
            None
        }
        Fields::Unnamed(fields) if fields.unnamed.len() > 1 => {
            let field_params = (0..fields.unnamed.len())
                .map(|index| format_ident!("RpcErrorField{index}"))
                .collect::<Vec<_>>();
            let field_defaults = fields
                .unnamed
                .iter()
                .zip(field_params.iter())
                .map(field_default);
            let definitions = fields
                .unnamed
                .iter()
                .zip(field_params.iter())
                .map(tuple_field_definition);
            let doc = format!("Data structure for {variant_name} error variant.");
            Some(Ok(quote! {
                #[doc = #doc]
                #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
                pub(super) struct #data_struct_name<#(#field_defaults),*>(
                    #(#definitions),*
                );
            }))
        }
        _ => None,
    }
}

fn named_data_struct(
    variant_name: &syn::Ident,
    data_struct_name: &syn::Ident,
    fields: &syn::FieldsNamed,
) -> syn::Result<proc_macro2::TokenStream> {
    let definitions = fields
        .named
        .iter()
        .enumerate()
        .map(|field| {
            let (index, field) = field;
            let field_name = field
                .ident
                .as_ref()
                .ok_or_else(|| syn::Error::new_spanned(field, "expected named field"))?;
            let field_param = format_ident!("RpcErrorField{index}");
            let docs = field
                .attrs
                .iter()
                .filter(|attr| attr.path().is_ident("doc"));
            let serde_attrs = serde_attrs(field);
            Ok(quote! {
                #(#docs)*
                #(#serde_attrs)*
                pub(super) #field_name: #field_param
            })
        })
        .collect::<syn::Result<Vec<_>>>()?;
    let field_defaults = fields.named.iter().enumerate().map(|(index, field)| {
        let field_param = format_ident!("RpcErrorField{index}");
        field_default((field, &field_param))
    });
    let doc = format!("Data structure for {variant_name} error variant.");
    Ok(quote! {
        #[doc = #doc]
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub(super) struct #data_struct_name<#(#field_defaults),*> {
            #(#definitions),*
        }
    })
}

fn tuple_field_definition((field, field_param): (&Field, &syn::Ident)) -> TokenStream {
    let serde_attrs = serde_attrs(field);
    quote! {
        #(#serde_attrs)*
        pub(super) #field_param
    }
}

fn field_default((field, field_param): (&Field, &syn::Ident)) -> TokenStream {
    let field_type = &field.ty;
    quote! { #field_param = #field_type }
}

fn has_serde_attrs(field: &Field) -> bool {
    field.attrs.iter().any(|attr| attr.path().is_ident("serde"))
}

fn serde_attrs(field: &Field) -> impl Iterator<Item = &Attribute> {
    field
        .attrs
        .iter()
        .filter(|attr| attr.path().is_ident("serde"))
}
