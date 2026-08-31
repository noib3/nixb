//! Bindings to Nix's input fetcher settings API.

#![no_std]

use core::ptr::NonNull;

use nixb_c_context::CContext;
use nixb_error::Result;

/// Settings shared by Nix input fetchers.
pub struct FetchersSettings {
    _ctx: CContext,
    inner: NonNull<nixb_sys::fetchers_settings>,
}

impl FetchersSettings {
    /// Creates settings initialized with Nix's default values.
    #[inline]
    pub fn new() -> Result<Self> {
        let mut ctx = CContext::create();
        let settings = ctx
            .with_ptr(|ctx| unsafe { nixb_sys::fetchers_settings_new(ctx) })?;

        let inner = NonNull::new(settings).expect(
            "nix_fetchers_settings_new returned null without setting an error",
        );

        Ok(Self { _ctx: ctx, inner })
    }

    /// Returns the underlying C settings pointer.
    ///
    /// Only meant to be used by other `nixb-*` crates whose C libraries take
    /// a `nix_fetchers_settings *`.
    #[doc(hidden)]
    #[inline]
    pub fn as_ptr(&self) -> *mut nixb_sys::fetchers_settings {
        self.inner.as_ptr()
    }
}

impl Drop for FetchersSettings {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::fetchers_settings_free(self.inner.as_ptr()) };
    }
}
