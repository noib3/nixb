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

#[derive(Default)]
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
    Literal(CString, Span),
    Expr(syn::Expr),
}

impl Attrset {
    fn insert_pair(&mut self, mut pair: KeyValuePair) -> syn::Result<()> {
        if matches!(pair.key, Key::Expr(_)) {
            self.pairs.insert(self.num_expr_keys, pair);
            self.num_expr_keys += 1;
        } else {
            let idx = self.literal_pair_insertion_idx(&mut pair)?;
            self.pairs.insert(idx, pair);
        }

        Ok(())
    }

    /// Returns the index at which the given literal pair should be inserted, or
    /// an error if the attrset already contains a non-`cfg`-gated pair with the
    /// same key.
    fn literal_pair_insertion_idx(
        &mut self,
        pair: &mut KeyValuePair,
    ) -> syn::Result<usize> {
        let Key::Literal(key, _) = &pair.key else {
            unreachable!("pair's key must be a literal")
        };

        let start_idx =
            self.pairs[self.num_expr_keys..].partition_point(|existing| {
                let Key::Literal(existing_key, _) = &existing.key else {
                    unreachable!("literal keys are stored after expr keys");
                };
                existing_key < key
            }) + self.num_expr_keys;

        let num_pairs_with_same_key = self.pairs[start_idx..]
            .iter()
            .take_while(|existing| {
                let Key::Literal(existing_key, _) = &existing.key else {
                    unreachable!("literal keys are stored after expr keys");
                };
                existing_key == key
            })
            .count();

        let insertion_index = start_idx + num_pairs_with_same_key;

        if num_pairs_with_same_key != 0 {
            let is_cfg_gated = pair.is_cfg_gated();

            let any_existing_is_not_cfg_gated = self.pairs
                [start_idx..insertion_index]
                .iter_mut()
                .any(|pair| !pair.is_cfg_gated());

            // Only emit an error if either the new pair or any of the existing
            // pairs with the same key is not `cfg`-gated. Otherwise allow the
            // duplicate since the gates might be mutually exclusive.
            if !is_cfg_gated || any_existing_is_not_cfg_gated {
                let Key::Literal(key, span) = &pair.key else { unreachable!() };

                return Err(syn::Error::new(
                    *span,
                    format_args!(
                        "duplicate attrset key `{}`",
                        key.as_c_str().to_string_lossy()
                    ),
                ));
            }
        }

        Ok(insertion_index)
    }
}

impl KeyValuePair {
    fn is_cfg_gated(&mut self) -> bool {
        if let Some(is_cfg_gated) = self.is_cfg_gated {
            return is_cfg_gated;
        }

        let is_cfg_gated =
            self.attrs.iter().any(|attr| attr.path().is_ident("cfg"));

        self.is_cfg_gated = Some(is_cfg_gated);

        is_cfg_gated
    }
}

impl Parse for Attrset {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut this = Self::default();
        let mut errors: Option<syn::Error> = None;

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

            let insert_res = this.insert_pair(KeyValuePair {
                attrs,
                is_cfg_gated,
                is_optional,
                key,
                value,
            });

            if let Err(err) = insert_res {
                if let Some(existing_errors) = errors.as_mut() {
                    existing_errors.combine(err);
                } else {
                    errors = Some(err);
                }
            }

            // Parse optional comma.
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        if let Some(errors) = errors { Err(errors) } else { Ok(this) }
    }
}

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
            Ok(Self::Literal(c_string, ident.span()))
        }
    }
}

impl ToTokens for Key {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::Literal(c_string, _) => {
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
    fn allows_duplicate_cfg_guarded_keys() {
        assert!(
            expand(quote! {
                #[cfg(feature = "foo")]
                value1: "Hello",
                #[cfg(feature = "bar")]
                value1: "World",
            })
            .is_ok()
        );
    }

    #[test]
    fn rejects_mixed_cfg_guarded_and_unguarded_duplicates() {
        let error = expand(quote! {
            #[cfg(feature = "foo")]
            value1: "Hello",
            value1: "World",
        })
        .unwrap_err();

        let compile_error = error.into_compile_error().to_string();

        assert_eq!(
            compile_error.matches("duplicate attrset key `value1`").count(),
            1
        );
    }
}
