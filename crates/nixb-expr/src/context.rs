//! TODO: docs.

use core::ffi::{CStr, c_uint};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::NonNull;

use nixb_c_context::CContext;
use nixb_error::Result;

use crate::attrset::NixAttrset;
use crate::builtins::Builtins;
use crate::value::{
    Borrowed,
    NixValue,
    Owned,
    TryFromValue,
    UninitValue,
    Value,
    ValueOwner,
};

/// TODO: docs.
pub struct Context<'eval> {
    inner: CContext,
    state: EvalState<'eval>,
}

/// TODO: docs.
pub struct EvalState<'a> {
    inner: NonNull<nixb_sys::EvalState>,
    _lifetime: PhantomData<&'a nixb_sys::EvalState>,
}

pub(crate) struct AttrsetBuilder<'ctx, 'eval> {
    inner: NonNull<nixb_sys::BindingsBuilder>,
    context: &'ctx mut Context<'eval>,
}

pub(crate) struct ListBuilder<'ctx, 'eval> {
    inner: NonNull<nixb_sys::ListBuilder>,
    context: &'ctx mut Context<'eval>,
    index: usize,
}

impl<'eval> Context<'eval> {
    /// Returns the global `builtins` attribute set.
    ///
    /// This provides access to all built-in functions like `fetchGit`,
    /// `fetchurl`, `toString`, etc.
    #[inline]
    pub fn builtins(&mut self) -> Builtins<'eval> {
        let builtins_raw = unsafe { nixb_cpp::get_builtins(self.state_ptr()) };

        let Some(builtins_ptr) = NonNull::new(builtins_raw) else {
            panic!("failed to get builtins attrset: got null pointer");
        };

        // SAFETY: the value returned by `get_builtins` is initialized.
        let owner = unsafe { Borrowed::new(builtins_ptr) };

        match NixAttrset::try_from_value(NixValue::new(owner), self) {
            Ok(attrset) => Builtins::new(attrset),
            Err(err) => unreachable!("builtins is not an attrset: {err}"),
        }
    }

    /// TODO: docs.
    #[inline]
    pub fn eval<T>(&mut self, expr: &CStr) -> Result<T>
    where
        T: TryFromValue<NixValue>,
    {
        let dest = self.alloc_value();

        self.with_raw_and_state(|raw_ctx, state| unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            nixb_cpp::expr_eval_from_string(
                raw_ctx,
                state.as_ptr(),
                expr.as_ptr(),
                c".".as_ptr(),
                dest.as_ptr(),
            );

            #[cfg(feature = "nix-2-34")]
            nixb_sys::expr_eval_from_string(
                raw_ctx,
                state.as_ptr(),
                expr.as_ptr(),
                c".".as_ptr(),
                dest.as_ptr(),
            );
        })?;

        // SAFETY: `expr_eval_from_string` has initialized the value.
        let owner = unsafe { Owned::new(dest.as_non_null()) };

        T::try_from_value(NixValue::new(owner), self)
    }

    /// TODO: docs.
    #[inline]
    pub fn new_value(&mut self, value: impl Value) -> Result<NixValue> {
        let dest = self.alloc_value();

        if let Err(err) = value.write(dest, self) {
            let _ = self.with_raw(|ctx| unsafe {
                nixb_sys::value_decref(ctx, dest.as_ptr())
            });

            return Err(err);
        }

        // SAFETY: `dest` points to an initialized Nix value, and this creates
        // the unique `Owned` handle responsible for releasing that value.
        let owner = unsafe { Owned::new(dest.as_non_null()) };

        Ok(NixValue::new(owner))
    }

    /// Allocates a new, uninitialized value, returning a pointer to it.
    ///
    /// The caller is responsible for freeing the value by calling
    /// [`nixb_sys::value_decref`] once it is no longer needed.
    #[inline]
    pub(crate) fn alloc_value(&mut self) -> UninitValue {
        #[cfg(not(feature = "nix-2-34"))]
        let raw_ptr = unsafe { nixb_cpp::alloc_value(self.state.as_ptr()) };

        #[cfg(feature = "nix-2-34")]
        let raw_ptr = self
            .inner
            .with_ptr(|ctx| unsafe {
                nixb_sys::alloc_value(ctx, self.state.as_ptr())
            })
            .unwrap_or_else(|err| {
                panic!("failed to allocate Nix value: {err}")
            });

        let non_null_ptr =
            NonNull::new(raw_ptr).expect("failed to allocate Nix value");

        // SAFETY: `alloc_value` returns a pointer to an uninitialized value.
        unsafe { UninitValue::new(non_null_ptr) }
    }

    #[inline]
    pub(crate) fn into_inner(self) -> CContext {
        self.inner
    }

    /// Creates a new [`AttrsetBuilder`] with the given capacity.
    #[inline]
    pub(crate) fn make_attrset_builder(
        &mut self,
        capacity: c_uint,
    ) -> AttrsetBuilder<'_, 'eval> {
        unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            let builder_ptr = nixb_cpp::make_bindings_builder(
                self.state.as_ptr(),
                capacity as _,
            );

            #[cfg(feature = "nix-2-34")]
            let builder_ptr = self
                .inner
                .with_ptr(|ptr| {
                    nixb_sys::make_bindings_builder(
                        ptr,
                        self.state.as_ptr(),
                        capacity as _,
                    )
                })
                .unwrap_or_else(|err| {
                    panic!("failed to allocate attrset builder: {err}")
                });

            let builder_ptr = NonNull::new(builder_ptr).expect(
                "nix_make_bindings_builder returned null without setting an \
                 error",
            );

            AttrsetBuilder { inner: builder_ptr, context: self }
        }
    }

    /// Creates a new [`ListBuilder`] with the given capacity.
    #[inline]
    pub(crate) fn make_list_builder(
        &mut self,
        capacity: c_uint,
    ) -> ListBuilder<'_, 'eval> {
        unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            let builder_ptr =
                nixb_cpp::make_list_builder(self.state.as_ptr(), capacity as _);

            #[cfg(feature = "nix-2-34")]
            let builder_ptr = self
                .inner
                .with_ptr(|ptr| {
                    nixb_sys::make_list_builder(
                        ptr,
                        self.state.as_ptr(),
                        capacity as _,
                    )
                })
                .unwrap_or_else(|err| {
                    panic!("failed to allocate list builder: {err}")
                });

            let builder_ptr = NonNull::new(builder_ptr).expect(
                "nix_make_list_builder returned null without setting an error",
            );

            ListBuilder { inner: builder_ptr, context: self, index: 0 }
        }
    }

    #[inline]
    pub(crate) fn new(inner: CContext, state: EvalState<'eval>) -> Self {
        Self { inner, state }
    }

    #[inline]
    pub(crate) fn state_ptr(&mut self) -> *mut nixb_sys::EvalState {
        self.state.as_ptr()
    }

    #[inline]
    pub(crate) fn with_raw<T>(
        &mut self,
        fun: impl FnOnce(*mut nixb_sys::c_context) -> T,
    ) -> Result<T> {
        self.inner.with_ptr(fun)
    }

    #[inline]
    pub(crate) fn with_raw_and_state<T>(
        &mut self,
        fun: impl FnOnce(*mut nixb_sys::c_context, &mut EvalState<'eval>) -> T,
    ) -> Result<T> {
        self.inner.with_ptr(|raw_ctx| fun(raw_ctx, &mut self.state))
    }
}

impl<'eval> EvalState<'eval> {
    #[inline]
    pub(crate) fn as_ptr(&mut self) -> *mut nixb_sys::EvalState {
        self.inner.as_ptr()
    }

    #[inline]
    pub(crate) fn new(inner: NonNull<nixb_sys::EvalState>) -> Self {
        Self { inner, _lifetime: PhantomData }
    }
}

impl<'eval> AttrsetBuilder<'_, 'eval> {
    #[inline]
    pub(crate) fn build(self, dest: UninitValue) {
        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::make_attrs(dest.as_ptr(), self.inner.as_ptr());
        }

        // `nix_make_attrs` errors when:
        //
        // 1. the destination pointer is null;
        // 2. the destination value is already initialized;
        // 3. sorting the bindings throws an exception.
        //
        // Having an `UninitValue` guards against 1) and 2). The builder's
        // bindings contain trivially movable `Attr`s whose comparison only
        // compares `Symbol`s, which is `noexcept`, so 3) cannot happen.
        #[cfg(feature = "nix-2-34")]
        unsafe {
            nixb_sys::make_attrs(
                core::ptr::null_mut(),
                dest.as_ptr(),
                self.inner.as_ptr(),
            );
        }
    }

    #[inline]
    pub(crate) fn insert(
        &mut self,
        key: &CStr,
        write_value: impl FnOnce(UninitValue, &mut Context) -> Result<()>,
    ) -> Result<()> {
        assert!(
            key.to_bytes().len() < u32::MAX as usize,
            "attribute name exceeds Nix's 4 GiB limit",
        );

        let dest = self.context.alloc_value();

        write_value(dest, self.context)?;

        self.context
            .with_raw(|ctx| unsafe {
                #[cfg(not(feature = "nix-2-34"))]
                nixb_cpp::bindings_builder_insert(
                    ctx,
                    self.inner.as_ptr(),
                    key.as_ptr(),
                    dest.as_ptr(),
                );

                #[cfg(feature = "nix-2-34")]
                nixb_sys::bindings_builder_insert(
                    ctx,
                    self.inner.as_ptr(),
                    key.as_ptr(),
                    dest.as_ptr(),
                );
            })
            .unwrap_or_else(|err| {
                panic!("failed to intern attribute name: {err}")
            });

        Ok(())
    }
}

impl<'eval> ListBuilder<'_, 'eval> {
    #[inline]
    pub(crate) fn build(self, dest: UninitValue) {
        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::make_list(dest.as_ptr(), self.inner.as_ptr());
        }

        // `nix_make_list` errors when:
        //
        // 1. the destination pointer is null;
        // 2. the destination value is already initialized.
        //
        // Having an `UninitValue` guards against both, so neither can happen.
        #[cfg(feature = "nix-2-34")]
        unsafe {
            nixb_sys::make_list(
                core::ptr::null_mut(),
                self.inner.as_ptr(),
                dest.as_ptr(),
            );
        }
    }

    #[inline]
    pub(crate) fn insert(
        &mut self,
        write_value: impl FnOnce(UninitValue, &mut Context) -> Result<()>,
    ) -> Result<()> {
        let dest = self.context.alloc_value();
        write_value(dest, self.context)?;

        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::list_builder_insert(
                self.inner.as_ptr(),
                self.index,
                dest.as_ptr(),
            );
        }

        // `nix_list_builder_insert` only errors when the value pointer is
        // null. Having an `UninitValue` guards against that, so it cannot
        // happen.
        #[cfg(feature = "nix-2-34")]
        unsafe {
            nixb_sys::list_builder_insert(
                core::ptr::null_mut(),
                self.inner.as_ptr(),
                self.index.try_into().unwrap_or(u32::MAX),
                dest.as_ptr(),
            );
        }

        self.index += 1;
        Ok(())
    }
}

impl<'eval> Deref for Context<'eval> {
    type Target = EvalState<'eval>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'eval> DerefMut for Context<'eval> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl<'eval> AsMut<Context<'eval>> for AttrsetBuilder<'_, 'eval> {
    #[inline]
    fn as_mut(&mut self) -> &mut Context<'eval> {
        self.context
    }
}

impl<'eval> AsMut<Context<'eval>> for ListBuilder<'_, 'eval> {
    #[inline]
    fn as_mut(&mut self) -> &mut Context<'eval> {
        self.context
    }
}

impl Drop for AttrsetBuilder<'_, '_> {
    #[inline]
    fn drop(&mut self) {
        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::bindings_builder_free(self.inner.as_ptr());
        }

        #[cfg(feature = "nix-2-34")]
        unsafe {
            nixb_sys::bindings_builder_free(self.inner.as_ptr());
        }
    }
}

impl Drop for ListBuilder<'_, '_> {
    #[inline]
    fn drop(&mut self) {
        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::list_builder_free(self.inner.as_ptr());
        }

        #[cfg(feature = "nix-2-34")]
        unsafe {
            nixb_sys::list_builder_free(self.inner.as_ptr());
        }
    }
}
