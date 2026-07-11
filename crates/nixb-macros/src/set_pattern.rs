use std::collections::HashSet;
use std::ffi::CString;

use proc_macro2::{Ident, Literal, Span, TokenStream};
use quote::quote;
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

    let pattern_input = Ident::new("__input", Span::call_site());
    let ctx = Ident::new("__ctx", Span::call_site());
    let (lifetime, lifetime_generic) = lifetime_generic(&input)?;

    let pattern_impl =
        match_pattern_impl(&attrs, fields, &pattern_input, &ctx, &lifetime)?;
    let match_pattern_impl = &pattern_impl.match_pattern;
    let formal_match_count_impl = &pattern_impl.formal_match_count;
    let has_flattened_fields = pattern_impl.has_flattened;
    let struct_name = &input.ident;
    let ellipsis = attrs.ellipsis;

    Ok(quote! {
        impl<#lifetime> ::nixb::expr::set_pattern::SetPattern<#lifetime>
            for #struct_name #lifetime_generic
        {
            const ELLIPSIS: bool = #ellipsis;
            const HAS_FLATTENED_FIELDS: bool = #has_flattened_fields;

            #[inline]
            fn formal_match_count(__key: &::core::ffi::CStr) -> usize {
                #formal_match_count_impl
            }

            #[inline]
            fn match_pattern<const __CHECK_DUPLICATES: bool>(
                #pattern_input: &mut ::nixb::expr::set_pattern::SetPatternInput<#lifetime, __CHECK_DUPLICATES>,
                #ctx: &mut ::nixb::expr::context::Context,
            ) -> ::nixb::Result<Self> {
                #match_pattern_impl
            }
        }
    })
}

fn lifetime_generic(
    input: &DeriveInput,
) -> syn::Result<(LifetimeParam, Option<TokenStream>)> {
    if input.generics.params.is_empty() {
        return Ok((parse_quote!('__pattern), None));
    }

    let Some(syn::GenericParam::Lifetime(lifetime)) =
        input.generics.params.first()
    else {
        return Err(syn::Error::new(
            input.generics.span(),
            "set patterns can only have zero or one lifetime generic parameter",
        ));
    };
    if input.generics.params.len() != 1 {
        return Err(syn::Error::new(
            input.generics.span(),
            "set patterns can only have zero or one lifetime generic parameter",
        ));
    }

    Ok((lifetime.clone(), Some(quote! { <#lifetime> })))
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

struct PatternImpl {
    match_pattern: TokenStream,
    formal_match_count: TokenStream,
    has_flattened: bool,
}

#[expect(clippy::too_many_lines)]
#[expect(clippy::too_many_arguments)]
fn match_pattern_impl(
    struct_attrs: &Attributes,
    fields: &FieldsNamed,
    input: &Ident,
    ctx: &Ident,
    lifetime: &LifetimeParam,
) -> syn::Result<PatternImpl> {
    let mut field_names = Punctuated::<_, Comma>::new();
    let mut field_initializers = TokenStream::new();
    let mut formal_match_arms = TokenStream::new();
    let mut flattened_match_counts = TokenStream::new();
    let mut formal_names = HashSet::new();
    let mut has_flattened = false;

    for field in fields.named.iter() {
        let field_attrs =
            Attributes::parse(&field.attrs, AttributePosition::Field)?;

        let field_name = field.ident.as_ref().expect("fields are named");
        field_names.push(field_name);

        if field_attrs.flatten {
            has_flattened = true;
            if field_attrs.rename.is_some()
                || field_attrs.default.is_some()
                || field_attrs.parse_with.is_some()
            {
                return Err(syn::Error::new(
                    field.span(),
                    "`flatten` cannot be combined with other attributes",
                ));
            }

            let field_ty = &field.ty;
            field_initializers.extend(quote! {
                let #field_name =
                    <#field_ty as ::nixb::expr::set_pattern::SetPattern<#lifetime>>::match_pattern::<__CHECK_DUPLICATES>(
                        #input,
                        #ctx,
                    )?;
            });
            flattened_match_counts.extend(quote! {
                + <#field_ty as ::nixb::expr::set_pattern::SetPattern<#lifetime>>::formal_match_count(__key)
            });
            continue;
        }

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
        formal_match_arms.extend(quote! { #key_bytes => 1, });

        let key_name = CString::new(key_name_str)
            .map_err(|err| {
                syn::Error::new(
                    field.span(),
                    format_args!("invalid field name: {err}"),
                )
            })
            .map(|name| Literal::c_string(&name))?;

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
            let #field_name = match #input.take(#key_name, #ctx)? {
                Some(#value_ident) => #value_expr,
                None => #default_expr,
            };
        };

        field_initializers.extend(field_initializer);
    }

    Ok(PatternImpl {
        match_pattern: quote! {
            #field_initializers
            Ok(Self { #field_names })
        },
        formal_match_count: quote! {
            (match __key.to_bytes() {
                #formal_match_arms
                _ => 0,
            }) #flattened_match_counts
        },
        has_flattened,
    })
}

#[derive(Clone, Default)]
struct Attributes {
    rename: Option<Rename>,
    default: Option<DefaultAttr>,
    parse_with: Option<Expr>,
    flatten: bool,
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
                } else if meta.path.is_ident("flatten") {
                    match pos {
                        AttributePosition::Struct => {
                            return Err(meta.error(
                                "`flatten` attribute is only allowed on \
                                 struct fields",
                            ));
                        },
                        AttributePosition::Field => {
                            this.flatten = true;
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
