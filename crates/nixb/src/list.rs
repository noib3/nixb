//! TODO: docs.

use alloc::vec::Vec;
use core::ffi::c_uint;
use core::ops::Deref;
use core::ptr::{self, NonNull};

pub use nixb_macros::list;

use crate::context::ListBuilder;
use crate::error::TypeMismatchError;
use crate::prelude::{Context, Result, Value, ValueKind};
use crate::value::{
    Borrowed,
    IntoValue,
    IntoValues,
    NixValue,
    Owned,
    TryFromValue,
    UninitValue,
    ValueOwner,
    Values,
};

/// TODO: docs.
pub trait List {
    /// TODO: docs.
    fn into_list_iter<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl ListIterator + use<'eval, Self>;

    /// TODO: docs.
    #[inline]
    fn concat<T: List>(self, other: T) -> ConcatList<Self, T>
    where
        Self: Sized,
    {
        ConcatList { left: self, right: other }
    }

    /// TODO: docs.
    #[inline]
    fn into_value(self) -> impl Value
    where
        Self: Sized,
    {
        struct Wrapper<L>(L);

        impl<L: List> Value for Wrapper<L> {
            #[inline(always)]
            fn kind(&self) -> ValueKind {
                ValueKind::List
            }

            #[inline(always)]
            fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
                List::write(self.0, dest, ctx)
            }
        }

        Wrapper(self)
    }

    /// TODO: docs.
    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()>
    where
        Self: Sized,
    {
        struct WriteNext {
            dest: UninitValue,
        }

        impl FnOnceValueIter<ListBuilder<'_, '_>, Result<()>> for WriteNext {
            #[inline]
            fn call(
                self,
                value: impl Value,
                rest: impl ListIterator,
                mut builder: ListBuilder<'_, '_>,
            ) -> Result<()> {
                builder.insert(|dest, ctx| value.write(dest, ctx))?;

                if rest.is_exhausted() {
                    builder.build(self.dest)
                } else {
                    rest.with_next(self, builder)
                }
            }
        }

        let iter = self.into_list_iter(ctx);
        let builder = ctx.make_list_builder(iter.len() as usize)?;
        if iter.is_exhausted() {
            builder.build(dest)
        } else {
            iter.with_next(WriteNext { dest }, builder)
        }
    }
}

/// TODO: docs.
#[expect(clippy::len_without_is_empty, reason = "I called it is_exhausted")]
pub trait ListIterator {
    /// Returns the number of elements in this list.
    fn len(&self) -> c_uint;

    /// TODO: docs.
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T;

    /// Returns whether the iterator is exhausted.
    #[inline(always)]
    fn is_exhausted(&self) -> bool {
        self.len() == 0
    }
}

/// An extension trait for iterators.
pub trait IteratorExt: IntoIterator<IntoIter: ExactSizeIterator> {
    /// Chains two [`ExactSizeIterator`]s together, returning a new
    /// [`ExactSizeIterator`] that will iterate over both.
    ///
    /// See the discussion in <https://github.com/rust-lang/rust/issues/34433>
    /// for why [`Chain`](core::iter::Chain) doesn't already do this.
    ///
    /// # Panics
    ///
    /// The [`ExactSizeIterator::len`] implementation of the returned iterator
    /// will panic if the sum of the two iterators' lengths overflows a
    /// `usize`.
    #[inline]
    fn chain_exact<T>(self, other: T) -> ChainExact<Self::IntoIter, T::IntoIter>
    where
        Self: Sized,
        T: IntoIterator<IntoIter: ExactSizeIterator<Item = Self::Item>>,
    {
        ChainExact {
            left: Some(self.into_iter()),
            right: Some(other.into_iter()),
        }
    }
}

/// TODO: docs.
pub trait FnOnceValueIter<Ctx, Out> {
    /// TODO: docs.
    fn call(self, value: impl Value, iter: impl ListIterator, ctx: Ctx) -> Out;
}

/// TODO: docs.
#[derive(Debug, Copy, Clone)]
pub struct NixList<Owner = Owned> {
    inner: NixValue<Owner>,
}

/// The list type produced by the [`list!`] macro.
pub struct StaticList<Values> {
    values: Values,
}

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct ConcatList<L, R> {
    left: L,
    right: R,
}

/// The iterator type returned by calling [`IteratorExt::chain_exact`].
#[derive(Clone)]
pub struct ChainExact<L, R> {
    left: Option<L>,
    right: Option<R>,
}

impl<Owner: ValueOwner> NixList<Owner> {
    /// TODO: docs.
    #[inline]
    pub fn borrow(&self) -> NixList<Owner::Borrow<'_>> {
        NixList { inner: self.inner.borrow() }
    }

    /// TODO: docs.
    #[inline]
    pub fn get<'a, T: TryFromValue<NixValue<Owner::Borrow<'a>>>>(
        &'a self,
        idx: c_uint,
        ctx: &mut Context,
    ) -> Result<T> {
        let Some(value) = self.get_value(idx) else {
            panic!(
                "Tried to get index {} from list of length {}",
                idx,
                self.len()
            );
        };
        T::try_from_value(value, ctx)
    }

    /// TODO: docs.
    #[inline]
    pub fn into_owned(self) -> NixList<Owned> {
        NixList { inner: self.inner.into_owned() }
    }

    /// Returns whether this list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of elements in this list.
    #[inline]
    pub fn len(&self) -> c_uint {
        // 'nix_get_list_size' errors when the value pointer is null or when
        // the value is not initizialized, but having a NixValue guarantees
        // neither of those can happen, so we can use a null context.
        unsafe { nixb_sys::get_list_size(ptr::null_mut(), self.inner.as_ptr()) }
    }

    #[inline]
    fn get_value<'this>(
        &'this self,
        idx: c_uint,
    ) -> Option<NixValue<Owner::Borrow<'this>>> {
        let value_raw = unsafe {
            nixb_cpp::get_list_byidx_lazy_no_incref(self.inner.as_ptr(), idx)
        };

        NonNull::new(value_raw)
            .map(|ptr| unsafe { Owner::Borrow::new(ptr) })
            .map(NixValue::new)
    }
}

impl<Values> StaticList<Values> {
    /// Creates a new [`StaticList`].
    #[inline]
    pub fn new(values: Values) -> Self {
        Self { values }
    }
}

impl<Owner: ValueOwner> List for NixList<Owner> {
    #[inline]
    fn into_list_iter(self, _: &mut Context) -> impl ListIterator + use<Owner>
    where
        Self: Sized,
    {
        struct NixListIterator<Owner> {
            list: NixList<Owner>,
            num_yielded: c_uint,
        }

        impl<Owner: ValueOwner> ListIterator for NixListIterator<Owner> {
            #[inline]
            fn len(&self) -> c_uint {
                self.list.len() - self.num_yielded
            }

            #[inline]
            fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
                mut self,
                fun: impl FnOnceValueIter<Ctx, T>,
                ctx: Ctx,
            ) -> T {
                let item = self
                    .list
                    .get_value(self.num_yielded)
                    .expect(
                        "ListIterator::with_next() called more times than \
                         advertised by len",
                    )
                    .into_owned();
                self.num_yielded += 1;
                fun.call(item, self, ctx)
            }
        }

        NixListIterator { list: self, num_yielded: 0 }
    }

    #[inline]
    fn into_value(self) -> impl Value
    where
        Self: Sized,
    {
        self
    }
}

impl<Owner: ValueOwner> Value for NixList<Owner> {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::List
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        self.inner.write(dest, ctx)
    }
}

impl<Owner: ValueOwner> TryFromValue<NixValue<Owner>> for NixList<Owner> {
    #[inline]
    fn try_from_value(
        mut value: NixValue<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        value.force_inline(ctx)?;

        match value.kind() {
            ValueKind::List => Ok(Self { inner: value }),
            other => Err(TypeMismatchError {
                expected: ValueKind::List,
                found: other,
            }
            .into()),
        }
    }
}

impl<Owner: ValueOwner> From<NixList<Owner>> for NixValue<Owner> {
    #[inline]
    fn from(list: NixList<Owner>) -> Self {
        list.inner
    }
}

impl<V> Deref for StaticList<V> {
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<V: Values> List for StaticList<V> {
    #[inline]
    fn into_list_iter(self, _: &mut Context) -> impl ListIterator + use<V> {
        self
    }
}

impl<V: Values> Value for StaticList<V> {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::List
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        List::write(self, dest, ctx)
    }
}

impl<L, R> List for ConcatList<L, R>
where
    L: List,
    R: List,
{
    #[inline]
    fn into_list_iter<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl ListIterator + use<'eval, L, R> {
        ConcatList {
            left: self.left.into_list_iter(ctx),
            right: self.right.into_list_iter(ctx),
        }
    }
}

impl<L, R> ListIterator for ConcatList<L, R>
where
    L: ListIterator,
    R: ListIterator,
{
    #[inline]
    fn len(&self) -> c_uint {
        self.left.len() + self.right.len()
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceValueIter<Ctx, T>,
        ctx: Ctx,
    ) -> T {
        if self.left.is_exhausted() {
            return self.right.with_next(fun, ctx);
        }

        struct Wrapper<F, R> {
            fun: F,
            right: R,
        }

        impl<'a, F, R, C, U> FnOnceValueIter<C, U> for Wrapper<F, R>
        where
            F: FnOnceValueIter<C, U>,
            R: ListIterator,
            C: AsMut<Context<'a>>,
        {
            #[inline]
            fn call(
                self,
                value: impl Value,
                left_rest: impl ListIterator,
                ctx: C,
            ) -> U {
                self.fun.call(
                    value,
                    ConcatList { left: left_rest, right: self.right },
                    ctx,
                )
            }
        }

        self.left.with_next(Wrapper { fun, right: self.right }, ctx)
    }
}

impl<L, R> Value for ConcatList<L, R>
where
    Self: List,
{
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::List
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        List::write(self, dest, ctx)
    }
}

impl<V: Values> ListIterator for StaticList<V> {
    #[inline]
    fn len(&self) -> c_uint {
        V::LEN as c_uint
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        self,
        fun: impl FnOnceValueIter<Ctx, T>,
        mut ctx: Ctx,
    ) -> T {
        let (first, rest) = V::split_first(self.values);
        fun.call(
            first.into_value(ctx.as_mut()),
            StaticList::new(rest.into_values()),
            ctx,
        )
    }
}

impl<I: IntoIterator> List for I
where
    I::IntoIter: ExactSizeIterator,
    I::Item: IntoValue,
{
    #[inline]
    fn into_list_iter(self, _: &mut Context) -> I::IntoIter {
        IntoIterator::into_iter(self)
    }
}

impl<I: ExactSizeIterator<Item: IntoValue>> ListIterator for I {
    #[inline]
    fn len(&self) -> c_uint {
        match ExactSizeIterator::len(self).try_into() {
            Ok(len) => len,
            Err(_overflow_err) => {
                panic!("iterator has too many elements, max is {}", c_uint::MAX)
            },
        }
    }

    #[inline]
    fn with_next<'eval, Ctx: AsMut<Context<'eval>>, T>(
        mut self,
        fun: impl FnOnceValueIter<Ctx, T>,
        mut ctx: Ctx,
    ) -> T {
        let value = self
            .next()
            .expect(
                "ListIterator::with_next() called more times than advertised \
                 by len",
            )
            .into_value(ctx.as_mut());
        fun.call(value, self, ctx)
    }
}

impl<I: IntoIterator<IntoIter: ExactSizeIterator>> IteratorExt for I {}

impl<L, R> Iterator for ChainExact<L, R>
where
    L: ExactSizeIterator,
    R: ExactSizeIterator<Item = L::Item>,
{
    type Item = L::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (item, is_left) = match (&mut self.left, &mut self.right) {
            (Some(left), _) => (left.next(), true),
            (None, Some(right)) => (right.next(), false),
            (None, None) => return None,
        };

        match item {
            Some(item) => Some(item),
            None => {
                if is_left {
                    self.left = None;
                    self.next()
                } else {
                    self.right = None;
                    None
                }
            },
        }
    }

    #[track_caller]
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = self.len();
        (exact, Some(exact))
    }
}

impl<L, R> ExactSizeIterator for ChainExact<L, R>
where
    L: ExactSizeIterator,
    R: ExactSizeIterator<Item = L::Item>,
{
    #[track_caller]
    #[inline]
    fn len(&self) -> usize {
        self.left.as_ref().map_or(0, |iter| iter.len())
            + self.right.as_ref().map_or(0, |iter| iter.len())
    }
}

impl<'a, T> TryFromValue<NixList<Borrowed<'a>>> for Vec<T>
where
    T: TryFromValue<NixValue<Borrowed<'a>>>,
{
    #[inline]
    fn try_from_value(
        list: NixList<Borrowed<'a>>,
        ctx: &mut Context,
    ) -> Result<Self> {
        (0..list.len()).map(|idx| list.get::<T>(idx, ctx)).collect()
    }
}

impl<T> TryFromValue<NixList<Owned>> for Vec<T>
where
    T: TryFromValue<NixValue<Owned>>,
{
    #[inline]
    fn try_from_value(list: NixList<Owned>, ctx: &mut Context) -> Result<Self> {
        (0..list.len())
            .map(|idx| {
                list.get::<NixValue<_>>(idx, ctx)
                    .map(NixValue::into_owned)
                    .and_then(|value| T::try_from_value(value, ctx))
            })
            .collect()
    }
}
