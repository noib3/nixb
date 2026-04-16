//! TODO: docs.

use alloc::boxed::Box;
use core::ffi::{CStr, c_char, c_void};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::{any, mem};

use nixb_result::{Error, Result};

use crate::IntoResult;
use crate::context::{Context, EvalState};
use crate::value::{
    Borrowed,
    IntoValue,
    NixValue,
    TryFromValue,
    UninitValue,
    Value,
    ValueKind,
    ValueOwner,
};

/// TODO: docs.
pub trait Function {
    /// TODO: docs.
    #[cfg(nightly)]
    const NAME: &'static str = any::type_name::<Self>();

    /// TODO: docs.
    type Args<'args>: Args<'args>;

    /// TODO: docs.
    fn call<'this, 'args, 'eval>(
        &'this mut self,
        args: Self::Args<'args>,
        ctx: &mut Context<'eval>,
    ) -> impl IntoResult<Output: IntoValue, Error: Into<Error>>
    + use<'this, 'args, 'eval, Self>;

    /// TODO: docs.
    #[inline]
    fn into_value(self) -> impl Value + 'static
    where
        Self: Sized + 'static,
    {
        struct Wrapper<T>(T);

        impl<T: Function + 'static> Value for Wrapper<T> {
            #[inline]
            fn kind(&self) -> ValueKind {
                ValueKind::Function
            }

            #[inline]
            fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
                Function::write(self.0, dest, ctx)
            }
        }

        Wrapper(self)
    }

    #[doc(hidden)]
    #[inline]
    fn args_arity(&self) -> u8 {
        // We subtract 1 because the last element in the names slice is
        // always the null pointer, which doesn't count towards the arity.
        self.args_names().len() as u8 - 1
    }

    #[doc(hidden)]
    #[inline]
    fn args_names(&self) -> &'static [*const c_char] {
        Self::Args::NAMES
    }

    #[doc(hidden)]
    #[inline]
    fn callback() -> nixb_cpp::FunctionCallback
    where
        Self: Sized + 'static,
    {
        unsafe extern "C" fn callback<Fun: Function + 'static>(
            userdata: *mut c_void,
            ctx_raw: *mut nixb_sys::c_context,
            state_raw: *mut nixb_sys::EvalState,
            args_raw: *mut *mut nixb_sys::Value,
            dest_raw: *mut nixb_sys::Value,
        ) {
            let Some(ctx_ptr) = NonNull::new(ctx_raw) else {
                panic!("received NULL `nix_c_context` pointer in primop call");
            };

            let Some(state_ptr) = NonNull::new(state_raw) else {
                panic!("received NULL `EvalState` pointer in primop call");
            };

            let fun = Fun::from_userdata(userdata);

            #[cfg(not(feature = "nix-2-33"))]
            let Some(args_ptr) = NonNull::new(args_raw) else {
                panic!("received NULL args pointer in primop call");
            };

            #[cfg(feature = "nix-2-33")]
            let args_ptr = match NonNull::new(args_raw) {
                Some(args_ptr) => args_ptr,
                None if fun.args_arity() == 0 => NonNull::dangling(),
                None => panic!("received NULL args pointer in primop call"),
            };

            let Some(dest_ptr) = NonNull::new(dest_raw) else {
                panic!("received NULL `Value` pointer in primop call");
            };

            let mut ctx = Context::new(ctx_ptr, EvalState::new(state_ptr));

            let args_list = ArgsList { args_ptr, _lifetime: PhantomData };

            // SAFETY: Nix guarantees the destination pointer to point to an
            // uninitialized value.
            let dest = unsafe { UninitValue::new(dest_ptr) };

            let mut try_block = || {
                let args = <Fun as Function>::Args::from_args_list(
                    args_list, &mut ctx,
                )?;

                let mut value = fun
                    .call(args, &mut ctx)
                    .into_result()
                    .map_err(Into::into)?
                    .into_value(&mut ctx);

                // As described in the [docs] of `nix_init_apply`, it's not
                // possible to return thunks from primops, so let's force the
                // value before writing it to the return location.
                //
                // [docs]: https://github.com/NixOS/nix/blob/af0ac14/src/libexpr-c/nix_api_value.h#L564
                value.force_inline(&mut ctx)?;

                value.write(dest, &mut ctx)
            };

            if let Err(err) = try_block() {
                unsafe {
                    nixb_sys::set_err_msg(
                        ctx_raw,
                        err.kind().code(),
                        err.message().as_ptr(),
                    );
                }
            }
        }

        callback::<Self>
    }

    #[doc(hidden)]
    #[inline]
    fn from_userdata<'any>(userdata: *mut c_void) -> &'any mut Self
    where
        Self: Sized + 'static,
    {
        if mem::size_of::<Self>() == 0 {
            // SAFETY: for zero-sized types we can construct a mutable
            // reference from a dangling pointer.
            unsafe { &mut *(ptr::NonNull::<Self>::dangling().as_ptr()) }
        } else {
            // SAFETY: userdata is a `*mut Self` created by `Box::into_raw`.
            unsafe { &mut *userdata.cast::<Self>() }
        }
    }

    #[doc(hidden)]
    #[inline]
    fn into_userdata(self) -> *mut c_void
    where
        Self: Sized + 'static,
    {
        if mem::size_of::<Self>() == 0 {
            ptr::null_mut()
        } else {
            Box::into_raw(Box::new(self)).cast()
        }
    }

    #[doc(hidden)]
    #[inline]
    fn on_drop() -> extern "C" fn(userdata: *mut c_void)
    where
        Self: Sized + 'static,
    {
        extern "C" fn on_drop<This>(userdata: *mut c_void) {
            if mem::size_of::<This>() > 0 {
                // SAFETY:
                // - userdata is a `*mut Self` created by `Box::into_raw`;
                // - this is only called once on error or via the finalizer.
                let _: Box<This> =
                    unsafe { Box::from_raw(userdata.cast::<This>()) };
            }
        }
        on_drop::<Self>
    }

    #[doc(hidden)]
    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()>
    where
        Self: Sized + 'static,
    {
        #[cfg(nightly)]
        let fun_name = Self::NAME;
        #[cfg(not(nightly))]
        let fun_name = any::type_name::<Self>();

        let args_arity = self.args_arity();
        let args_names = self.args_names();
        let userdata = self.into_userdata();

        let init_res = ctx.with_raw_and_state(|ctx, state| unsafe {
            nixb_cpp::init_function(
                ctx,
                state.as_ptr(),
                dest.as_ptr(),
                fun_name.as_ptr().cast(),
                fun_name.len(),
                args_arity.into(),
                args_names.as_ptr(),
                userdata,
                Self::callback(),
                Self::on_drop(),
            );
        });

        if init_res.is_err() {
            Self::on_drop()(userdata);
        }

        init_res
    }
}

/// TODO: docs.
pub trait Arg<'a>: TryFromValue<NixValue<Borrowed<'a>>> {
    /// TODO: docs.
    const NAME: &'static CStr;
}

/// TODO: docs.
pub trait Args<'a>: Sized {
    /// A slice containing pointers to the names of the arguments, terminated
    /// by a trailing null pointer.
    #[doc(hidden)]
    const NAMES: &'static [*const c_char];

    #[doc(hidden)]
    fn from_args_list(
        args_list: ArgsList<'a>,
        ctx: &mut Context,
    ) -> Result<Self>;
}

/// TODO: docs.
#[inline]
#[allow(clippy::too_many_lines)]
pub fn function<'a, A: Args<'a>>(
    fun: impl FnMutOutputIntoResult<A> + 'static,
) -> impl Function + Value + 'static {
    type CallFn<F> =
        for<'args, 'eval> fn(
            &mut F,
            ArgsList<'args>,
            &mut Context<'eval>,
        ) -> Result<NixValue<crate::value::Owned>>;

    struct Wrapper<F> {
        args_names: &'static [*const c_char],
        call: CallFn<F>,
        fun: F,
    }

    fn call_impl<'fixed, 'args, 'eval, A, F>(
        fun: &mut F,
        args_list: ArgsList<'args>,
        ctx: &mut Context<'eval>,
    ) -> Result<NixValue<crate::value::Owned>>
    where
        A: Args<'fixed>,
        F: FnMutOutputIntoResult<A> + 'static,
    {
        // `function()` binds the decoded argument type to the construction
        // lifetime, so we rebind the raw arguments before decoding them.
        let args_list = unsafe {
            mem::transmute::<ArgsList<'args>, ArgsList<'fixed>>(args_list)
        };

        let args = A::from_args_list(args_list, ctx)?;

        let mut value =
            fun.call(args).into_result().map_err(Into::into)?.into_value(ctx);

        value.force_inline(ctx)?;

        let dest = ctx.alloc_value()?;
        value.write(dest, ctx)?;

        // SAFETY: `write` initialized the allocated destination value.
        let owner = unsafe { crate::value::Owned::new(dest.as_non_null()) };

        Ok(NixValue::new(owner))
    }

    impl<F: 'static> Function for Wrapper<F> {
        type Args<'args> = ArgsList<'args>;

        #[inline]
        fn call<'this, 'eval, 'args>(
            &'this mut self,
            args: Self::Args<'args>,
            ctx: &mut Context<'eval>,
        ) -> impl IntoResult<Output: IntoValue, Error: Into<Error>>
        + use<'this, 'args, 'eval, F> {
            (self.call)(&mut self.fun, args, ctx)
        }

        #[inline]
        fn args_names(&self) -> &'static [*const c_char] {
            self.args_names
        }
    }

    impl<F> Value for Wrapper<F>
    where
        Self: Function + 'static,
    {
        #[inline]
        fn kind(&self) -> ValueKind {
            ValueKind::Function
        }

        #[inline]
        fn write(self, dest: UninitValue, ctx: &mut Context) -> Result<()> {
            Function::write(self, dest, ctx)
        }
    }

    Wrapper {
        args_names: A::NAMES,
        call: |fun, args_list, ctx| call_impl(fun, args_list, ctx),
        fun,
    }
}

/// TODO: docs.
#[derive(Debug, Copy, Clone)]
#[doc(hidden)]
pub struct ArgsList<'a> {
    args_ptr: NonNull<*mut nixb_sys::Value>,
    _lifetime: PhantomData<&'a [*mut nixb_sys::Value]>,
}

#[doc(hidden)]
pub struct NoArgs;

impl<'a> ArgsList<'a> {
    /// Returns the value at the given argument index.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given index is not out of bounds.
    #[inline]
    pub(crate) unsafe fn get(self, arg_idx: u8) -> NixValue<Borrowed<'a>> {
        let arg_raw = unsafe { *self.args_ptr.as_ptr().offset(arg_idx.into()) };
        let Some(arg_ptr) = NonNull::new(arg_raw) else {
            panic!("argument at index {arg_idx} is null");
        };
        // SAFETY: the argument list comes from a primop callback, so every
        // argument has been initialized.
        let owner = unsafe { Borrowed::new(arg_ptr) };
        NixValue::new(owner)
    }
}

impl<'spec> Args<'spec> for ArgsList<'spec> {
    // The only Function impl using ArgsList as their Args is the one returned
    // by `function()`, which overrides `args_names`.
    const NAMES: &'static [*const c_char] = unreachable!();

    #[inline]
    fn from_args_list(args_list: Self, _: &mut Context) -> Result<Self> {
        Ok(args_list)
    }
}

impl<T: IntoValue + Clone> Function for T {
    type Args<'a> = NoArgs;

    #[inline]
    fn call<'this, 'eval, 'args>(
        &'this mut self,
        _: NoArgs,
        ctx: &mut Context<'eval>,
    ) -> impl Value + use<'this, 'args, 'eval, T> {
        self.clone().into_value(ctx)
    }
}

impl<'a, A: Arg<'a>> Args<'a> for A {
    const NAMES: &'static [*const c_char] = &[Self::NAME.as_ptr(), ptr::null()];

    #[inline]
    fn from_args_list(
        args_list: ArgsList<'a>,
        ctx: &mut Context,
    ) -> Result<Self> {
        A::try_from_value(unsafe { args_list.get(0) }, ctx)
    }
}

impl Args<'_> for NoArgs {
    const NAMES: &'static [*const c_char] = &[ptr::null()];

    #[inline]
    fn from_args_list(_: ArgsList<'_>, _: &mut Context) -> Result<Self> {
        Ok(Self)
    }
}

/// TODO: docs.
pub trait FnMutOutputIntoResult<Args> {
    /// TODO: docs.
    type Output: IntoResult<Output: IntoValue, Error: Into<Error>>;

    /// TODO: docs.
    fn call(&mut self, args: Args) -> Self::Output;
}

impl<F, A, R> FnMutOutputIntoResult<A> for F
where
    F: FnMut(A) -> R,
    R: IntoResult<Output: IntoValue, Error: Into<Error>>,
{
    type Output = R;

    #[inline]
    fn call(&mut self, args: A) -> Self::Output {
        (self)(args)
    }
}

macro_rules! impl_args_for_tuple {
    ($($T:ident),+) => {
        impl_args_for_tuple!(@pair [] [0 1 2 3 4 5 6 7] [$($T)+]);
    };

    (@pair [$($pairs:tt)+] [$($rest_idx:tt)*] []) => {
        impl_args_for_tuple!(@final [$($pairs)+]);
    };

    (@pair [$($pairs:tt)*] [$next_idx:tt $($rest_idx:tt)*] [$next_T:ident $($rest_T:ident)*]) => {
        impl_args_for_tuple!(@pair [$($pairs)* ($next_idx $next_T)] [$($rest_idx)*] [$($rest_T)*]);
    };

    (@final [$(($idx:tt $T:ident))+]) => {
        impl<'a, $($T: Arg<'a>),+> Args<'a> for ($($T,)+) {
            const NAMES: &'static [*const c_char] = &[
                $($T::NAME.as_ptr(),)+
                ptr::null()
            ];

            #[inline]
            fn from_args_list(
                args_list: ArgsList<'a>,
                ctx: &mut Context,
            ) -> Result<Self> {
                Ok((
                    $($T::try_from_value(unsafe { args_list.get($idx) }, ctx)?,)+
                ))
            }
        }
    };
}

// NOTE: we only implement `Args` for tuples of up to 8 elements because that's
// the maximum arity of a Nix primitive operation, which is also used as the
// maximum number of arguments allowed by our `init_function` binding.
//
// See [this][source] for more infos.
//
// [source]: https://github.com/NixOS/nix/blob/2.32.2/src/libexpr/include/nix/expr/eval.hh#L33-L38

impl_args_for_tuple!(A1);
impl_args_for_tuple!(A1, A2);
impl_args_for_tuple!(A1, A2, A3);
impl_args_for_tuple!(A1, A2, A3, A4);
impl_args_for_tuple!(A1, A2, A3, A4, A5);
impl_args_for_tuple!(A1, A2, A3, A4, A5, A6);
impl_args_for_tuple!(A1, A2, A3, A4, A5, A6, A7);
impl_args_for_tuple!(A1, A2, A3, A4, A5, A6, A7, A8);
