//! TODO: docs.

use alloc::vec::Vec;
use core::ptr;

use crate::attrset::NixAttrset;
use crate::context::Context;
use crate::error::{Result, TypeMismatchError};
use crate::thunk::NixThunk;
use crate::value::{
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

/// TODO: docs.
pub trait Callable {
    /// TODO: docs.
    fn value_ptr(&self) -> *mut nixb_sys::Value;

    /// TODO: docs.
    #[inline]
    fn call(&self, arg: impl IntoValue, ctx: &mut Context) -> Result<NixThunk> {
        let dest_val = ctx.alloc_value()?;
        let arg_val = ctx.alloc_value()?;

        let res = arg.into_value(ctx).write(arg_val, ctx).and_then(|()| {
            ctx.with_raw(|ctx| {
                unsafe {
                    nixb_sys::init_apply(
                        ctx,
                        dest_val.as_ptr(),
                        self.value_ptr(),
                        arg_val.as_ptr(),
                    )
                };
            })
        });

        // Free the argument once we're done with it.
        ctx.with_raw(|ctx| unsafe {
            nixb_sys::value_decref(ctx, arg_val.as_ptr())
        })
        .ok();

        // Free the destination value if the call failed.
        if let Err(err) = res {
            ctx.with_raw(|ctx| unsafe {
                nixb_sys::value_decref(ctx, dest_val.as_ptr())
            })
            .ok();
            return Err(err);
        }

        // SAFETY: `init_apply` has initialized the value at `dest_ptr`.
        let owner = unsafe { Owned::new(dest_val.as_non_null()) };

        Ok(NixValue::new(owner).into())
    }

    /// TODO: docs.
    ///
    /// # Panics
    ///
    /// Panics if the number of arguments is less than 2.
    #[inline]
    #[track_caller]
    #[expect(clippy::too_many_lines)]
    fn call_multi<Args: Values>(
        &self,
        args: Args,
        ctx: &mut Context,
    ) -> Result<NixThunk> {
        assert!(
            Args::LEN >= 2,
            "Callable::call_multi() requires at least 2 arguments"
        );

        #[cfg(nightly)]
        fn new_args_array<V: Values>(
            _: &V,
        ) -> impl AsMut<[*mut nixb_sys::Value]> + use<V> {
            core::array::from_fn::<_, V::LEN, _>(|_| ptr::null_mut())
        }

        #[cfg(not(nightly))]
        fn new_args_array<V: Values>(
            _: &V,
        ) -> impl AsMut<[*mut nixb_sys::Value]> + use<V> {
            (0..V::LEN).map(|_| ptr::null_mut()).collect::<Vec<_>>()
        }

        fn init_args_array<Args: Values>(
            args: Args,
            slice: &mut [*mut nixb_sys::Value],
            num_written: &mut usize,
            ctx: &mut Context,
        ) -> Result<()> {
            debug_assert_eq!(Args::LEN, slice.len());
            let Some((first_ptr, rest_slice)) = slice.split_first_mut() else {
                return Ok(());
            };
            let (first_arg, rest_args) = args.split_first();
            let dest = ctx.alloc_value()?;
            first_arg.into_value(ctx).write(dest, ctx)?;
            *first_ptr = dest.as_ptr();
            *num_written += 1;
            init_args_array(
                rest_args.into_values(),
                rest_slice,
                num_written,
                ctx,
            )
        }

        // We'll do an eager call with the first N - 1 arguments, followed by
        // a lazy call with the last argument.
        let (args, last) = args.split_last();
        let args = args.into_values();

        let mut args_array = new_args_array(&args);
        let mut num_written = 0;

        let args_slice = &mut args_array.as_mut()[..];

        let dest = ctx.alloc_value()?;

        let res = init_args_array(args, args_slice, &mut num_written, ctx)
            .and_then(|()| {
                ctx.with_raw_and_state(|ctx, state| unsafe {
                    #[cfg(not(feature = "nix-2-34"))]
                    nixb_cpp::value_call_multi(
                        ctx,
                        state.as_ptr(),
                        self.value_ptr(),
                        args_slice.len(),
                        args_slice.as_mut_ptr(),
                        dest.as_ptr(),
                    );

                    #[cfg(feature = "nix-2-34")]
                    nixb_sys::value_call_multi(
                        ctx,
                        state.as_ptr(),
                        self.value_ptr(),
                        args_slice.len(),
                        args_slice.as_mut_ptr(),
                        dest.as_ptr(),
                    );
                })
            });

        // Free the arguments once we're done with them.
        for &raw_arg in &args_slice[..num_written] {
            ctx.with_raw(|ctx| unsafe { nixb_sys::value_decref(ctx, raw_arg) })
                .ok();
        }

        // Free the destination value if the call failed.
        if let Err(err) = res {
            ctx.with_raw(|ctx| unsafe {
                nixb_sys::value_decref(ctx, dest.as_ptr())
            })
            .ok();
            return Err(err);
        }

        // SAFETY: `value_call_multi` has initialized the value at `dest_ptr`.
        let owner = unsafe { Owned::new(dest.as_non_null()) };

        NixLambda::try_from_value(NixValue::new(owner), ctx)?.call(last, ctx)
    }
}

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct NixFunctor<Owner = Owned> {
    inner: NixAttrset<Owner>,
}

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct NixLambda<Owner = Owned> {
    inner: NixValue<Owner>,
}

impl<Owner: ValueOwner> NixFunctor<Owner> {
    /// TODO: docs.
    #[inline]
    pub fn borrow(&self) -> NixFunctor<Owner::Borrow<'_>> {
        NixFunctor { inner: self.inner.borrow() }
    }

    /// TODO: docs.
    #[inline]
    pub fn into_owned(self) -> NixFunctor<Owned> {
        NixFunctor { inner: self.inner.into_owned() }
    }
}

impl<Owner: ValueOwner> NixLambda<Owner> {
    /// TODO: docs.
    #[inline]
    pub fn borrow(&self) -> NixLambda<Owner::Borrow<'_>> {
        NixLambda { inner: self.inner.borrow() }
    }

    /// TODO: docs.
    #[inline]
    pub fn into_owned(self) -> NixLambda<Owned> {
        NixLambda { inner: self.inner.into_owned() }
    }
}

impl<Owner: ValueOwner> Callable for NixFunctor<Owner> {
    #[inline]
    fn value_ptr(&self) -> *mut nixb_sys::Value {
        NixValue::from(self.inner.borrow()).as_ptr()
    }
}

impl<Owner: ValueOwner> TryFromValue<NixValue<Owner>> for NixFunctor<Owner> {
    #[inline]
    fn try_from_value(
        value: NixValue<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        NixAttrset::try_from_value(value, ctx)
            .and_then(|attrset| Self::try_from_value(attrset, ctx))
    }
}

impl<Owner: ValueOwner> TryFromValue<NixAttrset<Owner>> for NixFunctor<Owner> {
    #[inline]
    fn try_from_value(
        attrset: NixAttrset<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        let value = attrset.get::<NixValue<_>>(c"__functor", ctx)?;
        let value_kind = value.kind();
        drop(value);
        match value_kind {
            // We also accept thunks to avoid eagerly forcing functors. If the
            // __functor doesn't evaluates to a function, the user will get an
            // error when calling 'Callable::call{_multi}()'.
            ValueKind::Function | ValueKind::Thunk => {
                Ok(Self { inner: attrset })
            },
            other => Err(TypeMismatchError {
                expected: ValueKind::Function,
                found: other,
            }
            .into()),
        }
    }
}

impl<Owner: ValueOwner> Value for NixFunctor<Owner> {
    #[inline]
    fn kind(&self) -> ValueKind {
        self.inner.kind()
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        self.inner.write(dest, ctx)
    }
}

impl<Owner: ValueOwner> Callable for NixLambda<Owner> {
    #[inline]
    fn value_ptr(&self) -> *mut nixb_sys::Value {
        self.inner.as_ptr()
    }
}

impl<Owner: ValueOwner> TryFromValue<NixValue<Owner>> for NixLambda<Owner> {
    #[inline]
    fn try_from_value(
        mut value: NixValue<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        value.force_inline(ctx)?;

        match value.kind() {
            ValueKind::Function => Ok(Self { inner: value }),
            other => Err(TypeMismatchError {
                expected: ValueKind::Function,
                found: other,
            }
            .into()),
        }
    }
}

impl<Owner: ValueOwner> Value for NixLambda<Owner> {
    #[inline]
    fn kind(&self) -> ValueKind {
        self.inner.kind()
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        self.inner.write(dest, ctx)
    }
}

#[cfg(feature = "either")]
impl<L, R> Callable for either::Either<L, R>
where
    L: Callable,
    R: Callable,
{
    #[inline]
    fn value_ptr(&self) -> *mut nixb_sys::Value {
        match self {
            Self::Left(l) => l.value_ptr(),
            Self::Right(r) => r.value_ptr(),
        }
    }
}
