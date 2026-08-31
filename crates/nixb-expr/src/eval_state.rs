//! Evaluator state ownership and construction.

use alloc::vec::Vec;
use core::ffi::CStr;
use core::ptr;
use core::ptr::NonNull;

use nixb_c_context::CContext;
use nixb_error::Result;
use nixb_store::Store;

use crate::context::Context;
use crate::init::InitSentinel;

/// An evaluator state.
pub struct EvalState {
    inner: NonNull<nixb_sys::EvalState>,
}

/// A builder for configuring and creating an [`EvalState`].
pub struct EvalStateBuilder {
    ctx: CContext,
    inner: NonNull<nixb_sys::eval_state_builder>,
}

impl EvalState {
    /// Creates a [`Context`] borrowing this state.
    #[inline]
    pub fn context(&mut self) -> Context<'_> {
        Context::new(CContext::create(), unsafe { self.inner.as_mut() })
    }

    /// Creates a new evaluator state.
    ///
    /// Settings are read from the ambient environment (environment variables
    /// and configuration files), except for the lookup path used by `<...>`
    /// expressions, which is always set to the given entries.
    #[inline]
    pub fn new<'a>(
        mut init: InitSentinel,
        lookup_path: impl IntoIterator<Item = &'a CStr>,
        store: &mut Store,
    ) -> Result<Self> {
        let mut entries = lookup_path
            .into_iter()
            .map(|entry| entry.as_ptr())
            .collect::<Vec<_>>();

        let entries_ptr = if entries.is_empty() {
            ptr::null_mut()
        } else {
            entries.push(ptr::null());
            entries.as_mut_ptr()
        };

        let state = init.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::state_create(ctx, entries_ptr, store.as_ptr())
        })?;

        let inner = NonNull::new(state)
            .expect("nix_state_create returned null without setting an error");

        Ok(Self { inner })
    }
}

impl EvalStateBuilder {
    /// Returns the underlying C builder pointer.
    ///
    /// Only meant to be used by other `nixb-*` crates whose C libraries take
    /// a `nix_eval_state_builder *`.
    #[doc(hidden)]
    #[inline]
    pub fn as_ptr(&mut self) -> *mut nixb_sys::eval_state_builder {
        self.inner.as_ptr()
    }

    /// Creates an owned evaluator state, consuming the builder.
    #[inline]
    pub fn build(mut self) -> Result<EvalState> {
        let state = self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::eval_state_build(ctx, self.inner.as_ptr())
        })?;

        let inner = NonNull::new(state).expect(
            "nix_eval_state_build returned null without setting an error",
        );

        Ok(EvalState { inner })
    }

    /// Reads settings from environment variables and configuration files.
    #[inline]
    pub fn load(&mut self) -> Result<&mut Self> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::eval_state_builder_load(ctx, self.inner.as_ptr())
        })?;

        Ok(self)
    }

    /// Creates a builder with default evaluator settings.
    #[inline]
    pub fn new(mut init: InitSentinel, store: &mut Store) -> Result<Self> {
        let builder = init.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::eval_state_builder_new(ctx, store.as_ptr())
        })?;

        let inner = NonNull::new(builder).expect(
            "nix_eval_state_builder_new returned null without setting an error",
        );

        Ok(Self { ctx: init.ctx, inner })
    }

    /// Sets the lookup path used by `<...>` expressions.
    #[inline]
    pub fn set_lookup_path<'a>(
        &mut self,
        entries: impl IntoIterator<Item = &'a CStr>,
    ) -> Result<&mut Self> {
        let mut entries =
            entries.into_iter().map(|entry| entry.as_ptr()).collect::<Vec<_>>();

        let entries_ptr = if entries.is_empty() {
            ptr::null_mut()
        } else {
            entries.push(ptr::null());
            entries.as_mut_ptr()
        };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::eval_state_builder_set_lookup_path(
                ctx,
                self.inner.as_ptr(),
                entries_ptr,
            )
        })?;

        Ok(self)
    }
}

impl Drop for EvalState {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::state_free(self.inner.as_ptr()) }
    }
}

impl Drop for EvalStateBuilder {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::eval_state_builder_free(self.inner.as_ptr()) };
    }
}
