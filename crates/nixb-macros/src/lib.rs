//! TODO: docs.

mod attrset;
mod attrset_derive;
mod list;
mod plugin;
mod primop;
mod try_from_value;
mod value;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// TODO: docs
#[proc_macro]
pub fn attrset(input: TokenStream) -> TokenStream {
    attrset::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// TODO: docs
#[proc_macro_derive(Attrset, attributes(attrset))]
pub fn attrset_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    attrset_derive::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// TODO: docs
#[proc_macro]
pub fn list(input: TokenStream) -> TokenStream {
    list::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Marks the entrypoint function of a Nix plugin.
#[proc_macro_attribute]
pub fn plugin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let item = parse_macro_input!(item as syn::ItemFn);
    plugin::expand(attr, item)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// TODO: docs
#[proc_macro_derive(PrimOp)]
pub fn primop(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    primop::expand(input).unwrap_or_else(syn::Error::into_compile_error).into()
}

/// TODO: docs
#[proc_macro_derive(TryFromValue, attributes(try_from))]
pub fn try_from_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    try_from_value::expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// TODO: docs
#[proc_macro_derive(Value)]
pub fn value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    value::expand(input).unwrap_or_else(syn::Error::into_compile_error).into()
}
