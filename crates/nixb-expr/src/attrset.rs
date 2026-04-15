//! TODO: docs.

use alloc::borrow::ToOwned;
use alloc::ffi::CString;
use alloc::format;
use alloc::string::String;
use alloc::vec::{self, Vec};
use core::cell::OnceCell;
use core::ffi::{CStr, c_uint};
use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::{self, NonNull};
use core::result::Result as CoreResult;
use core::{fmt, mem};

pub use nixb_macros::attrset;

use crate::callable::{Callable, NixLambda};
use crate::context::{AttrsetBuilder, Context};
use crate::error::{Error, ErrorKind, Result, TypeMismatchError};
use crate::function::function;
use crate::tuple::Tuple;
use crate::value::{
    Borrowed,
    IntoValue,
    IntoValues,
    NixValue,
    Owned,
    TryFromValue,
    UninitValue,
    Value,
    ValueKind,
    ValueOwner,
    Values,
};
use crate::{IntoResult, Utf8CStr};

/// TODO: docs.
pub trait Attrset {
    /// TODO: docs.
    fn into_attrset_iter<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl AttrsetIterator + use<'eval, Self>;

    /// TODO: docs.
    ///
    /// # Safety
    ///
    /// The caller must ensure that there are no overlapping keys between
    /// `self` and `other`.
    #[inline]
    unsafe fn concat<T: Attrset>(self, other: T) -> ConcatAttrset<Self, T>
    where
        Self: Sized,
    {
        ConcatAttrset { left: self, right: other }
    }

    /// TODO: docs.
    #[inline]
    fn into_value(self) -> impl Value
    where
        Self: Sized,
    {
        struct Wrapper<T>(T);

        impl<T: Attrset> Value for Wrapper<T> {
            #[inline(always)]
            fn kind(&self) -> ValueKind {
                ValueKind::Attrset
            }

            #[inline(always)]
            fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
                Attrset::write(self.0, dest, ctx)
            }
        }

        Wrapper(self)
    }

    #[doc(hidden)]
    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()>
    where
        Self: Sized,
    {
        struct WriteNext {
            dest: UninitValue,
        }

        impl FnOnceKeyValueIter<AttrsetBuilder<'_, '_>, Result<()>> for WriteNext {
            #[inline]
            fn call(
                self,
                key: impl Key,
                value: impl Value,
                rest: impl AttrsetIterator,
                mut builder: AttrsetBuilder<'_, '_>,
            ) -> Result<()> {
                key.with_cstr(|key| {
                    builder.insert(key, |dest, ctx| value.write(dest, ctx))
                })?;

                if rest.is_exhausted() {
                    builder.build(self.dest)
                } else {
                    rest.with_next(self, builder)
                }
            }
        }

        let iter = self.into_attrset_iter(ctx);
        let builder = ctx.make_attrset_builder(iter.len() as usize)?;
        if iter.is_exhausted() {
            builder.build(dest)
        } else {
            iter.with_next(WriteNext { dest }, builder)
        }
    }
}

/// TODO: docs.
#[expect(clippy::len_without_is_empty, reason = "I called it is_exhausted")]
pub trait AttrsetIterator {
    /// Returns the number of elements remaining in the iterator.
    fn len(&self) -> c_uint;

    /// TODO: docs.
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T;

    /// Returns whether the iterator is exhausted.
    #[inline(always)]
    fn is_exhausted(&self) -> bool {
        self.len() == 0
    }
}

/// TODO: docs.
pub trait MergeableAttrset: Attrset {
    /// TODO: docs.
    fn contains_key(&self, key: &CStr, ctx: &mut Context) -> bool;

    /// TODO: docs.
    fn for_each_key<'eval>(
        &self,
        fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    );

    /// TODO: docs.
    #[inline]
    fn merge<T: MergeableAttrset>(self, other: T) -> Merge<Self, T>
    where
        Self: Sized,
    {
        Merge { left: self, right: other, conflicts: OnceCell::new() }
    }
}

/// TODO: docs.
pub trait Key: fmt::Debug {
    /// TODO: docs.
    fn with_cstr<T>(&self, fun: impl FnOnce(&CStr) -> T) -> T;
}

/// TODO: docs.
pub trait Keys:
    Tuple<First: Key, Last: Key, FromFirst: IntoKeys, UpToLast: IntoKeys>
{
    /// Calls the given closure for each key in the tuple.
    ///
    /// The closure must return a boolean indicating whether to continue
    /// iterating over the remaining keys. Returning `false` will stop the
    /// iteration early.
    #[inline]
    fn for_each(self, mut fun: impl FnMut(&CStr) -> bool)
    where
        Self: Sized,
    {
        if self.is_empty() {
            return;
        }
        let (first, rest) = self.split_first();
        if first.with_cstr(&mut fun) {
            rest.into_keys().for_each(fun);
        }
    }
}

#[doc(hidden)]
pub trait IntoKeys {
    fn into_keys(self) -> impl Keys;
}

/// TODO: docs.
pub trait FnOnceKeyValueIter<Ctx, Out> {
    /// TODO: docs.
    #[expect(clippy::too_many_arguments)]
    fn call(
        self,
        key: impl Key,
        value: impl Value,
        iter: impl AttrsetIterator,
        ctx: Ctx,
    ) -> Out;
}

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct NixAttrset<Owner = Owned> {
    inner: NixValue<Owner>,
}

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct NixDerivation<Owner = Owned> {
    inner: NixAttrset<Owner>,
}

/// The attribute set type produced by the [`attrset!`] macro.
#[derive(Clone)]
pub struct StaticAttrset<const KEYS_ARE_ORDERED: bool, Keys, Values> {
    keys: Keys,
    values: Values,
}

/// TODO: docs.
#[derive(Clone)]
pub struct StaticAttrsetWithSkips<const KEYS_ARE_ORDERED: bool, Keys, Values> {
    keys: Keys,
    values: Values,
    len: c_uint,
}

/// The attribute set type created by [`concat`](Attrset::concat)enating two
/// attribute sets.
#[derive(Clone)]
pub struct ConcatAttrset<Left, Right> {
    left: Left,
    right: Right,
}

/// The attribute set type created by [`merge`](MergeableAttrset::merge)ing two
/// attribute sets.
#[derive(Clone)]
pub struct Merge<Left, Right> {
    left: Left,
    right: Right,
    conflicts: OnceCell<Vec<CString>>,
}

/// The type of error returned when an expected attribute is missing from
/// an [`Attrset`].
#[derive(Debug)]
pub struct MissingAttributeError<Key> {
    /// The key of the missing attribute.
    pub key: Key,
}

struct NixAttrsetIterator<Owner> {
    iterator: NonNull<nixb_cpp::AttrIterator>,
    len: c_uint,
    _attrset: PhantomData<NixAttrset<Owner>>,
}

struct StaticAttrsetIterator<K, V> {
    keys: K,
    values: V,
}

struct ConcatIterator<L, R> {
    left: L,
    right: R,
}

struct MergeIterator<L, R> {
    left: L,
    right: R,
    conflicts: vec::IntoIter<CString>,
}

impl<Owner: ValueOwner> NixAttrset<Owner> {
    /// TODO: docs.
    #[inline]
    pub fn as_borrowed(&self) -> NixAttrset<Borrowed<'_>> {
        NixAttrset { inner: self.inner.as_borrowed() }
    }

    /// TODO: docs.
    #[inline]
    pub fn borrow(&self) -> NixAttrset<Owner::Borrow<'_>> {
        NixAttrset { inner: self.inner.borrow() }
    }

    /// TODO: docs.
    #[inline]
    pub fn get<'a, T: TryFromValue<NixValue<Owner::Borrow<'a>>>>(
        &'a self,
        key: impl Key,
        ctx: &mut Context,
    ) -> Result<T> {
        match self.get_opt(&key, ctx)? {
            Some(value) => Ok(value),
            None => Err(MissingAttributeError { key }.into()),
        }
    }

    /// TODO: docs.
    #[inline]
    pub fn get_opt<'a, T: TryFromValue<NixValue<Owner::Borrow<'a>>>>(
        &'a self,
        key: impl Key,
        ctx: &mut Context,
    ) -> Result<Option<T>> {
        let Some(value) = key.with_cstr(|key| self.get_value(key, ctx)) else {
            return Ok(None);
        };

        T::try_from_value(value, ctx).map(Some).map_err(|err| {
            err.map_message(|msg| {
                let mut orig_msg = msg.into_owned().into_bytes_with_nul();
                let mut new_msg =
                    format!("error getting attribute {key:?}: ").into_bytes();
                new_msg.append(&mut orig_msg);
                // SAFETY: the new message does contain a NUL byte and
                // we've preserved the trailing NUL byte from the
                // original message.
                unsafe { CString::from_vec_with_nul_unchecked(new_msg) }
            })
        })
    }

    /// TODO: docs.
    #[inline]
    pub fn get_nested<'a, T: TryFromValue<NixValue<Owner::Borrow<'a>>>>(
        &'a self,
        keys: impl Keys,
        ctx: &mut Context,
    ) -> Result<T> {
        self.get_nested_inner(
            keys,
            |key| MissingAttributeError { key }.into(),
            |_key, error| error,
            ctx,
        )
        .and_then(|val| T::try_from_value(val, ctx))
    }

    /// TODO: docs.
    #[inline]
    pub fn get_nested_opt<'a, T: TryFromValue<NixValue<Owner::Borrow<'a>>>>(
        &'a self,
        keys: impl Keys,
        ctx: &mut Context,
    ) -> Result<Option<T>> {
        match self.get_nested_inner(
            keys,
            |_key| None,
            |_key, error| Some(error),
            ctx,
        ) {
            Ok(val) => T::try_from_value(val, ctx).map(Some),
            Err(None) => Ok(None),
            Err(Some(try_from_value_err)) => Err(try_from_value_err),
        }
    }

    /// TODO: docs.
    #[inline]
    pub fn into_owned(self) -> NixAttrset<Owned> {
        NixAttrset { inner: self.inner.into_owned() }
    }

    /// Returns whether this attribute set is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of attributes in this attribute set.
    #[inline]
    pub fn len(&self) -> c_uint {
        // 'nix_get_attrs_size' errors when the value pointer is null or when
        // the value is not initizialized, but having a NixValue guarantees
        // neither of those can happen, so we can use a null context.
        unsafe {
            nixb_sys::get_attrs_size(ptr::null_mut(), self.inner.as_ptr())
        }
    }

    #[inline]
    #[expect(clippy::too_many_arguments)]
    fn get_nested_inner<'this, Err>(
        &'this self,
        keys: impl Keys,
        on_key_missing: impl FnOnce(&CStr) -> Err,
        on_get_error: impl FnOnce(&CStr, Error) -> Err,
        ctx: &mut Context,
    ) -> CoreResult<NixValue<Owner::Borrow<'this>>, Err> {
        fn get_attrs<'a, OnKeyMissing, Err>(
            attrs: NixAttrset<Borrowed<'a>>,
            keys: impl Keys,
            on_key_missing: OnKeyMissing,
            on_get_error: impl FnOnce(&CStr, Error) -> Err,
            ctx: &mut Context,
        ) -> CoreResult<(NixAttrset<Borrowed<'a>>, OnKeyMissing), Err>
        where
            OnKeyMissing: FnOnce(&CStr) -> Err,
        {
            if keys.is_empty() {
                Ok((attrs, on_key_missing))
            } else {
                let (key, rest) = keys.split_first();
                let res = key.with_cstr(|key| attrs.get_opt(key, ctx));
                match res {
                    Ok(Some(next_attrs)) => get_attrs(
                        next_attrs,
                        rest.into_keys(),
                        on_key_missing,
                        on_get_error,
                        ctx,
                    ),
                    Ok(None) => Err(key.with_cstr(on_key_missing)),
                    Err(err) => {
                        Err(key.with_cstr(|key| on_get_error(key, err)))
                    },
                }
            }
        }

        let (up_to_last, last) = keys.split_last();

        let (attrs, on_key_missing) = get_attrs(
            self.as_borrowed(),
            up_to_last.into_keys(),
            on_key_missing,
            on_get_error,
            ctx,
        )?;

        let Some(val): Option<NixValue<Borrowed<'this>>> =
            last.with_cstr(|key| attrs.get_value(key, ctx))
        else {
            return Err(last.with_cstr(on_key_missing));
        };

        let val_ptr = val.owner().value_ptr();

        // SAFETY: the `Borrow` GAT is always `Borrowed`, the only difference
        // between `Owned::Borrow` and `Borrowed::Borrow` is the lifetime.
        //
        // If `Owner` is `Owned`, then `Owner::Borrow<'this>` is
        // `Borrowed<'this>`, so we're returning the same value.
        //
        // If `Owner` is `Borrowed<'outer>`, then `Owner::Borrow<'this>` is
        // `Borrowed<'outer>`, which means we're effectively extending the
        // borrow's lifetime from `'this` to `'outer`, which is also sound
        // because we're not extending it past `'outer`.
        let borrow = unsafe { Owner::Borrow::new(val_ptr) };

        Ok(NixValue::new(borrow))
    }

    #[inline]
    fn get_value<'this>(
        &'this self,
        key: &CStr,
        ctx: &mut Context,
    ) -> Option<NixValue<Owner::Borrow<'this>>> {
        let value_raw = unsafe {
            nixb_cpp::get_attr_byname_lazy_no_incref(
                self.inner.as_ptr(),
                ctx.state_ptr(),
                key.as_ptr(),
            )
        };

        NonNull::new(value_raw)
            .map(|ptr| unsafe { Owner::Borrow::new(ptr) })
            .map(NixValue::new)
    }
}

impl<Owner: ValueOwner> NixDerivation<Owner> {
    /// TODO: docs.
    #[inline]
    pub fn borrow(&self) -> NixDerivation<Owner::Borrow<'_>> {
        NixDerivation { inner: self.inner.borrow() }
    }

    /// TODO: docs.
    #[inline]
    pub fn into_owned(self) -> NixDerivation<Owned> {
        NixDerivation { inner: self.inner.into_owned() }
    }

    /// Returns the output path of this derivation.
    #[inline]
    pub fn override_attrs<'a, NewAttrs>(
        &self,
        fun: impl FnMut(NixAttrset<Borrowed<'a>>, &mut Context) -> NewAttrs
        + 'static,
        ctx: &mut Context,
    ) -> Result<NixDerivation>
    where
        NewAttrs: IntoResult<Output: Attrset + Value, Error: Into<Error>> + 'a,
    {
        self.inner
            .get::<NixLambda<_>>(c"overrideAttrs", ctx)?
            .call(function::<NixAttrset<Borrowed<'a>>>(fun), ctx)?
            .force_into(ctx)
    }

    /// Returns the output path of this derivation.
    #[cfg(feature = "std")]
    #[inline]
    pub fn out_path(&self, ctx: &mut Context) -> Result<std::path::PathBuf> {
        self.out_path_as_string(ctx).map(Into::into)
    }

    /// Returns the output path of this derivation as a string.
    #[inline]
    pub fn out_path_as_string(&self, ctx: &mut Context) -> Result<String> {
        self.inner.get(c"outPath", ctx)
    }

    /// TODO: docs.
    #[inline]
    pub fn realise(&self, ctx: &mut Context) -> Result<()> {
        let expr = c"drv: \"${drv}\"";
        let string_drv =
            ctx.eval::<NixLambda>(expr)?.call(self.inner.borrow(), ctx)?;
        let value = string_drv.into_inner();
        let realised_str = ctx.with_raw_and_state(|ctx, state| unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            {
                nixb_cpp::string_realise(
                    ctx,
                    state.as_ptr(),
                    value.as_ptr(),
                    true,
                )
            }

            #[cfg(feature = "nix-2-34")]
            nixb_sys::string_realise(ctx, state.as_ptr(), value.as_ptr(), true)
        })?;
        unsafe {
            nixb_sys::realised_string_free(realised_str);
        }
        Ok(())
    }
}

impl<Owner: ValueOwner> NixAttrsetIterator<Owner> {
    #[inline]
    unsafe fn change_owner<NewOwner>(self) -> NixAttrsetIterator<NewOwner> {
        let new_iter = NixAttrsetIterator {
            iterator: self.iterator,
            len: self.len,
            _attrset: PhantomData::<NixAttrset<NewOwner>>,
        };
        // We've transferred ownership of the iterator pointer to the new
        // iterator, so don't drop `self` or we'll run into a double-free.
        mem::forget(self);
        new_iter
    }

    #[inline]
    fn new(set: NixAttrset<Owner>, ctx: &mut Context) -> Self {
        let iter_raw = unsafe {
            nixb_cpp::attr_iter_create(set.inner.as_ptr(), ctx.state_ptr())
        };

        let iterator =
            NonNull::new(iter_raw).expect("failed to create attr iterator");

        Self { iterator, len: set.len(), _attrset: PhantomData }
    }
}

impl<const KEYS_ARE_ORDERED: bool, K, V> StaticAttrset<KEYS_ARE_ORDERED, K, V> {
    #[doc(hidden)]
    #[inline]
    pub fn new(keys: K, values: V) -> Self {
        Self { keys, values }
    }
}

impl<L: MergeableAttrset, R: MergeableAttrset> Merge<L, R> {
    #[inline]
    fn init_conflicts(&self, ctx: &mut Context) -> &[CString] {
        self.conflicts.get_or_init(|| {
            let mut conflicts = Vec::new();

            self.left.for_each_key(
                |key, ctx| {
                    if self.right.contains_key(key, ctx) {
                        conflicts.push(key.to_owned().to_owned());
                    }
                },
                ctx,
            );

            conflicts
        })
    }
}

impl<Owner: ValueOwner> Attrset for NixAttrset<Owner> {
    #[inline]
    fn into_value(self) -> impl Value
    where
        Self: Sized,
    {
        self
    }

    #[inline]
    fn into_attrset_iter(
        self,
        ctx: &mut Context,
    ) -> impl AttrsetIterator + use<Owner> {
        NixAttrsetIterator::new(self, ctx)
    }
}

impl<Owner: ValueOwner> MergeableAttrset for NixAttrset<Owner> {
    #[inline]
    fn contains_key(&self, key: &CStr, ctx: &mut Context) -> bool {
        self.get_value(key, ctx).is_some()
    }

    #[inline]
    fn for_each_key<'eval>(
        &self,
        mut fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    ) {
        for (key, _value) in NixAttrsetIterator::new(self.as_borrowed(), ctx) {
            fun(key, ctx);
        }
    }
}

impl<Owner: ValueOwner> Value for NixAttrset<Owner> {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Attrset
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        self.inner.write(dest, ctx)
    }
}

impl<Owner: ValueOwner> TryFromValue<NixValue<Owner>> for NixAttrset<Owner> {
    #[inline]
    fn try_from_value(
        mut value: NixValue<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        value.force_inline(ctx)?;

        match value.kind() {
            ValueKind::Attrset => Ok(Self { inner: value }),
            other => Err(TypeMismatchError {
                expected: ValueKind::Attrset,
                found: other,
            }
            .into()),
        }
    }
}

impl<Owner: ValueOwner> From<NixDerivation<Owner>> for NixAttrset<Owner> {
    #[inline]
    fn from(derivation: NixDerivation<Owner>) -> Self {
        derivation.inner
    }
}

impl<Owner: ValueOwner> From<NixAttrset<Owner>> for NixValue<Owner> {
    #[inline]
    fn from(attrset: NixAttrset<Owner>) -> Self {
        attrset.inner
    }
}

impl<Owner: ValueOwner> Attrset for NixDerivation<Owner> {
    #[inline]
    fn into_attrset_iter(
        self,
        ctx: &mut Context,
    ) -> impl AttrsetIterator + use<Owner>
    where
        Self: Sized,
    {
        self.inner.into_attrset_iter(ctx)
    }
}

impl<Owner: ValueOwner> MergeableAttrset for NixDerivation<Owner> {
    #[inline]
    fn contains_key(&self, key: &CStr, ctx: &mut Context) -> bool {
        self.inner.contains_key(key, ctx)
    }

    #[inline]
    fn for_each_key<'eval>(
        &self,
        fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    ) {
        self.inner.for_each_key(fun, ctx);
    }
}

impl<Owner: ValueOwner> Value for NixDerivation<Owner> {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Attrset
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        Value::write(self.inner, dest, ctx)
    }
}

impl<Owner: ValueOwner> Deref for NixDerivation<Owner> {
    type Target = NixAttrset<Owner>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<Owner: ValueOwner> TryFromValue<NixValue<Owner>> for NixDerivation<Owner> {
    #[inline]
    fn try_from_value(
        value: NixValue<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        NixAttrset::try_from_value(value, ctx)
            .and_then(|attrset| Self::try_from_value(attrset, ctx))
    }
}

impl<Owner: ValueOwner> TryFromValue<NixAttrset<Owner>>
    for NixDerivation<Owner>
{
    #[inline]
    fn try_from_value(
        attrset: NixAttrset<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        let Some(mut r#type) = attrset.get_opt::<NixValue<_>>(c"type", ctx)?
        else {
            return Err(Error::new(
                ErrorKind::Nix,
                c"attrset doesn't have the \"type\" attribute, so it can't be a derivation",
            ));
        };

        r#type.force_inline(ctx)?;

        if r#type.kind() != ValueKind::String {
            return Err(Error::new(
                ErrorKind::Nix,
                c"the \"type\" attribute is not a string, so the attrset can't be a derivation",
            ));
        }

        let mut is_type_derivation = false;

        // SAFETY: we've just checked that the `type` is a string.
        unsafe {
            r#type.with_string(
                |str| is_type_derivation = str == c"derivation",
                ctx,
            )?
        }

        if !is_type_derivation {
            return Err(Error::new(
                ErrorKind::Nix,
                c"the \"type\" attribute is not \"derivation\", so the attrset can't be a derivation",
            ));
        }

        drop(r#type);

        Ok(Self { inner: attrset })
    }
}

impl<const KEYS_ARE_ORDERED: bool, K: Keys, V: Values> Attrset
    for StaticAttrset<KEYS_ARE_ORDERED, K, V>
{
    #[inline]
    fn into_attrset_iter(
        self,
        _: &mut Context,
    ) -> impl AttrsetIterator + use<KEYS_ARE_ORDERED, K, V>
    where
        Self: Sized,
    {
        StaticAttrsetIterator { keys: self.keys, values: self.values }
    }
}

impl<const KEYS_ARE_ORDERED: bool> Attrset
    for StaticAttrset<KEYS_ARE_ORDERED, (), ()>
{
    #[inline]
    fn into_attrset_iter(
        self,
        _: &mut Context,
    ) -> impl AttrsetIterator + use<KEYS_ARE_ORDERED>
    where
        Self: Sized,
    {
        StaticAttrsetIterator { keys: (), values: () }
    }
}

impl<const KEYS_ARE_ORDERED: bool, K: Keys, V: Values> MergeableAttrset
    for StaticAttrset<KEYS_ARE_ORDERED, K, V>
where
    for<'a> K::Borrow<'a>: Keys,
{
    #[inline]
    fn contains_key(&self, key: &CStr, _: &mut Context) -> bool {
        let mut contains_key = false;

        self.keys.borrow().for_each(|probe| {
            if key == probe {
                contains_key = true;
                false
            } else {
                true
            }
        });

        contains_key
    }

    #[inline]
    fn for_each_key<'eval>(
        &self,
        mut fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    ) {
        self.keys.borrow().for_each(|key| {
            fun(key, ctx);
            true
        });
    }
}

impl<const KEYS_ARE_ORDERED: bool, K: Keys, V: Values> Value
    for StaticAttrset<KEYS_ARE_ORDERED, K, V>
{
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Attrset
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        Attrset::write(self, dest, ctx)
    }
}

impl<const KEYS_ARE_ORDERED: bool> Value
    for StaticAttrset<KEYS_ARE_ORDERED, (), ()>
{
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Attrset
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        Attrset::write(self, dest, ctx)
    }
}

impl<L: Attrset, R: Attrset> Attrset for ConcatAttrset<L, R> {
    #[inline]
    fn into_attrset_iter<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl AttrsetIterator + use<'eval, L, R>
    where
        Self: Sized,
    {
        ConcatIterator {
            left: self.left.into_attrset_iter(ctx),
            right: self.right.into_attrset_iter(ctx),
        }
    }
}

impl<L: MergeableAttrset, R: MergeableAttrset> MergeableAttrset
    for ConcatAttrset<L, R>
{
    #[inline]
    fn contains_key(&self, key: &CStr, ctx: &mut Context) -> bool {
        self.left.contains_key(key, ctx) || self.right.contains_key(key, ctx)
    }

    #[inline]
    fn for_each_key<'eval>(
        &self,
        mut fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    ) {
        self.left.for_each_key(|key, ctx| fun(key, ctx), ctx);
        self.right.for_each_key(|key, ctx| fun(key, ctx), ctx);
    }
}

impl<L, R> Value for ConcatAttrset<L, R>
where
    Self: Attrset,
{
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Attrset
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        Attrset::write(self, dest, ctx)
    }
}

impl<L: MergeableAttrset, R: MergeableAttrset> Attrset for Merge<L, R> {
    #[inline]
    fn into_attrset_iter<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl AttrsetIterator + use<'eval, L, R> {
        self.init_conflicts(ctx);

        MergeIterator {
            left: self.left.into_attrset_iter(ctx),
            right: self.right.into_attrset_iter(ctx),
            conflicts: self
                .conflicts
                .into_inner()
                .expect("conflicts have just been initialized")
                .into_iter(),
        }
    }
}

impl<L: MergeableAttrset, R: MergeableAttrset> MergeableAttrset
    for Merge<L, R>
{
    #[inline]
    fn contains_key(&self, key: &CStr, ctx: &mut Context) -> bool {
        self.left.contains_key(key, ctx) || self.right.contains_key(key, ctx)
    }

    #[inline]
    fn for_each_key<'eval>(
        &self,
        mut fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    ) {
        let mut conflicts = self.init_conflicts(ctx);

        self.left.for_each_key(
            |key, ctx| {
                // Don't call the function on conflicting keys, as they will be
                // handled when iterating over the right attrset.
                if conflicts.first().map(|key| &**key) == Some(key) {
                    conflicts = &conflicts[1..];
                } else {
                    fun(key, ctx);
                }
            },
            ctx,
        );

        self.right.for_each_key(|key, ctx| fun(key, ctx), ctx);
    }
}

impl<L, R> Value for Merge<L, R>
where
    Self: Attrset,
{
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Attrset
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        Attrset::write(self, dest, ctx)
    }
}

impl<'set> Iterator for NixAttrsetIterator<Borrowed<'set>> {
    type Item = (&'set CStr, NixValue<Borrowed<'set>>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.is_exhausted() {
            return None;
        }

        // SAFETY: Nix guarantees that the key pointer is valid as long as the
        // iterator is.
        let key = unsafe {
            CStr::from_ptr(nixb_cpp::attr_iter_key(self.iterator.as_ptr()))
        };

        let value_ptr = unsafe {
            NonNull::new(nixb_cpp::attr_iter_value(self.iterator.as_ptr()))
                .expect("not null if not exhausted")
        };

        // SAFETY: the value returned by Nix is initialized, and it guarantees
        // that the value pointer is valid as long as the iterator is.
        let borrow = unsafe { Borrowed::<'set>::new(value_ptr) };

        if !self.is_exhausted() {
            // SAFETY: we've checked that the iterator is not exhausted.
            unsafe { nixb_cpp::attr_iter_advance(self.iterator.as_ptr()) };
            self.len -= 1;
        }

        Some((key, NixValue::new(borrow)))
    }
}

impl<Owner: ValueOwner> AttrsetIterator for NixAttrsetIterator<Owner> {
    #[inline]
    fn len(&self) -> c_uint {
        self.len
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T {
        let mut iter = unsafe { self.change_owner::<Borrowed>() };
        let Some((key, value)) = iter.next() else {
            panic!("called with_next() on an exhausted iterator");
        };
        let rest: Self = unsafe { iter.change_owner::<Owner>() };
        fun.call(key, value, rest, ctx)
    }
}

impl<Owner> Drop for NixAttrsetIterator<Owner> {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_cpp::attr_iter_destroy(self.iterator.as_ptr()) };
    }
}

impl<A: Attrset> Attrset for Option<A> {
    #[inline]
    fn into_attrset_iter<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl AttrsetIterator + use<'eval, A>
    where
        Self: Sized,
    {
        self.map(|attrset| attrset.into_attrset_iter(ctx))
    }
}

impl<A: MergeableAttrset> MergeableAttrset for Option<A> {
    #[inline]
    fn contains_key(&self, key: &CStr, ctx: &mut Context) -> bool {
        self.as_ref().is_some_and(|attrset| attrset.contains_key(key, ctx))
    }

    #[inline]
    fn for_each_key<'eval>(
        &self,
        fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    ) {
        if let Some(attrset) = self.as_ref() {
            attrset.for_each_key(fun, ctx);
        }
    }
}

impl<A: AttrsetIterator> AttrsetIterator for Option<A> {
    #[inline]
    fn len(&self) -> c_uint {
        self.as_ref().map_or(0, AttrsetIterator::len)
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T {
        self.expect("called with_next() on an exhausted iterator")
            .with_next(fun, ctx)
    }
}

impl<T: Key + ?Sized> Key for &T {
    #[inline(always)]
    fn with_cstr<F>(&self, fun: impl FnOnce(&CStr) -> F) -> F {
        (*self).with_cstr(fun)
    }
}

macro_rules! impl_key_for_as_ref_cstr {
    ($ty:ty) => {
        impl Key for $ty {
            #[inline(always)]
            fn with_cstr<T>(&self, fun: impl FnOnce(&CStr) -> T) -> T {
                fun(self.as_ref())
            }
        }
    };
}

impl_key_for_as_ref_cstr!(CStr);
impl_key_for_as_ref_cstr!(Utf8CStr);

/// # Panics
///
/// The [`with_cstr`](Key::with_cstr) implementation will panic if the string
/// contains a NUL byte.
#[cfg(feature = "std")]
impl Key for str {
    #[track_caller]
    #[inline]
    fn with_cstr<T>(&self, fun: impl FnOnce(&CStr) -> T) -> T {
        use core::cell::RefCell;

        std::thread_local! {
            static KEY_BUFFER: RefCell<Vec<u8>> = RefCell::default();
        }

        if self.as_bytes().contains(&0) {
            panic!(
                "string {self:?} contains a NUL byte, so it cannot be used as \
                 an attrset key"
            )
        }

        KEY_BUFFER.with_borrow_mut(|buf| {
            buf.clear();
            buf.extend_from_slice(self.as_bytes());
            buf.push(0);
            // SAFETY: we checked that the string doesn't contain any NUL bytes,
            // and we just pushed a trailing NUL.
            let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(buf) };
            fun(cstr)
        })
    }
}

impl<K: Keys, V: Values> AttrsetIterator for StaticAttrsetIterator<K, V> {
    #[inline]
    fn len(&self) -> c_uint {
        debug_assert_eq!(K::LEN, V::LEN);
        K::LEN as c_uint
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        mut ctx: Ctx,
    ) -> T {
        let (first_key, rest_keys) = self.keys.split_first();
        let (first_value, rest_values) = self.values.split_first();
        let rest = StaticAttrsetIterator {
            keys: rest_keys.into_keys(),
            values: rest_values.into_values(),
        };
        fun.call(first_key, first_value.into_value(ctx.as_mut()), rest, ctx)
    }
}

impl AttrsetIterator for StaticAttrsetIterator<(), ()> {
    #[inline]
    fn len(&self) -> c_uint {
        0
    }

    #[cold]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        _: impl FnOnceKeyValueIter<Ctx, T>,
        _: Ctx,
    ) -> T {
        panic!("AttrsetItrator::with_next called on an exhausted iterator");
    }
}

impl<L: AttrsetIterator, R: AttrsetIterator> AttrsetIterator
    for ConcatIterator<L, R>
{
    #[inline]
    fn len(&self) -> c_uint {
        self.left.len() + self.right.len()
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T {
        if self.left.is_exhausted() {
            return self.right.with_next(fun, ctx);
        }

        struct Wrapper<F, R> {
            fun: F,
            right: R,
        }

        impl<'a, F, R, C, U> FnOnceKeyValueIter<C, U> for Wrapper<F, R>
        where
            F: FnOnceKeyValueIter<C, U>,
            R: AttrsetIterator,
            C: AsMut<Context<'a>>,
        {
            #[inline]
            fn call(
                self,
                key: impl Key,
                value: impl Value,
                left_rest: impl AttrsetIterator,
                ctx: C,
            ) -> U {
                self.fun.call(
                    key,
                    value,
                    ConcatIterator { left: left_rest, right: self.right },
                    ctx,
                )
            }
        }

        self.left.with_next(Wrapper { fun, right: self.right }, ctx)
    }
}

impl<L: AttrsetIterator, R: AttrsetIterator> AttrsetIterator
    for MergeIterator<L, R>
{
    #[inline]
    fn len(&self) -> c_uint {
        self.left.len() + self.right.len() - (self.conflicts.len() as c_uint)
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T {
        if self.left.is_exhausted() {
            return self.right.with_next(fun, ctx);
        }

        struct Wrapper<F, R> {
            fun: F,
            right: R,
            conflicts: vec::IntoIter<CString>,
        }

        impl<'a, F, R, C, U> FnOnceKeyValueIter<C, U> for Wrapper<F, R>
        where
            F: FnOnceKeyValueIter<C, U>,
            R: AttrsetIterator,
            C: AsMut<Context<'a>>,
        {
            #[inline]
            fn call(
                mut self,
                key: impl Key,
                value: impl Value,
                left_rest: impl AttrsetIterator,
                ctx: C,
            ) -> U {
                // Skip the current key from the left iterator if it's also
                // contained in the right iterator.
                if let Some(next_conflicting_key) =
                    self.conflicts.as_slice().first()
                    && key.with_cstr(|key| key == &**next_conflicting_key)
                {
                    let _ = self.conflicts.next();
                    MergeIterator {
                        left: left_rest,
                        right: self.right,
                        conflicts: self.conflicts,
                    }
                    .with_next(self.fun, ctx)
                } else {
                    let rest = MergeIterator {
                        left: left_rest,
                        right: self.right,
                        conflicts: self.conflicts,
                    };
                    self.fun.call(key, value, rest, ctx)
                }
            }
        }

        self.left.with_next(
            Wrapper { fun, right: self.right, conflicts: self.conflicts },
            ctx,
        )
    }
}

impl<K: Key> fmt::Display for MissingAttributeError<K> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.key.with_cstr(|key| {
            write!(f, "attribute at '{}' missing", key.to_string_lossy())
        })
    }
}

impl<K: Key> From<MissingAttributeError<K>> for Error {
    #[inline]
    fn from(err: MissingAttributeError<K>) -> Self {
        Self::from_message(err)
    }
}

#[cfg(all(feature = "compact_str", feature = "std"))]
impl Key for compact_str::CompactString {
    #[inline(always)]
    fn with_cstr<T>(&self, fun: impl FnOnce(&CStr) -> T) -> T {
        self.as_str().with_cstr(fun)
    }
}

#[cfg(feature = "either")]
impl<L: Attrset, R: Attrset> Attrset for either::Either<L, R> {
    #[inline]
    fn into_attrset_iter<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl AttrsetIterator + use<'eval, L, R>
    where
        Self: Sized,
    {
        match self {
            Self::Left(left) => {
                either::Either::Left(left.into_attrset_iter(ctx))
            },
            Self::Right(right) => {
                either::Either::Right(right.into_attrset_iter(ctx))
            },
        }
    }
}

#[cfg(feature = "either")]
impl<L: AttrsetIterator, R: AttrsetIterator> AttrsetIterator
    for either::Either<L, R>
{
    #[inline]
    fn len(&self) -> c_uint {
        match self {
            Self::Left(left) => left.len(),
            Self::Right(right) => right.len(),
        }
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T {
        match self {
            Self::Left(left) => left.with_next(fun, ctx),
            Self::Right(right) => right.with_next(fun, ctx),
        }
    }
}

#[cfg(feature = "either")]
impl<L: MergeableAttrset, R: MergeableAttrset> MergeableAttrset
    for either::Either<L, R>
{
    #[inline]
    fn contains_key(&self, key: &CStr, ctx: &mut Context) -> bool {
        match self {
            Self::Left(left) => left.contains_key(key, ctx),
            Self::Right(right) => right.contains_key(key, ctx),
        }
    }

    #[inline]
    fn for_each_key<'eval>(
        &self,
        mut fun: impl FnMut(&CStr, &mut Context<'eval>),
        ctx: &mut Context<'eval>,
    ) {
        match self {
            Self::Left(left) => {
                left.for_each_key(|key, ctx| fun(key, ctx), ctx)
            },
            Self::Right(right) => {
                right.for_each_key(|key, ctx| fun(key, ctx), ctx)
            },
        }
    }
}

#[cfg(feature = "either")]
impl<L: Key, R: Key> Key for either::Either<L, R> {
    #[inline]
    fn with_cstr<T>(&self, fun: impl FnOnce(&CStr) -> T) -> T {
        match self {
            Self::Left(left) => left.with_cstr(fun),
            Self::Right(right) => right.with_cstr(fun),
        }
    }
}

#[cfg(feature = "std")]
impl<K, V, S> Attrset for std::collections::HashMap<K, V, S>
where
    K: Key,
    V: IntoValue,
{
    #[inline]
    fn into_attrset_iter(
        self,
        _: &mut Context,
    ) -> impl AttrsetIterator + use<K, V, S>
    where
        Self: Sized,
    {
        IntoIterator::into_iter(self)
    }
}

#[cfg(feature = "std")]
impl<K, V> AttrsetIterator for std::collections::hash_map::IntoIter<K, V>
where
    K: Key,
    V: IntoValue,
{
    #[inline]
    fn len(&self) -> c_uint {
        ExactSizeIterator::len(self) as c_uint
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        mut self,
        fun: impl FnOnceKeyValueIter<Ctx, T>,
        mut ctx: Ctx,
    ) -> T {
        let (key, value) = Iterator::next(&mut self)
            .expect("called AttrsetIterator::next on exhausted iterator");
        fun.call(key, value.into_value(ctx.as_mut()), self, ctx)
    }
}

#[cfg(feature = "std")]
impl<K, V, S> Value for std::collections::HashMap<K, V, S>
where
    Self: Attrset,
{
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Attrset
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        Attrset::write(self, dest, ctx)
    }
}

impl<T> Keys for T where
    T: Tuple<First: Key, Last: Key, FromFirst: IntoKeys, UpToLast: IntoKeys>
{
}

impl IntoKeys for () {
    #[inline]
    fn into_keys(self) -> impl Keys {
        #[derive(Copy, Clone)]
        struct EmptyKeys;

        impl Tuple for EmptyKeys {
            const LEN: usize = 0;
            type First = &'static CStr;
            type Last = &'static CStr;
            type FromFirst = ();
            type UpToLast = ();
            type Borrow<'a> = Self;

            fn borrow(&self) -> Self::Borrow<'_> {
                *self
            }
            fn split_first(self) -> (Self::First, Self::FromFirst) {
                panic!("cannot split first from empty keys")
            }
            fn split_last(self) -> (Self::UpToLast, Self::Last) {
                panic!("cannot split last from empty keys")
            }
        }

        EmptyKeys
    }
}

impl<T: Keys> IntoKeys for T {
    fn into_keys(self) -> impl Keys {
        self
    }
}

#[doc(hidden)]
pub mod skips {
    //! TODO: docs.

    use super::*;

    #[derive(Clone)]
    pub struct MightSkip<V> {
        should_skip: bool,
        value: V,
    }

    pub trait SkippableValue {
        fn should_skip(&self) -> bool;

        /// Same as [`IntoValue::into_value`].
        fn into_value<'eval>(
            self,
            ctx: &mut Context<'eval>,
        ) -> impl Value + use<'eval, Self>;
    }

    pub trait SkippableValues:
        Tuple<First: SkippableValue, FromFirst: IntoSkippableValues>
    {
    }

    pub trait SkippableValuesRefs:
        Tuple<
            First: Deref<Target: SkippableValue>,
            FromFirst: IntoSkippableValuesRefs,
        >
    {
    }

    pub trait IntoSkippableValues {
        fn into_values(self) -> impl SkippableValues;
    }

    pub trait IntoSkippableValuesRefs {
        fn into_values(self) -> impl SkippableValuesRefs;
    }

    struct StaticAttrsetWithSkipsIterator<K, V> {
        keys: K,
        values: V,
        len: c_uint,
    }

    impl<V> MightSkip<V> {
        #[inline]
        pub fn new(value: V, should_skip: bool) -> Self {
            Self { should_skip, value }
        }
    }

    impl<const KEYS_ARE_ORDERED: bool, K, V>
        StaticAttrsetWithSkips<KEYS_ARE_ORDERED, K, V>
    {
        #[doc(hidden)]
        #[inline]
        pub fn new(keys: K, values: V, len: c_uint) -> Self {
            Self { keys, values, len }
        }

        #[inline]
        fn is_empty(&self) -> bool {
            self.len == 0
        }
    }

    impl<const KEYS_ARE_ORDERED: bool, K, V>
        StaticAttrsetWithSkips<KEYS_ARE_ORDERED, K, V>
    where
        K: Keys,
        V: SkippableValuesRefs,
    {
        #[expect(clippy::same_name_method)]
        #[inline]
        fn contains_key(self, key: &CStr) -> bool {
            if self.is_empty() {
                return false;
            }
            let (first_key, rest_keys) = self.keys.split_first();
            let (first_value, rest_values) = self.values.split_first();
            let should_skip = first_value.deref().should_skip();

            if first_key.with_cstr(|k| k == key) {
                !should_skip
            } else {
                StaticAttrsetWithSkips::<KEYS_ARE_ORDERED, _, _> {
                    keys: rest_keys.into_keys(),
                    values: rest_values.into_values(),
                    // The length only decreases if this key-value pair is not
                    // skipped.
                    len: self.len - (!should_skip as c_uint),
                }
                .contains_key(key)
            }
        }

        #[expect(clippy::same_name_method)]
        #[inline]
        fn for_each_key<'eval>(
            self,
            mut fun: impl FnMut(&CStr, &mut Context<'eval>),
            ctx: &mut Context<'eval>,
        ) {
            if self.is_empty() {
                return;
            }
            let (first_key, rest_keys) = self.keys.split_first();
            let (first_value, rest_values) = self.values.split_first();
            let should_skip = first_value.deref().should_skip();
            if !should_skip {
                first_key.with_cstr(|key| fun(key, ctx));
            }
            StaticAttrsetWithSkips::<KEYS_ARE_ORDERED, _, _> {
                keys: rest_keys.into_keys(),
                values: rest_values.into_values(),
                // The length only decreases if this key-value pair is not
                // skipped.
                len: self.len - (!should_skip as c_uint),
            }
            .for_each_key(fun, ctx);
        }
    }

    impl<const KEYS_ARE_ORDERED: bool, K: Keys, V: SkippableValues> Attrset
        for StaticAttrsetWithSkips<KEYS_ARE_ORDERED, K, V>
    {
        #[inline]
        fn into_attrset_iter(
            self,
            _: &mut Context,
        ) -> impl AttrsetIterator + use<KEYS_ARE_ORDERED, K, V>
        where
            Self: Sized,
        {
            StaticAttrsetWithSkipsIterator {
                keys: self.keys,
                values: self.values,
                len: self.len,
            }
        }
    }

    impl<const KEYS_ARE_ORDERED: bool, K: Keys, V: SkippableValues>
        MergeableAttrset for StaticAttrsetWithSkips<KEYS_ARE_ORDERED, K, V>
    where
        for<'a> K::Borrow<'a>: Keys,
        for<'a> V::Borrow<'a>: SkippableValuesRefs,
    {
        #[inline]
        fn contains_key(&self, key: &CStr, _: &mut Context) -> bool {
            StaticAttrsetWithSkips::<KEYS_ARE_ORDERED, _, _> {
                keys: self.keys.borrow(),
                values: self.values.borrow(),
                len: self.len,
            }
            .contains_key(key)
        }

        #[inline]
        fn for_each_key<'eval>(
            &self,
            fun: impl FnMut(&CStr, &mut Context<'eval>),
            ctx: &mut Context<'eval>,
        ) {
            StaticAttrsetWithSkips::<KEYS_ARE_ORDERED, _, _> {
                keys: self.keys.borrow(),
                values: self.values.borrow(),
                len: self.len,
            }
            .for_each_key(fun, ctx)
        }
    }

    impl<const KEYS_ARE_ORDERED: bool, K: Keys, V: SkippableValues> Value
        for StaticAttrsetWithSkips<KEYS_ARE_ORDERED, K, V>
    {
        #[inline]
        fn kind(&self) -> ValueKind {
            ValueKind::Attrset
        }

        #[inline]
        fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
            Attrset::write(self, dest, ctx)
        }
    }

    impl<K: Keys, V: SkippableValues> AttrsetIterator
        for StaticAttrsetWithSkipsIterator<K, V>
    {
        #[inline]
        fn len(&self) -> c_uint {
            self.len
        }

        #[inline]
        fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
            self,
            fun: impl FnOnceKeyValueIter<Ctx, T>,
            mut ctx: Ctx,
        ) -> T {
            let (first_key, rest_keys) = self.keys.split_first();
            let (first_value, rest_values) = self.values.split_first();
            if first_value.should_skip() {
                StaticAttrsetWithSkipsIterator {
                    keys: rest_keys.into_keys(),
                    values: rest_values.into_values(),
                    // This key-value pair is skipped, so the length remains
                    // the same.
                    len: self.len,
                }
                .with_next(fun, ctx)
            } else {
                let rest = StaticAttrsetWithSkipsIterator {
                    keys: rest_keys.into_keys(),
                    values: rest_values.into_values(),
                    // The key-value pair is not skipped, so the length
                    // decreases by 1.
                    len: self.len - 1,
                };
                fun.call(
                    first_key,
                    first_value.into_value(ctx.as_mut()),
                    rest,
                    ctx,
                )
            }
        }
    }

    impl<V: IntoValue> SkippableValue for V {
        #[inline]
        fn should_skip(&self) -> bool {
            false
        }

        #[inline]
        fn into_value<'eval>(
            self,
            ctx: &mut Context<'eval>,
        ) -> impl Value + use<'eval, V> {
            IntoValue::into_value(self, ctx)
        }
    }

    impl<V: IntoValue> SkippableValue for MightSkip<V> {
        #[inline]
        fn should_skip(&self) -> bool {
            self.should_skip
        }

        #[inline]
        fn into_value<'eval>(
            self,
            ctx: &mut Context<'eval>,
        ) -> impl Value + use<'eval, V> {
            self.value.into_value(ctx)
        }
    }

    impl<T> SkippableValues for T where
        T: Tuple<First: SkippableValue, FromFirst: IntoSkippableValues>
    {
    }

    impl<T: SkippableValues> IntoSkippableValues for T {
        fn into_values(self) -> impl SkippableValues {
            self
        }
    }

    impl IntoSkippableValues for () {
        #[inline]
        fn into_values(self) -> impl SkippableValues {
            #[derive(Copy, Clone)]
            struct EmptySkippableValues;

            impl Tuple for EmptySkippableValues {
                const LEN: usize = 0;
                type First = u8;
                type Last = u8;
                type FromFirst = ();
                type UpToLast = ();
                type Borrow<'a> = Self;

                fn borrow(&self) -> Self::Borrow<'_> {
                    *self
                }
                fn split_first(self) -> (Self::First, Self::FromFirst) {
                    panic!("cannot split first from empty values")
                }
                fn split_last(self) -> (Self::UpToLast, Self::Last) {
                    panic!("cannot split last from empty values")
                }
            }

            EmptySkippableValues
        }
    }

    impl<V> SkippableValuesRefs for V where
        V: Tuple<
                First: Deref<Target: SkippableValue>,
                FromFirst: IntoSkippableValuesRefs,
            >
    {
    }

    impl<T: SkippableValuesRefs> IntoSkippableValuesRefs for T {
        fn into_values(self) -> impl SkippableValuesRefs {
            self
        }
    }

    impl IntoSkippableValuesRefs for () {
        #[inline]
        fn into_values(self) -> impl SkippableValuesRefs {
            #[derive(Copy, Clone)]
            struct EmptySkippableValuesRefs;

            impl Tuple for EmptySkippableValuesRefs {
                const LEN: usize = 0;
                type First = &'static u8;
                type Last = u8;
                type FromFirst = ();
                type UpToLast = ();
                type Borrow<'a> = Self;

                fn borrow(&self) -> Self::Borrow<'_> {
                    *self
                }
                fn split_first(self) -> (Self::First, Self::FromFirst) {
                    panic!("cannot split first from empty values")
                }
                fn split_last(self) -> (Self::UpToLast, Self::Last) {
                    panic!("cannot split last from empty values")
                }
            }

            EmptySkippableValuesRefs
        }
    }
}
