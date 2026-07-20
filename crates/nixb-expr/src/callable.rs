//! TODO: docs.

use alloc::boxed::Box;
use core::ffi::c_void;
use core::mem::ManuallyDrop;
use core::ptr;

use nixb_error::Result;

use crate::attrset::NixAttrset;
use crate::context::Context;
use crate::error::TypeMismatchError;
use crate::thunk::NixThunk;
use crate::value::{
    IntoValue,
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
    fn call(&self, arg: impl IntoValue, ctx: &mut Context) -> NixThunk {
        let function_ptr = self.value_ptr();
        assert!(!function_ptr.is_null(), "Callable::value_ptr() returned null");

        let dest_val = ctx.alloc_value();
        let arg_val = ctx.alloc_value();

        arg.into_value(ctx).write(arg_val, ctx);

        // `nix_init_apply` errors only when one of its pointers is null, and
        // `UninitValue` together with the assertion above guard against that.
        unsafe {
            nixb_sys::init_apply(
                ptr::null_mut(),
                dest_val.as_ptr(),
                function_ptr,
                arg_val.as_ptr(),
            );
        }

        // Free the argument once we're done with it.
        let _ = ctx.with_raw(|ctx| unsafe {
            nixb_sys::value_decref(ctx, arg_val.as_ptr())
        });

        // SAFETY: `init_apply` has initialized the value at `dest_ptr`.
        let owner = unsafe { Owned::new(dest_val.as_non_null()) };

        NixValue::new(owner).into()
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
    ) -> NixThunk {
        struct CallMultiUserdata(Box<[*mut nixb_sys::Value]>);

        impl CallMultiUserdata {
            #[inline]
            fn args_mut(&mut self) -> &mut [*mut nixb_sys::Value] {
                &mut self.0[2..]
            }

            #[inline]
            unsafe fn from_userdata(userdata: *mut c_void) -> Self {
                let values = userdata.cast::<*mut nixb_sys::Value>();
                // SAFETY: `userdata` points to the allocation created by
                // `into_ptr`, whose first word stores the argument count.
                let num_args = unsafe { (*values).addr() };
                let values =
                    ptr::slice_from_raw_parts_mut(values, num_args + 2);
                // SAFETY: the slice metadata was recovered from the first
                // word, recreating the original Box allocation.
                Self(unsafe { Box::from_raw(values) })
            }

            #[inline]
            fn function_ptr(&self) -> *mut nixb_sys::Value {
                self.0[1]
            }

            #[inline]
            fn into_ptr(self) -> *mut c_void {
                let this = ManuallyDrop::new(self);
                // SAFETY: `this` will not be dropped, and this moves its Box
                // into the raw allocation managed by the thunk callbacks.
                let values = unsafe { ptr::read(&this.0) };
                Box::into_raw(values).cast::<*mut nixb_sys::Value>().cast()
            }

            #[inline]
            fn new(
                callable: &(impl Callable + ?Sized),
                args: impl Values,
                num_args: usize,
                ctx: &mut Context,
            ) -> Self {
                fn write_args<Args: Values>(
                    args: Args,
                    args_buf: &mut [*mut nixb_sys::Value],
                    num_allocated: &mut usize,
                    ctx: &mut Context,
                ) {
                    let Some((arg_ptr, rest_args_buf)) =
                        args_buf.split_first_mut()
                    else {
                        debug_assert!(args.is_empty());
                        return;
                    };
                    let (first_arg, rest_args) = args.split_first();
                    let dest = ctx.alloc_value();
                    *arg_ptr = dest.as_ptr();
                    *num_allocated += 1;
                    first_arg.into_value(ctx).write(dest, ctx);
                    write_args(rest_args, rest_args_buf, num_allocated, ctx);
                }

                debug_assert_eq!(args.len(), num_args);

                let mut values = {
                    let mut ptr_slice = Box::new_uninit_slice(num_args + 2);
                    for ptr in &mut ptr_slice {
                        ptr.write(ptr::null_mut());
                    }
                    // SAFETY: every slot was just initialized to null.
                    unsafe { ptr_slice.assume_init() }
                };

                values[0] = ptr::without_provenance_mut(0);

                values[1] = {
                    let function_ptr = callable.value_ptr();
                    assert!(
                        !function_ptr.is_null(),
                        "Callable::value_ptr() returned null"
                    );
                    ctx.with_raw(|ctx| unsafe {
                        nixb_sys::value_incref(ctx, function_ptr);
                    })
                    .unwrap_or_else(|err| {
                        panic!("failed to retain Nix function: {err}")
                    });
                    function_ptr
                };

                // Construct the owner before initializing any arguments so
                // its Drop impl cleans up the function and every allocated
                // argument in case we panic mid-initialization.
                let mut this = Self(values);

                let (num_allocated, values) =
                    this.0.split_first_mut().expect("never empty");
                // SAFETY: `usize` and a thin raw pointer have the same size
                // and alignment. This header word is accessed exclusively as
                // a `usize` for the duration of this mutable borrow.
                let num_allocated = unsafe {
                    &mut *ptr::from_mut(num_allocated).cast::<usize>()
                };
                write_args(args, &mut values[1..], num_allocated, ctx);
                debug_assert_eq!(*num_allocated, num_args);
                this
            }
        }

        impl Drop for CallMultiUserdata {
            #[inline]
            fn drop(&mut self) {
                let num_allocated = (self.0[0]).addr();
                for &value in &self.0[1..num_allocated + 2] {
                    unsafe { nixb_sys::value_decref(ptr::null_mut(), value) };
                }
            }
        }

        unsafe extern "C" fn on_force(
            ctx: *mut nixb_sys::c_context,
            state: *mut nixb_sys::EvalState,
            dest: *mut nixb_sys::Value,
            userdata: *mut c_void,
        ) {
            // SAFETY:
            // - `userdata` was created by `CallMultiUserdata::into_ptr`;
            // - `init_thunk` calls this callback at most once.
            let mut call =
                unsafe { CallMultiUserdata::from_userdata(userdata) };
            let function = call.function_ptr();
            let args = call.args_mut();

            unsafe {
                #[cfg(not(feature = "nix-2-34"))]
                nixb_cpp::value_call_multi(
                    ctx,
                    state,
                    function,
                    args.len(),
                    args.as_mut_ptr(),
                    dest,
                );

                #[cfg(feature = "nix-2-34")]
                nixb_sys::value_call_multi(
                    ctx,
                    state,
                    function,
                    args.len(),
                    args.as_mut_ptr(),
                    dest,
                );
            }
        }

        unsafe extern "C" fn on_drop(userdata: *mut c_void) {
            // SAFETY:
            // - `userdata` was created by `CallMultiUserdata::into_ptr`;
            // - `init_thunk` calls this callback only when the force callback
            //   is not called.
            let _: CallMultiUserdata =
                unsafe { CallMultiUserdata::from_userdata(userdata) };
        }

        let num_args = args.len();

        assert!(
            num_args >= 2,
            "Callable::call_multi() requires at least 2 arguments"
        );

        let userdata = CallMultiUserdata::new(self, args, num_args, ctx);
        let dest = ctx.alloc_value();
        let userdata_ptr = userdata.into_ptr();

        let init_res = ctx.with_raw_and_state(|ctx, state| unsafe {
            nixb_cpp::init_thunk(
                ctx,
                state.as_ptr(),
                dest.as_ptr(),
                userdata_ptr,
                on_force,
                on_drop,
            );
        });

        if let Err(err) = init_res {
            // SAFETY: initialization failed, so C++ did not take ownership of
            // `userdata`.
            unsafe { on_drop(userdata_ptr) };
            let _ = ctx.with_raw(|ctx| unsafe {
                nixb_sys::value_decref(ctx, dest.as_ptr())
            });
            panic!("failed to allocate Nix thunk: {err}")
        }

        // SAFETY: `init_thunk` has initialized the value at `dest`.
        let owner = unsafe { Owned::new(dest.as_non_null()) };

        NixValue::new(owner).into()
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
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        self.inner.write(dest, ctx)
    }
}

impl<Owner: ValueOwner> From<NixFunctor<Owner>> for NixAttrset<Owner> {
    #[inline]
    fn from(functor: NixFunctor<Owner>) -> Self {
        functor.inner
    }
}

impl<Owner: ValueOwner> From<NixFunctor<Owner>> for NixValue<Owner> {
    #[inline]
    fn from(functor: NixFunctor<Owner>) -> Self {
        functor.inner.into()
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
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        self.inner.write(dest, ctx)
    }
}

impl<Owner: ValueOwner> From<NixLambda<Owner>> for NixValue<Owner> {
    #[inline]
    fn from(lambda: NixLambda<Owner>) -> Self {
        lambda.inner
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
