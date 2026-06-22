//! Shared derive helpers.

use syn::{FieldsNamed, Ident};

pub fn named_field_idents(fields: &FieldsNamed) -> syn::Result<Vec<&Ident>> {
    fields
        .named
        .iter()
        .map(|field| {
            field
                .ident
                .as_ref()
                .ok_or_else(|| syn::Error::new_spanned(field, "expected named field"))
        })
        .collect()
}
