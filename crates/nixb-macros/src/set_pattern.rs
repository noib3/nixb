use std::collections::HashSet;
use std::ffi::CString;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::{ToTokens, quote};
use syn::meta::ParseNestedMeta;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{
    Attribute,
    Data,
    DeriveInput,
    Expr,
    Fields,
    FieldsNamed,
    LifetimeParam,
    parse_quote,
};

const MACRO_NAME: &str = "SetPattern";

#[inline]
pub(crate) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let attrs = Attributes::parse(&input.attrs, AttributePosition::Struct)?;
    let fields = named_fields(&input)?;

    let attrset = Ident::new("__attrset", Span::call_site());
    let value = Ident::new("__value", Span::call_site());
    let ctx = Ident::new("__ctx", Span::call_site());
    let lifetime: LifetimeParam = parse_quote!('a);

    let try_from_attrset_impl =
        try_from_attrset_impl(&attrs, fields, &attrset, &ctx)?;
    let lifetime_generic = lifetime_generic(&input, &lifetime)?;
    let struct_name = &input.ident;

    Ok(quote! {
        impl<#lifetime> ::nixb::expr::value::TryFromValue<::nixb::expr::value::NixValue<::nixb::expr::value::Borrowed<#lifetime>>> for #struct_name #lifetime_generic {
            #[inline]
            fn try_from_value(
                #value: ::nixb::expr::value::NixValue<::nixb::expr::value::Borrowed<#lifetime>>,
                #ctx: &mut ::nixb::expr::context::Context,
            ) -> ::nixb::Result<Self> {
                let #attrset = ::nixb::expr::attrset::NixAttrset::<_>::try_from_value(
                    #value, #ctx,
                )?;
                <Self as ::nixb::expr::value::TryFromValue<_>>::try_from_value(
                    #attrset,
                    #ctx,
                )
            }
        }

        impl<#lifetime> ::nixb::expr::value::TryFromValue<::nixb::expr::attrset::NixAttrset<::nixb::expr::value::Borrowed<#lifetime>>> for #struct_name #lifetime_generic {
            #[inline]
            fn try_from_value(
                #attrset: ::nixb::expr::attrset::NixAttrset<::nixb::expr::value::Borrowed<#lifetime>>,
                #ctx: &mut ::nixb::expr::context::Context,
            ) -> ::nixb::Result<Self> {
                #try_from_attrset_impl
            }
        }
    })
}

fn lifetime_generic(
    input: &DeriveInput,
    lifetime: &LifetimeParam,
) -> syn::Result<impl ToTokens> {
    match input.generics.params.iter().fold(
        (0, 0),
        |(num_total, num_lifetimes), r#gen| {
            let is_lifetime = matches!(r#gen, syn::GenericParam::Lifetime(_));
            (num_total + 1, num_lifetimes + (is_lifetime as usize))
        },
    ) {
        (0, 0) => Ok(None),
        (1, 1) => Ok(Some(quote! { <#lifetime> })),
        _ => Err(syn::Error::new(
            input.generics.span(),
            "Args can only have zero or one lifetime generic parameter",
        )),
    }
}

fn named_fields(input: &DeriveInput) -> syn::Result<&FieldsNamed> {
    let r#struct = match &input.data {
        Data::Struct(str) => str,
        Data::Enum(_) => {
            return Err(syn::Error::new(
                input.span(),
                format_args!("{MACRO_NAME} cannot be derived for enums"),
            ));
        },
        Data::Union(_) => {
            return Err(syn::Error::new(
                input.span(),
                format_args!("{MACRO_NAME} cannot be derived for unions"),
            ));
        },
    };

    match &r#struct.fields {
        Fields::Named(fields) => match fields.named.len() {
            0 => Err(syn::Error::new(
                input.span(),
                "struct must have at least one field",
            )),

            _ => Ok(fields),
        },
        Fields::Unit | Fields::Unnamed(_) => Err(syn::Error::new(
            input.span(),
            format_args!(
                "{MACRO_NAME} can only be derived for structs with named \
                 fields"
            ),
        )),
    }
}

#[expect(clippy::too_many_lines)]
fn try_from_attrset_impl(
    struct_attrs: &Attributes,
    fields: &FieldsNamed,
    attrset: &Ident,
    ctx: &Ident,
) -> syn::Result<impl ToTokens> {
    let mut field_names = Punctuated::<_, Comma>::new();
    let mut field_initializers = TokenStream::new();
    let mut formal_match_arms = TokenStream::new();
    let mut formal_names = HashSet::new();
    let count_match =
        (!struct_attrs.ellipsis).then(|| quote! { __num_matched += 1; });

    for field in fields.named.iter() {
        let field_attrs =
            Attributes::parse(&field.attrs, AttributePosition::Field)?;

        let field_name = field.ident.as_ref().expect("fields are named");

        let mut key_name_str = field_name.to_string();

        if let Some(rename) =
            field_attrs.rename.as_ref().or(struct_attrs.rename.as_ref())
        {
            rename.clone().apply(&mut key_name_str);
        }

        if !formal_names.insert(key_name_str.clone()) {
            return Err(syn::Error::new(
                field.span(),
                format_args!(
                    "duplicate formal function argument {key_name_str:?}",
                ),
            ));
        }

        let key_bytes = Literal::byte_string(key_name_str.as_bytes());
        formal_match_arms.extend(quote! { #key_bytes => true, });

        let key_name = CString::new(key_name_str)
            .map_err(|err| {
                syn::Error::new(
                    field.span(),
                    format_args!("invalid field name: {err}"),
                )
            })
            .map(|name| Literal::c_string(&name))?;

        field_names.push(field_name);

        let default_attr =
            field_attrs.default.as_ref().or(struct_attrs.default.as_ref());

        let value_ident = Ident::new("__value", Span::call_site());

        let value_expr = match field_attrs.parse_with {
            Some(parse_with) => quote!((#parse_with)(#value_ident, #ctx)?),
            None => quote!(#value_ident),
        };

        let default_expr = match default_attr {
            Some(attr) => attr.default_expr(),
            None => quote! {
                return ::core::result::Result::Err(
                    ::nixb::expr::attrset::MissingAttributeError {
                        key: #key_name,
                    }
                    .into()
                )
            },
        };

        let field_initializer = quote! {
            let #field_name = match #attrset.get_opt(#key_name, #ctx)? {
                Some(#value_ident) => {
                    #count_match
                    #value_expr
                },
                None => #default_expr,
            };
        };

        field_initializers.extend(field_initializer);
    }

    let reject_extra_attrs = (!struct_attrs.ellipsis).then(|| {
        quote! {
            if #attrset.len(#ctx) != __num_matched {
                let mut __unexpected_arg = ::core::option::Option::None;
                ::nixb::expr::attrset::MergeableAttrset::for_each_key(
                    &#attrset,
                    |__key, _| {
                        if __unexpected_arg.is_some() {
                            return;
                        }
                        let __is_formal = match __key.to_bytes() {
                            #formal_match_arms
                            _ => false,
                        };
                        if !__is_formal {
                            __unexpected_arg = ::core::option::Option::Some(
                                ::nixb::Error::from_message(
                                    ::core::format_args!(
                                        "function called with unexpected argument '{}'",
                                        __key.to_string_lossy(),
                                    ),
                                ),
                            );
                        }
                    },
                    #ctx,
                );
                let ::core::option::Option::Some(__unexpected_arg) =
                    __unexpected_arg
                else {
                    ::core::unreachable!(
                        "attribute count differs, so an unexpected argument exists",
                    );
                };
                return ::core::result::Result::Err(__unexpected_arg);
            }
        }
    });
    let matched_count = (!struct_attrs.ellipsis).then(|| {
        quote! { let mut __num_matched: ::core::ffi::c_uint = 0; }
    });

    Ok(quote! {
        #matched_count
        #field_initializers
        #reject_extra_attrs
        Ok(Self { #field_names })
    })
}

#[derive(Clone, Default)]
struct Attributes {
    rename: Option<Rename>,
    default: Option<DefaultAttr>,
    parse_with: Option<Expr>,
    ellipsis: bool,
}

#[derive(Clone)]
enum DefaultAttr {
    /// Use the type's `Default` impl.
    Default,
    /// Use a custom expression.
    Expr(Expr),
}

#[derive(Copy, Clone)]
pub(crate) enum AttributePosition {
    Field,
    Struct,
}

#[derive(Clone)]
pub(crate) enum Rename {
    CamelCase,
    Replace(String),
}

impl Attributes {
    #[expect(clippy::too_many_lines)]
    fn parse(attrs: &[Attribute], pos: AttributePosition) -> syn::Result<Self> {
        let mut this = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("pattern") {
                continue;
            }

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    match pos {
                        AttributePosition::Struct => {
                            this.rename = Some(Rename::parse(meta, pos)?);
                        },
                        AttributePosition::Field => {
                            return Err(meta.error(
                                "`rename_all` attribute is only allowed on \
                                 structs",
                            ));
                        },
                    }
                } else if meta.path.is_ident("rename") {
                    match pos {
                        AttributePosition::Struct => {
                            return Err(meta.error(
                                "`rename` attribute is only allowed on struct \
                                 fields",
                            ));
                        },
                        AttributePosition::Field => {
                            this.rename = Some(Rename::parse(meta, pos)?);
                        },
                    }
                } else if meta.path.is_ident("default") {
                    // Check if there's a value assignment (default = {expr}).
                    if meta.input.peek(syn::Token![=]) {
                        let value = meta.value()?;
                        let expr: Expr = value.parse()?;
                        this.default = Some(DefaultAttr::Expr(expr));
                    } else {
                        this.default = Some(DefaultAttr::Default);
                    }
                } else if meta.path.is_ident("parse_with") {
                    match pos {
                        AttributePosition::Struct => {
                            return Err(meta.error(
                                "`parse_with` attribute is only allowed on \
                                 struct fields",
                            ));
                        },
                        AttributePosition::Field => {
                            this.parse_with = Some(meta.value()?.parse()?);
                        },
                    }
                } else if meta.path.is_ident("ellipsis") {
                    match pos {
                        AttributePosition::Struct => {
                            this.ellipsis = true;
                        },
                        AttributePosition::Field => {
                            return Err(meta.error(
                                "`ellipsis` attribute is only allowed on \
                                 structs",
                            ));
                        },
                    }
                } else {
                    return Err(meta.error("unsupported attribute"));
                }

                Ok(())
            })?;
        }

        Ok(this)
    }
}

impl DefaultAttr {
    fn default_expr(&self) -> TokenStream {
        match self {
            Self::Default => quote!(::core::default::Default::default()),
            Self::Expr(expr) => quote!(#expr),
        }
    }
}

impl Rename {
    pub(crate) fn apply(self, field_name: &mut String) {
        match self {
            Self::CamelCase => to_camel_case(field_name),
            Self::Replace(s) => *field_name = s,
        }
    }

    pub(crate) fn parse(
        meta: ParseNestedMeta<'_>,
        pos: AttributePosition,
    ) -> syn::Result<Self> {
        let value = meta.value()?;

        let fork = value.fork();
        if let Ok(ident) = fork.parse::<syn::Ident>() {
            value.parse::<syn::Ident>()?;
            match ident.to_string().as_str() {
                "camelCase" => return Ok(Self::CamelCase),
                _ => {
                    return Err(syn::Error::new(
                        ident.span(),
                        format_args!("unsupported rename value: {}", ident),
                    ));
                },
            }
        }

        let lit: Literal = value.parse()?;
        let lit_str = lit.to_string();
        let value = lit_str.trim_matches('"');

        match pos {
            AttributePosition::Field => Ok(Self::Replace(value.to_string())),
            AttributePosition::Struct => Err(syn::Error::new(
                lit.span(),
                "literal string renames are only allowed on struct fields",
            )),
        }
    }
}

fn to_camel_case(field_name: &mut String) {
    debug_assert!(!field_name.contains(' '));

    let mut offset = 0;

    let mut replace_buffer = *b"  ";

    while offset < field_name.len() {
        let Some((component, rest)) = field_name[offset..].split_once('_')
        else {
            break;
        };

        offset += component.len();

        let Some(char_after_underscore) = rest.chars().next() else {
            // Trailing underscore.
            break;
        };

        let replacement = if char_after_underscore.is_ascii() {
            let uppercased = char_after_underscore.to_ascii_uppercase();
            replace_buffer[1] = uppercased as u8;
            str::from_utf8(&replace_buffer).expect("valid utf8")
        } else {
            " "
        };

        let replace_end = offset + 1 + (replacement.len() > 1) as usize;
        field_name.replace_range(offset..replace_end, replacement);
        offset += 1 + char_after_underscore.len_utf8();
    }

    field_name.retain(|ch| ch != ' ');
}
