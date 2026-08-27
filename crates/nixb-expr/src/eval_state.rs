use alloc::vec::Vec;
use core::ffi::CStr;
use core::ptr::{self, NonNull};

use nixb_c_context::CContext;
use nixb_error::Result;
use nixb_store::Store;

use crate::context::{Context, EvalStateRef};
use crate::init::InitSentinel;

/// An owned evaluator state.
///
/// Unlike [`EvalStateRef`], which borrows a state owned by the host Nix
/// process, this is a state created (and eventually freed) by us, allowing
/// standalone executables that embed Nix to evaluate expressions without
/// being loaded as a plugin.
pub struct EvalState {
    inner: NonNull<nixb_sys::EvalState>,
}

impl EvalState {
    /// Creates a [`Context`] borrowing this state, scoped exactly like the
    /// contexts provided to plugin callbacks.
    ///
    /// This is what unlocks the rest of this crate's API ([`eval`], attrsets,
    /// functions, ...) for embedded programs.
    ///
    /// [`eval`]: Context::eval
    #[inline]
    pub fn context(&mut self) -> Context<'_> {
        Context::new(CContext::create(), EvalStateRef::new(self.inner))
    }

    /// Creates a new evaluator state by calling `nix_state_create`, the C
    /// API's builder-less constructor.
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

impl Drop for EvalState {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::state_free(self.inner.as_ptr()) };
    }
}
