use std::ffi::CString;

use proc_macro2::{Literal, Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::ext::IdentExt;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Token, braced, parse_quote};

use crate::list::Value;

#[expect(clippy::too_many_lines)]
#[inline]
pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let Attrset { pairs, num_expr_keys } = syn::parse2(input)?;

    let all_keys_are_literals = num_expr_keys == 0;

    let mut keys = TokenStream::new();
    let mut values = TokenStream::new();
    let mut num_non_optional = 0;

    let comma = <Token![,]>::default();

    for (idx, pair) in pairs.iter().enumerate() {
        // Add the pair's attributes to both keys and values.
        for attr in &pair.attrs {
            attr.to_tokens(&mut keys);
            attr.to_tokens(&mut values);
        }

        pair.key.to_tokens(&mut keys);

        // Wrap optional values in MightSkip.
        if pair.is_optional {
            let value_var = format_ident!("__value_{idx}");
            let skip_var = format_ident!("__should_skip_{idx}");
            quote! {
                ::nixb::expr::attrset::skips::MightSkip::new(#value_var, #skip_var)
            }
            .to_tokens(&mut values);
        } else {
            num_non_optional += 1;
            pair.value.to_tokens(&mut values);
        }

        // Add a comma if this is not the last pair or if there's only one
        // pair.
        if idx + 1 < pairs.len() || pairs.len() == 1 {
            comma.to_tokens(&mut keys);
            comma.to_tokens(&mut values);
        }
    }

    if num_non_optional < pairs.len() {
        let num_non_optional = Literal::usize_unsuffixed(num_non_optional);

        let optional_var_declarations = pairs.iter().enumerate().filter_map(|(idx, pair)| {
            if pair.is_optional {
                let value_var = format_ident!("__value_{idx}");
                let skip_var = format_ident!("__should_skip_{idx}");
                let value = &pair.value;
                Some(quote! {
                    let #value_var = #value;
                    let #skip_var = ::core::option::Option::is_none(&#value_var);
                })
            } else {
                None
            }
        });

        let plus_one_if_not_skipped =
            pairs.iter().enumerate().filter_map(|(idx, pair)| {
                if pair.is_optional {
                    let skip_var = format_ident!("__should_skip_{idx}");
                    Some(quote! { + (!#skip_var as ::core::ffi::c_uint) })
                } else {
                    None
                }
            });

        Ok(quote! {{
            #(#optional_var_declarations)*
            ::nixb::expr::attrset::StaticAttrsetWithSkips::<#all_keys_are_literals, _, _>::new(
                (#keys),
                (#values),
                #num_non_optional #(#plus_one_if_not_skipped)*,
            )
        }})
    } else {
        Ok(quote! {
            ::nixb::expr::attrset::StaticAttrset::<#all_keys_are_literals, _, _>::new(
                (#keys),
                (#values)
            )
        })
    }
}

struct Attrset {
    /// The parsed key-value pairs, sorted lexicographically by key.
    /// [`Expr`](Key::Expr)ession keys are not sorted since they can't be
    /// compared at compile time, and they're all stored at the start of the
    /// list.
    pairs: Vec<KeyValuePair>,

    /// The number of [`Expr`](Key::Expr)ession keys in `pairs`.
    num_expr_keys: usize,
}

struct KeyValuePair {
    attrs: Vec<Attribute>,
    is_cfg_gated: Option<bool>,
    is_optional: bool,
    key: Key,
    value: Value,
}

enum Key {
    Literal(CString),
    Expr(syn::Expr),
}

impl Parse for Attrset {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut pairs = Vec::new();
        let mut num_expr_keys = 0;

        while !input.is_empty() {
            // Parse attributes (e.g., #[cfg(...)]).
            let attrs = input.call(Attribute::parse_outer)?;

            // Try to get the key ident to support shorthand syntax.
            let key_ident = if !input.peek(syn::token::Brace) {
                Some(input.fork().call(syn::Ident::parse_any)?)
            } else {
                None
            };

            // Parse key.
            let key = input.parse()?;

            // Parse optional `?` to mark this key as optional.
            let is_optional = input.peek(Token![?]);
            if is_optional {
                input.parse::<Token![?]>()?;
            }

            let value = if input.peek(Token![:]) {
                input.parse::<Token![:]>()?;
                input.parse()?
            } else if let Some(ident) = key_ident {
                // Shorthand syntax: use key ident as value.
                Value::Expr(parse_quote! { #ident })
            } else {
                return Err(syn::Error::new(
                    input.span(),
                    "expected `:` after attribute set key",
                ));
            };

            let is_cfg_gated =
                if attrs.is_empty() { Some(false) } else { None };

            let pair =
                KeyValuePair { attrs, is_cfg_gated, is_optional, key, value };

            if matches!(pair.key, Key::Expr(_)) {
                pairs.insert(num_expr_keys, pair);
                num_expr_keys += 1;
            } else {
                // a) get insertion index by binary searching the vec.
                // b) if there's a duplicate, resolve both this keypair and the
                // existing keypair's `is_cfg_gated`. If both of them are true
                // we allow it (Q: how do we break Ordering ties in that case?).
                // c) otherwise, combine the error with this key's span.
                todo!();
            }

            // Parse optional comma.
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { pairs, num_expr_keys })
    }
}

// fn validate_no_duplicate_keys(pairs: &[KeyValuePair]) -> syn::Result<()> {
//     let mut first_occurrences = HashMap::new();
//     let mut errors: Option<syn::Error> = None;
//
//     for pair in pairs {
//         let Key::Literal(key) = &pair.key else {
//             continue;
//         };
//
//         // Pairs annotated with `#[cfg]` and `#[cfg_attr]` can be omitted when
//         // the gate is not active, so skip duplicate detection for those cases.
//         if pair.attrs.iter().any(|attr| {
//             attr.path().is_ident("cfg") || attr.path().is_ident("cfg_attr")
//         }) {
//             continue;
//         }
//
//         match first_occurrences.entry(key.name.clone()) {
//             Entry::Vacant(entry) => {
//                 entry.insert(key.span);
//             },
//             Entry::Occupied(_) => {
//                 let error = syn::Error::new(
//                     key.span,
//                     format_args!("duplicate attrset key `{}`", key.name),
//                 );
//
//                 if let Some(existing_errors) = &mut errors {
//                     existing_errors.combine(error);
//                 } else {
//                     errors = Some(error);
//                 }
//             },
//         }
//     }
//
//     errors.map_or(Ok(()), Err)
// }

impl Parse for Key {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // If the key is wrapped in braces, parse it as an expression.
        if input.peek(syn::token::Brace) {
            let content;
            braced!(content in input);
            let expr: syn::Expr = content.parse()?;
            Ok(Self::Expr(expr))
        }
        // Otherwise, parse it as an ident (including keywords) and convert to
        // c-string literal.
        else {
            let ident = input.call(syn::Ident::parse_any)?;
            let ident_str = ident.to_string();
            let c_string = CString::new(ident_str).map_err(|_| {
                syn::Error::new(
                    ident.span(),
                    "attrset key cannot contain NUL byte",
                )
            })?;
            Ok(Self::Literal(c_string))
        }
    }
}

impl ToTokens for Key {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Literal(c_string) => {
                let literal = Literal::c_string(c_string);
                tokens.extend(quote! {
                    // SAFETY: valid UTF-8.
                    unsafe { ::nixb::expr::Utf8CStr::new_unchecked(#literal) }
                })
            },
            Self::Expr(expr) => tokens.extend(quote! { { #expr } }),
        }
    }
}

#[cfg(test)]
mod tests {
    use quote::quote;

    use super::expand;

    #[test]
    fn rejects_each_duplicate_literal_key_after_the_first() {
        let error = expand(quote! {
            value1: "Hello",
            value1: "World",
            value1: "!",
        })
        .unwrap_err();

        let compile_error = error.into_compile_error().to_string();

        assert_eq!(compile_error.matches("compile_error").count(), 2);
        assert_eq!(
            compile_error.matches("duplicate attrset key `value1`").count(),
            2
        );
    }

    #[test]
    fn skips_duplicate_detection_for_cfg_guarded_keys() {
        assert!(
            expand(quote! {
                #[cfg(feature = "foo")]
                value1: "Hello",
                value1: "World",
            })
            .is_ok()
        );
    }
}
