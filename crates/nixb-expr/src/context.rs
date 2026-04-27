//! TODO: docs.

use core::ffi::CStr;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};

use nixb_c_context::CContext;
use nixb_error::{Error, ErrorKind, Result};

use crate::attrset::NixAttrset;
use crate::builtins::Builtins;
use crate::value::{
    Borrowed,
    NixValue,
    Owned,
    TryFromValue,
    UninitValue,
    ValueOwner,
};

/// TODO: docs.
pub struct Context<'state, State = EvalState<'state>> {
    inner: CContext,
    state: State,
    _lifetime: PhantomData<&'state ()>,
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

impl<'eval> Context<'eval, EvalState<'eval>> {
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
        let dest = self.alloc_value()?;

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

    /// Creates a new [`AttrsetBuilder`] with the given capacity.
    #[inline]
    pub(crate) fn make_attrset_builder(
        &mut self,
        capacity: usize,
    ) -> Result<AttrsetBuilder<'_, 'eval>> {
        unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            let builder_ptr =
                nixb_cpp::make_bindings_builder(self.state.as_ptr(), capacity);

            #[cfg(feature = "nix-2-34")]
            let builder_ptr = self.inner.with_ptr(|ptr| {
                nixb_sys::make_bindings_builder(
                    ptr,
                    self.state.as_ptr(),
                    capacity,
                )
            })?;

            match NonNull::new(builder_ptr) {
                Some(builder_ptr) => {
                    Ok(AttrsetBuilder { inner: builder_ptr, context: self })
                },
                None => Err(Error::new(
                    ErrorKind::Overflow,
                    c"failed to create AttrsetBuilder",
                )),
            }
        }
    }

    /// Creates a new [`ListBuilder`] with the given capacity.
    #[inline]
    pub(crate) fn make_list_builder(
        &mut self,
        capacity: usize,
    ) -> Result<ListBuilder<'_, 'eval>> {
        unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            let builder_ptr =
                nixb_cpp::make_list_builder(self.state.as_ptr(), capacity);

            #[cfg(feature = "nix-2-34")]
            let builder_ptr = self.inner.with_ptr(|ptr| {
                nixb_sys::make_list_builder(ptr, self.state.as_ptr(), capacity)
            })?;

            match NonNull::new(builder_ptr) {
                Some(builder_ptr) => Ok(ListBuilder {
                    inner: builder_ptr,
                    context: self,
                    index: 0,
                }),
                None => Err(Error::new(
                    ErrorKind::Overflow,
                    c"failed to create ListBuilder",
                )),
            }
        }
    }

    #[inline]
    pub(crate) fn state_ptr(&mut self) -> *mut nixb_sys::EvalState {
        self.state.as_ptr()
    }
}

impl<State> Context<'_, State> {
    /// TODO: docs.
    #[inline]
    pub fn with_raw<T>(
        &mut self,
        fun: impl FnOnce(*mut nixb_sys::c_context) -> T,
    ) -> Result<T> {
        self.inner.with_ptr(fun)
    }

    #[inline]
    #[doc(hidden)]
    pub fn new(inner: CContext, state: State) -> Self {
        Self { inner, state, _lifetime: PhantomData }
    }

    #[inline]
    pub(crate) fn into_inner(self) -> CContext {
        self.inner
    }

    /// TODO: docs.
    #[inline]
    pub(crate) fn with_raw_and_state<T>(
        &mut self,
        fun: impl FnOnce(*mut nixb_sys::c_context, &mut State) -> T,
    ) -> Result<T> {
        self.inner.with_ptr(|raw_ctx| fun(raw_ctx, &mut self.state))
    }
}

impl<'eval> EvalState<'eval> {
    /// Allocates a new, uninitialized value, returning a pointer to it.
    ///
    /// The caller is responsible for freeing the value by calling
    /// [`nixb_sys::value_decref`] once it is no longer needed.
    #[inline]
    pub(crate) fn alloc_value(&mut self) -> Result<UninitValue> {
        #[cfg(not(feature = "nix-2-34"))]
        let raw_ptr = unsafe { nixb_cpp::alloc_value(self.inner.as_ptr()) };

        #[cfg(feature = "nix-2-34")]
        let raw_ptr = unsafe {
            nixb_sys::alloc_value(ptr::null_mut(), self.inner.as_ptr())
        };

        let Some(non_null_ptr) = NonNull::new(raw_ptr) else {
            return Err(Error::new(
                ErrorKind::Overflow,
                c"failed to allocate Value",
            ));
        };

        // SAFETY: `alloc_value` returns a pointer to an uninitialized value.
        Ok(unsafe { UninitValue::new(non_null_ptr) })
    }

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
    pub(crate) fn build(self, dest: UninitValue) -> Result<()> {
        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::make_attrs(dest.as_ptr(), self.inner.as_ptr());
            Ok(())
        }

        #[cfg(feature = "nix-2-34")]
        self.context.with_raw(|ctx| unsafe {
            nixb_sys::make_attrs(ctx, dest.as_ptr(), self.inner.as_ptr());
        })
    }

    #[inline]
    pub(crate) fn insert(
        &mut self,
        key: &CStr,
        write_value: impl FnOnce(UninitValue, &mut Context) -> Result<()>,
    ) -> Result<()> {
        let dest = self.context.alloc_value()?;

        write_value(dest, self.context)?;

        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::bindings_builder_insert(
                self.inner.as_ptr(),
                key.as_ptr(),
                dest.as_ptr(),
            );

            Ok(())
        }

        #[cfg(feature = "nix-2-34")]
        self.context.with_raw(|ctx| unsafe {
            nixb_sys::bindings_builder_insert(
                ctx,
                self.inner.as_ptr(),
                key.as_ptr(),
                dest.as_ptr(),
            );
        })
    }
}

impl<'eval> ListBuilder<'_, 'eval> {
    #[inline]
    pub(crate) fn build(self, dest: UninitValue) -> Result<()> {
        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::make_list(dest.as_ptr(), self.inner.as_ptr());
            Ok(())
        }

        #[cfg(feature = "nix-2-34")]
        self.context.with_raw(|ctx| unsafe {
            nixb_sys::make_list(ctx, self.inner.as_ptr(), dest.as_ptr());
        })
    }

    #[inline]
    pub(crate) fn insert(
        &mut self,
        write_value: impl FnOnce(UninitValue, &mut Context) -> Result<()>,
    ) -> Result<()> {
        let dest = self.context.alloc_value()?;
        write_value(dest, self.context)?;

        #[cfg(not(feature = "nix-2-34"))]
        unsafe {
            nixb_cpp::list_builder_insert(
                self.inner.as_ptr(),
                self.index,
                dest.as_ptr(),
            );
        }

        #[cfg(feature = "nix-2-34")]
        self.context.with_raw(|ctx| unsafe {
            nixb_sys::list_builder_insert(
                ctx,
                self.inner.as_ptr(),
                self.index.try_into().unwrap_or(u32::MAX),
                dest.as_ptr(),
            )
        })?;

        self.index += 1;
        Ok(())
    }
}

impl<'state, State> Deref for Context<'state, State> {
    type Target = State;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl<'state, State> DerefMut for Context<'state, State> {
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
