use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{Data, DeriveInput, Fields, Ident, Type, Variant, parse_quote};

const MACRO_NAME: &str = "Value";

#[expect(clippy::too_many_lines)]
#[inline]
pub(crate) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let enum_data = match &input.data {
        Data::Enum(data) => data,
        Data::Struct(_) => {
            return Err(syn::Error::new(
                input.span(),
                format_args!("{MACRO_NAME} can only be derived for enums"),
            ));
        },
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.span(),
                format_args!("{MACRO_NAME} cannot be derived for unions"),
            ));
        },
    };

    let variants = enum_data
        .variants
        .iter()
        .map(|variant| validate_variant(variant))
        .collect::<syn::Result<Vec<_>>>()?;

    let enum_name = &input.ident;

    let field_ident = Ident::new("__inner", Span::call_site());
    let dest = Ident::new("__dest", Span::call_site());
    let ctx = Ident::new("__ctx", Span::call_site());

    let value_path: syn::Path = parse_quote!(::nixb::value::Value);

    // Generate match arms for `kind()` method
    let kind_arms = variants.iter().map(|(variant_name, _field_type)| {
        quote! {
            Self::#variant_name(#field_ident) => #value_path::kind(#field_ident),
        }
    });

    // Generate match arms for `write()` method
    let write_arms = variants.iter().map(|(variant_name, _field_type)| {
        quote! {
            Self::#variant_name(#field_ident) => {
                #value_path::write(#field_ident, #dest, #ctx)
            },
        }
    });

    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    // Build where clause with bounds for each variant type
    let where_predicates = variants.iter().map(|(_, field_type)| {
        quote! { #field_type: #value_path }
    });

    let existing_predicates = where_clause
        .map(|wc| {
            let preds = &wc.predicates;
            quote! { #preds, }
        })
        .unwrap_or_default();

    let extended_where_clause = quote! {
        where
            #existing_predicates
            #(#where_predicates),*
    };

    Ok(quote! {
        impl #impl_generics ::nixb::value::Value for #enum_name #ty_generics #extended_where_clause {
            #[inline]
            fn kind(&self) -> ::nixb::value::ValueKind {
                match self {
                    #(#kind_arms)*
                }
            }

            #[inline]
            fn write(
                self,
                #dest: ::nixb::value::UninitValue,
                #ctx: &mut ::nixb::context::Context,
            ) -> ::nixb::prelude::Result<()> {
                match self {
                    #(#write_arms)*
                }
            }
        }
    })
}

/// Validate that the variant is a tuple variant with exactly one field,
/// returning the variant name and field type if valid.
fn validate_variant(variant: &Variant) -> syn::Result<(&Ident, &Type)> {
    let variant_name = &variant.ident;

    match &variant.fields {
        Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
            let field = fields.unnamed.first().expect("checked length");
            Ok((variant_name, &field.ty))
        },
        Fields::Unnamed(_) => Err(syn::Error::new(
            variant.span(),
            format_args!(
                "{MACRO_NAME} requires all variants to have exactly one \
                 field, but `{variant_name}` has {} fields",
                variant.fields.len()
            ),
        )),
        Fields::Named(_) => Err(syn::Error::new(
            variant.span(),
            format_args!(
                "{MACRO_NAME} requires tuple variants, but `{variant_name}` \
                 has named fields"
            ),
        )),
        Fields::Unit => Err(syn::Error::new(
            variant.span(),
            format_args!(
                "{MACRO_NAME} requires tuple variants with one field, but \
                 `{variant_name}` is a unit variant"
            ),
        )),
    }
}
