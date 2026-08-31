use alloc::vec::Vec;
use core::ffi::CStr;
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

use nixb_c_context::CContext;
use nixb_error::Result;
use nixb_store::Store;

use crate::context::Context;
use crate::init::InitSentinel;

/// An owned evaluator state.
pub struct EvalState {
    inner: NonNull<nixb_sys::EvalState>,
}

/// A borrowed evaluator state.
///
/// This is a view into an evaluator state owned by someone else: either the
/// host Nix process when running as a plugin, or an owned [`EvalState`] in
/// embedded programs.
pub struct EvalStateRef<'a> {
    inner: NonNull<nixb_sys::EvalState>,
    _lifetime: PhantomData<&'a nixb_sys::EvalState>,
}

impl EvalState {
    /// Creates a [`Context`] borrowing this state.
    #[inline]
    pub fn context(&mut self) -> Context<'_> {
        Context::new(CContext::create(), self.as_ref())
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

    #[inline]
    pub(crate) fn as_ref(&self) -> EvalStateRef<'_> {
        EvalStateRef::new(self.inner)
    }
}

impl<'eval> EvalStateRef<'eval> {
    #[inline]
    pub(crate) fn as_ptr(&mut self) -> *mut nixb_sys::EvalState {
        self.inner.as_ptr()
    }

    #[inline]
    pub(crate) fn new(inner: NonNull<nixb_sys::EvalState>) -> Self {
        Self { inner, _lifetime: PhantomData }
    }
}

impl Drop for EvalState {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::state_free(self.inner.as_ptr()) };
    }
}
