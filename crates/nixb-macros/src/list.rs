use std::ffi::CString;

use proc_macro2::{Literal, TokenStream};
use quote::{ToTokens, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::Comma;

#[inline]
pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream> {
    let List { values } = syn::parse2(input)?;

    Ok(quote! {
        ::nix_bindings::list::LiteralList::new((#values))
    })
}

struct List {
    values: Punctuated<Value, Comma>,
}

pub(crate) enum Value {
    StringLiteral(proc_macro2::Literal),
    Expr(syn::Expr),
}

impl Parse for List {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut values = Punctuated::new();

        while !input.is_empty() {
            let value = input.parse()?;
            values.push(value);

            if input.peek(Comma) {
                let comma = input.parse()?;
                values.push_punct(comma);
            }
        }

        Ok(Self { values })
    }
}

impl Parse for Value {
    #[inline]
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let expr: syn::Expr = input.parse()?;

        let syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(lit_str), ..
        }) = &expr
        else {
            return Ok(Self::Expr(expr));
        };

        // If the value is a Rust string literal, convert it to a C string
        // literal to avoid having to allocate at runtime.
        let string_content = lit_str.value();
        let c_string = CString::new(string_content).map_err(|_| {
            syn::Error::new(
                lit_str.span(),
                "string literal cannot contain NUL byte",
            )
        })?;

        Ok(Self::StringLiteral(Literal::c_string(&c_string)))
    }
}

impl ToTokens for Value {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::StringLiteral(lit) => lit.to_tokens(tokens),
            Self::Expr(expr) => expr.to_tokens(tokens),
        }
    }
}
