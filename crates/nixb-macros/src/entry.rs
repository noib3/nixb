use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Nothing;
use syn::{ItemFn, parse_quote};

#[inline]
pub(crate) fn expand(
    attr: proc_macro::TokenStream,
    entrypoint_fn: ItemFn,
) -> syn::Result<TokenStream> {
    syn::parse::<Nothing>(attr)?;

    let nix_bindings: syn::Path = parse_quote!(::nix_bindings);
    let entrypoint_fn_name = &entrypoint_fn.sig.ident;

    Ok(quote! {
        #entrypoint_fn

        #[unsafe(no_mangle)]
        unsafe extern "C" fn nix_plugin_entry() {
            #nix_bindings::entry(#entrypoint_fn_name);
        }
    })
}
