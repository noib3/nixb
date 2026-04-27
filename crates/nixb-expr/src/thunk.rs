//! TODO: docs.

use alloc::boxed::Box;
use core::ffi::c_void;
use core::mem;
use core::ptr::NonNull;

use nixb_c_context::CContext;
use nixb_error::{Error, Result};

use crate::context::{Context, EvalState};
use crate::into_result::IntoResult;
use crate::value::{
    IntoValue,
    NixValue,
    Owned,
    TryFromValue,
    UninitValue,
    Value,
    ValueKind,
    ValueOwner,
};

/// TODO: docs.
pub trait Thunk {
    /// TODO: docs.
    type Output;

    /// TODO: docs.
    fn force(self, ctx: &mut Context) -> Result<Self::Output>;

    /// TODO: docs.
    #[inline]
    fn into_value(self) -> impl Thunk + Value + 'static
    where
        Self: Sized + 'static,
        Self::Output: IntoValue,
    {
        struct Wrapper<T>(T);

        impl<T: Thunk> Thunk for Wrapper<T> {
            type Output = T::Output;

            fn force(self, ctx: &mut Context) -> Result<Self::Output> {
                self.0.force(ctx)
            }
        }

        impl<T: Thunk<Output: IntoValue> + 'static> Value for Wrapper<T> {
            #[inline]
            fn kind(&self) -> ValueKind {
                ValueKind::Thunk
            }

            #[inline]
            fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
                Thunk::write(self.0, dest, ctx)
            }
        }

        Wrapper(self)
    }

    #[doc(hidden)]
    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()>
    where
        Self: Sized + 'static,
        Self::Output: IntoValue,
    {
        unsafe extern "C" fn on_force<Th>(
            ctx_raw: *mut nixb_sys::c_context,
            state_raw: *mut nixb_sys::EvalState,
            dest_raw: *mut nixb_sys::Value,
            userdata: *mut c_void,
        ) where
            Th: Thunk + 'static,
            Th::Output: IntoValue,
        {
            let c_context = CContext::new(ctx_raw);
            let Some(state) = NonNull::new(state_raw) else {
                panic!("received NULL `EvalState` pointer in thunk force");
            };
            let Some(dest) = NonNull::new(dest_raw) else {
                panic!("received NULL `Value` pointer in thunk force");
            };
            // SAFETY: dest is guaranteed to be a valid pointer to an
            // uninitialized Value.
            let dest = unsafe { UninitValue::new(dest) };

            let mut ctx = Context::new(c_context, EvalState::new(state));

            // SAFETY:
            // - userdata is a `*mut Th` created by `Box::into_raw`;
            // - `init_thunk`'s API contract guarantees that this function is
            //   only called once;
            let thunk = unsafe { Box::from_raw(userdata.cast::<Th>()) };

            let result = thunk.force(&mut ctx).and_then(|output| {
                output.into_value(&mut ctx).write(dest, &mut ctx)
            });

            if let Err(err) = result {
                unsafe {
                    nixb_sys::set_err_msg(
                        ctx_raw,
                        err.kind().code(),
                        err.message().as_ptr(),
                    );
                }
            }

            mem::forget(ctx.into_inner());
        }

        unsafe extern "C" fn on_drop<Th>(userdata: *mut c_void) {
            // SAFETY:
            // - userdata is a `*mut Th` created by `Box::into_raw`;
            // - `init_thunk`'s API contract guarantees that this function is
            //   only called once, and only if `on_force` was not called;
            let _: Box<Th> = unsafe { Box::from_raw(userdata.cast::<Th>()) };
        }

        ctx.with_raw_and_state(|ctx, state| unsafe {
            nixb_cpp::init_thunk(
                ctx,
                state.as_ptr(),
                dest.as_ptr(),
                Box::into_raw(Box::new(self)).cast(),
                on_force::<Self>,
                on_drop::<Self>,
            );
        })
    }
}

/// TODO: docs.
#[derive(Debug, Copy, Clone)]
pub struct NixThunk<Owner = Owned> {
    value: NixValue<Owner>,
}

/// TODO: docs.
pub fn thunk<F, MaybeRes>(fun: F) -> impl Thunk + Value + 'static
where
    F: FnOnce() -> MaybeRes + 'static,
    MaybeRes: IntoResult,
    MaybeRes::Output: IntoValue,
    MaybeRes::Error: Into<Error>,
{
    Thunk::into_value(fun)
}

impl<Owner: ValueOwner> NixThunk<Owner> {
    /// TODO: docs.
    #[inline]
    pub fn borrow(&self) -> NixThunk<Owner::Borrow<'_>> {
        NixThunk { value: self.value.borrow() }
    }

    /// TODO: docs.
    #[inline(always)]
    pub fn force_into<T>(mut self, ctx: &mut Context) -> Result<T>
    where
        T: TryFromValue<NixValue<Owner>>,
    {
        self.value.force_inline(ctx)?;
        T::try_from_value(self.value, ctx)
    }

    /// TODO: docs.
    #[inline]
    pub fn into_inner(self) -> NixValue<Owner> {
        self.value
    }

    /// TODO: docs.
    #[inline]
    pub fn into_owned(self) -> NixThunk<Owned> {
        NixThunk { value: self.value.into_owned() }
    }
}

impl<Owner: ValueOwner> Value for NixThunk<Owner> {
    #[inline]
    fn force_inline(&mut self, ctx: &mut Context) -> Result<()> {
        self.value.force_inline(ctx)
    }

    #[inline]
    fn kind(&self) -> ValueKind {
        self.value.kind()
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
        self.value.write(dest, ctx)
    }
}

impl<Owner: ValueOwner> From<NixValue<Owner>> for NixThunk<Owner> {
    #[inline]
    fn from(value: NixValue<Owner>) -> Self {
        Self { value }
    }
}

impl<Owner: ValueOwner> TryFromValue<NixValue<Owner>> for NixThunk<Owner> {
    #[inline]
    fn try_from_value(value: NixValue<Owner>, _: &mut Context) -> Result<Self> {
        Ok(value.into())
    }
}

impl<Owner: ValueOwner> Thunk for NixThunk<Owner> {
    type Output = NixValue<Owner>;

    #[inline]
    fn force(mut self, ctx: &mut Context) -> Result<Self::Output> {
        self.value.force_inline(ctx)?;
        Ok(self.value)
    }

    #[inline]
    fn into_value(self) -> impl Thunk + Value + 'static
    where
        Self: Sized + 'static,
    {
        self
    }
}

impl<F, MaybeRes> Thunk for F
where
    F: FnOnce() -> MaybeRes,
    MaybeRes: IntoResult,
    MaybeRes::Error: Into<Error>,
{
    type Output = MaybeRes::Output;

    #[inline(always)]
    fn force(self, _ctx: &mut Context) -> Result<Self::Output> {
        (self)().into_result().map_err(Into::into)
    }
}

#[cfg(feature = "either")]
impl<L, R> Thunk for either::Either<L, R>
where
    L: Thunk,
    R: Thunk<Output = L::Output>,
{
    type Output = L::Output;

    #[inline]
    fn force(self, ctx: &mut Context) -> Result<Self::Output> {
        match self {
            Self::Left(l) => l.force(ctx),
            Self::Right(r) => r.force(ctx),
        }
    }
}
