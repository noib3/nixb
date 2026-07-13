use core::iter;
use std::ffi::CString;

use proc_macro2::{Literal, Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{
    Attribute,
    Data,
    DeriveInput,
    Expr,
    Fields,
    FieldsNamed,
    Ident,
    Token,
    WherePredicate,
};

use crate::set_pattern::{AttributePosition, Rename};

const MACRO_NAME: &str = "Attrset";

#[expect(clippy::cognitive_complexity)]
#[expect(clippy::too_many_lines)]
#[inline]
pub(crate) fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let attrs = Attributes::parse(&input.attrs, AttributePosition::Struct)?;

    let struct_name = &input.ident;

    let mut fields = named_fields(&input)?
        .named
        .iter()
        .map(|field| Field::new(field, &attrs, struct_name))
        .collect::<syn::Result<Vec<_>>>()?;

    // Sort the fields by key so that the generated attrset is ordered.
    fields
        .sort_by(|a, b| a.field_key_as_c_string.cmp(&b.field_key_as_c_string));

    let (impl_generics, ty_generics, where_clause) =
        input.generics.split_for_impl();

    let extended_where_clause = if !attrs.bounds.is_empty() {
        let predicates = where_clause
            .map(|wc| wc.predicates.iter())
            .into_iter()
            .flatten()
            .chain(attrs.bounds.iter());
        quote! { where #(#predicates),* }
    } else {
        quote! { #where_clause }
    };

    let eval_lifetime = syn::Lifetime::new("'__eval", Span::call_site());

    let use_params = input
        .generics
        .params
        .iter()
        .map(|param| match param {
            syn::GenericParam::Lifetime(lt) => {
                let lifetime = &lt.lifetime;
                quote! { #lifetime }
            },
            syn::GenericParam::Type(ty) => {
                let ident = &ty.ident;
                quote! { #ident }
            },
            syn::GenericParam::Const(c) => {
                let ident = &c.ident;
                quote! { #ident }
            },
        })
        .chain(iter::once(quote! { #eval_lifetime }));

    let keys = fields
        .iter()
        .map(|field| Literal::c_string(&field.field_key_as_c_string));

    let is_present = fields.iter().map(|field| match &field.should_skip_expr {
        Some(_) => {
            let skip_var = &field.skip_var_name;
            quote! { !#skip_var }
        },
        None => quote! { true },
    });

    let num_non_skippable_fields =
        fields.iter().filter(|f| f.should_skip_expr.is_none()).count();

    let into_value_wrappers = fields
        .iter()
        .map(|field| field.into_value_adapter.clone())
        .collect::<Vec<_>>();

    let key_types = fields
        .iter()
        .map(|_| quote! { &'static ::core::ffi::CStr })
        .collect::<Vec<_>>();

    let value_types = fields
        .iter()
        .zip(&into_value_wrappers)
        .map(|(field, wrapper)| {
            let field_ty = &field.field_ty;
            match wrapper {
                Some(wrapper) => quote! {
                    #wrapper<#struct_name #ty_generics, #field_ty>
                },
                None => quote! { #field_ty },
            }
        })
        .collect::<Vec<_>>();

    let tuple = |items: &[_]| match items {
        [item] => quote! { #item },
        _ => quote! { (#(#items),*) },
    };
    let key_tuple_type = tuple(&key_types);
    let value_tuple_type = tuple(&value_types);
    let static_attrset_type = if num_non_skippable_fields == fields.len() {
        quote! {
            ::nixb::expr::attrset::StaticAttrset<
                true,
                #key_tuple_type,
                #value_tuple_type,
            >
        }
    } else {
        let num_fields = Literal::usize_unsuffixed(fields.len());
        quote! {
            ::nixb::expr::attrset::StaticAttrsetWithOptionalFields<
                true,
                #key_tuple_type,
                #value_tuple_type,
                #num_fields,
            >
        }
    };

    let values =
        fields.iter().zip(&into_value_wrappers).map(|(field, wrapper)| {
            let value_expr = &field.value_expr;
            match wrapper {
                Some(wrapper) => quote! {
                    #wrapper {
                        field: #value_expr,
                        _owner: ::core::marker::PhantomData,
                    }
                },
                None => quote! { #value_expr },
            }
        });

    let into_static_attrset_body = if num_non_skippable_fields == fields.len() {
        quote! {
            ::nixb::expr::attrset::StaticAttrset::new(
                (#(#keys),*),
                (#(#values),*),
            )
        }
    } else {
        let skip_var_declarations = fields.iter().filter_map(|field| {
            field.should_skip_expr.as_ref().map(|skip_expr| {
                let skip_var = &field.skip_var_name;
                quote! { let #skip_var = #skip_expr; }
            })
        });

        let num_non_skippable =
            Literal::usize_unsuffixed(num_non_skippable_fields);

        let len_increments = fields.iter().filter_map(|field| {
            field.should_skip_expr.as_ref().map(|_| {
                let skip_var = &field.skip_var_name;
                quote! {
                    if !#skip_var {
                        __len += 1;
                    }
                }
            })
        });

        quote! {
            #(#skip_var_declarations)*
            let mut __len: ::core::ffi::c_uint = #num_non_skippable;
            #(#len_increments)*
            ::nixb::expr::attrset::StaticAttrsetWithOptionalFields::new(
                (#(#keys),*),
                (#(#values),*),
                [#(#is_present),*],
                __len,
            )
        }
    };

    let contains_key_match_arms = fields.iter().map(|field| {
        let key_bytes = Literal::byte_string(field.field_key.as_bytes());
        let arm_body = match &field.should_skip_expr {
            Some(expr) => quote! { !#expr },
            None => quote! { true },
        };
        quote! { #key_bytes => #arm_body, }
    });

    let ctx = syn::Ident::new("__ctx", Span::call_site());

    let for_each_key_stmts = fields.iter().map(|field| {
        let key_name = Literal::c_string(&field.field_key_as_c_string);
        let call = quote! { __fun(#key_name, #ctx); };
        match &field.should_skip_expr {
            Some(expr) => quote! { if !#expr { #call } },
            None => call,
        }
    });

    let into_value_wrapper_impls = fields
        .iter()
        .zip(&into_value_wrappers)
        .filter_map(|(field, wrapper)| {
            let visibility = &input.vis;
            let wrapper = wrapper.as_ref()?;
            let field_ty = &field.field_ty;
            let expr = field.into_value_expr.as_ref().expect("wrapper exists");
            let use_params = use_params.clone();

            Some(quote! {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #visibility struct #wrapper<__Owner, __Field> {
                    field: __Field,
                    _owner: ::core::marker::PhantomData<fn() -> __Owner>,
                }

                impl #impl_generics ::nixb::expr::value::IntoValue
                    for #wrapper<#struct_name #ty_generics, #field_ty>
                    #extended_where_clause
                {
                    #[inline]
                    fn into_value<#eval_lifetime>(
                        self,
                        #ctx: &mut ::nixb::expr::context::Context<#eval_lifetime>,
                    ) -> impl ::nixb::expr::value::Value + use<#(#use_params),*> {
                        // This helper lets closures infer the field's type.
                        #[inline(always)]
                        fn __call<T, R>(f: impl FnOnce(T) -> R, value: T) -> R {
                            f(value)
                        }

                        ::nixb::expr::value::IntoValue::into_value(
                            __call(#expr, self.field),
                            #ctx,
                        )
                    }
                }
            })
        });

    let rename_self = (num_non_skippable_fields < fields.len())
        .then(|| quote! { let __value = self; });

    Ok(quote! {
        #(#into_value_wrapper_impls)*

        impl #impl_generics ::core::convert::From<#struct_name #ty_generics>
            for #static_attrset_type
            #extended_where_clause
        {
            #[inline]
            fn from(__value: #struct_name #ty_generics) -> Self {
                #into_static_attrset_body
            }
        }

        impl #impl_generics ::nixb::expr::attrset::Attrset for #struct_name #ty_generics #extended_where_clause {
            #[inline]
            fn into_attrset_iter<#eval_lifetime>(
                self,
                #ctx: &mut ::nixb::expr::context::Context<#eval_lifetime>,
            ) -> impl ::nixb::expr::attrset::AttrsetIterator + use<#(#use_params),*> {
                <#static_attrset_type>::from(self).into_attrset_iter(#ctx)
            }
        }

        impl #impl_generics ::nixb::expr::attrset::MergeableAttrset for #struct_name #ty_generics #extended_where_clause {
            #[inline]
            fn contains_key(&self, __key: &::core::ffi::CStr, _: &mut ::nixb::expr::context::Context) -> bool {
                #rename_self
                match __key.to_bytes() {
                    #(#contains_key_match_arms)*
                    _ => false,
                }
            }

            #[inline]
            fn for_each_key<#eval_lifetime>(
                &self,
                mut __fun: impl FnMut(&::core::ffi::CStr, &mut ::nixb::expr::context::Context<#eval_lifetime>),
                #ctx: &mut ::nixb::expr::context::Context<#eval_lifetime>,
            ) {
                #rename_self
                #(#for_each_key_stmts)*
            }
        }

        impl #impl_generics ::nixb::expr::value::Value for #struct_name #ty_generics #extended_where_clause {
            #[inline]
            fn kind(&self) -> ::nixb::expr::value::ValueKind {
                ::nixb::expr::value::ValueKind::Attrset
            }

            #[inline]
            fn write(
                self,
                dest: ::nixb::expr::value::UninitValue,
                ctx: &mut ::nixb::expr::context::Context,
            ) {
                ::nixb::expr::attrset::Attrset::write(self, dest, ctx)
            }
        }
    })
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

#[derive(Clone, Default)]
struct Attributes {
    rename: Option<Rename>,
    skip_if: Option<Expr>,
    into_value: Option<Expr>,
    bounds: Vec<WherePredicate>,
}

struct Field {
    field_ty: syn::Type,
    field_key_as_c_string: CString,
    field_key: String,
    should_skip_expr: Option<TokenStream>,
    skip_var_name: Ident,
    into_value_adapter: Option<Ident>,
    into_value_expr: Option<Expr>,
    value_expr: TokenStream,
}

impl Attributes {
    #[expect(clippy::too_many_lines)]
    fn parse(attrs: &[Attribute], pos: AttributePosition) -> syn::Result<Self> {
        let mut this = Self::default();

        for attr in attrs {
            if !attr.path().is_ident("attrset") {
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
                } else if meta.path.is_ident("skip_if") {
                    this.skip_if = Some(meta.value()?.parse()?);
                } else if meta.path.is_ident("into_value") {
                    match pos {
                        AttributePosition::Struct => {
                            return Err(meta.error(
                                "`into_value` attribute is only allowed on \
                                 struct fields",
                            ));
                        },
                        AttributePosition::Field => {
                            this.into_value = Some(meta.value()?.parse()?);
                        },
                    }
                } else if meta.path.is_ident("bounds") {
                    match pos {
                        AttributePosition::Struct => {
                            meta.input.parse::<Token![=]>()?;
                            let content;
                            syn::braced!(content in meta.input);
                            let bounds: Punctuated<WherePredicate, Token![,]> =
                                content
                                    .parse_terminated(Parse::parse, Token![,])?;
                            this.bounds.extend(bounds);
                        },
                        AttributePosition::Field => {
                            return Err(meta.error(
                                "`bounds` attribute is only allowed on structs",
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

impl Field {
    fn new(
        field: &syn::Field,
        struct_attrs: &Attributes,
        struct_name: &Ident,
    ) -> syn::Result<Self> {
        let field_attrs =
            Attributes::parse(&field.attrs, AttributePosition::Field)?;

        let field_ident = field.ident.clone().expect("fields are named");

        let field_key = {
            let mut key = field_ident.to_string();

            // Strip the `r#` prefix from raw identifiers.
            if let Some(stripped) = key.strip_prefix("r#") {
                key = stripped.to_owned();
            }

            if let Some(rename) =
                field_attrs.rename.as_ref().or(struct_attrs.rename.as_ref())
            {
                rename.clone().apply(&mut key);
            }

            key
        };

        let field_key_as_c_string =
            CString::new(field_key.clone()).map_err(|err| {
                syn::Error::new(
                    field.span(),
                    format_args!("invalid field name: {err}"),
                )
            })?;

        let should_skip_expr = field_attrs
            .skip_if
            .as_ref()
            .or(struct_attrs.skip_if.as_ref())
            .cloned();

        let should_skip_expr = should_skip_expr.map(|expr| {
            quote! {{
                // This helper lets closures infer the field's type.
                #[inline(always)]
                fn __call<T>(f: impl FnOnce(T) -> bool, value: T) -> bool {
                    f(value)
                }

                __call(#expr, &__value.#field_ident)
            }}
        });

        let skip_var_name = format_ident!("__should_skip_{field_ident}");
        let field_name = field_ident.to_string();
        let field_name = field_name.strip_prefix("r#").unwrap_or(&field_name);
        let into_value_adapter = field_attrs.into_value.as_ref().map(|_| {
            format_ident!("__{struct_name}{field_name}IntoValueAdapter")
        });
        let value_expr = quote! { __value.#field_ident };

        Ok(Self {
            field_ty: field.ty.clone(),
            field_key,
            field_key_as_c_string,
            should_skip_expr,
            skip_var_name,
            into_value_adapter,
            into_value_expr: field_attrs.into_value,
            value_expr,
        })
    }
}
