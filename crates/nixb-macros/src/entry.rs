use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;
use syn::parse::Nothing;

#[inline]
pub(crate) fn expand(
    attr: proc_macro::TokenStream,
    entrypoint_fn: ItemFn,
) -> syn::Result<TokenStream> {
    syn::parse::<Nothing>(attr)?;

    let entrypoint_fn_name = &entrypoint_fn.sig.ident;

    Ok(quote! {
        #entrypoint_fn

        #[unsafe(no_mangle)]
        unsafe extern "C" fn nix_plugin_entry() {
            ::nixb::entry(#entrypoint_fn_name);
        }
    })
}
