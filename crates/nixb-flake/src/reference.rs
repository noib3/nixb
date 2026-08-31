use core::ffi::{c_char, c_uint, c_void};
use core::ptr::{self, NonNull};

use nixb_c_context::CContext;
use nixb_error::Result;
use nixb_expr::context::Context;
use nixb_fetchers::FetchersSettings;

use crate::settings::FlakeSettings;
use crate::{FlakeLockFlags, LockedFlake};

/// A reference describing how to obtain a flake or raw source tree.
pub struct FlakeReference {
    _ctx: CContext,
    inner: NonNull<nixb_sys::flake_reference>,
}

/// Parameters for parsing a [`FlakeReference`].
pub struct FlakeReferenceParseFlags {
    ctx: CContext,
    inner: NonNull<nixb_sys::flake_reference_parse_flags>,
}

impl FlakeReference {
    /// Locks a flake reference using the given evaluator state.
    #[expect(clippy::too_many_arguments)]
    #[inline]
    pub fn lock(
        &self,
        fetchers: &FetchersSettings,
        settings: &FlakeSettings,
        flags: &FlakeLockFlags,
        ctx: &mut Context<'_>,
    ) -> Result<LockedFlake> {
        let locked_flake = ctx.with_raw_and_state(|raw_ctx, state| unsafe {
            nixb_sys::flake_lock(
                raw_ctx,
                fetchers.as_ptr(),
                settings.as_ptr(),
                state,
                flags.as_ptr(),
                self.as_ptr(),
            )
        })?;

        let ptr = NonNull::new(locked_flake)
            .expect("nix_flake_lock returned null without setting an error");

        Ok(LockedFlake::new(ptr))
    }

    /// Parses a URL-like flake reference and passes its fragment to a callback.
    #[expect(clippy::too_many_arguments)]
    #[inline]
    pub fn parse_with_fragment<T, F>(
        fetchers: &FetchersSettings,
        settings: &FlakeSettings,
        flags: &FlakeReferenceParseFlags,
        reference: &str,
        on_fragment: F,
    ) -> Result<(Self, T)>
    where
        F: FnOnce(&str) -> T,
    {
        struct CallbackState<F, T> {
            fun: Option<F>,
            ret: Option<T>,
        }

        unsafe extern "C" fn callback<F, T>(
            start: *const c_char,
            n: c_uint,
            user_data: *mut c_void,
        ) where
            F: FnOnce(&str) -> T,
        {
            let bytes = unsafe {
                core::slice::from_raw_parts(start.cast::<u8>(), n as usize)
            };
            let fragment = unsafe { core::str::from_utf8_unchecked(bytes) };
            let state =
                unsafe { &mut *user_data.cast::<CallbackState<F, T>>() };
            let fun = state.fun.take().expect("callback is called once");
            state.ret = Some(fun(fragment));
        }

        let mut ctx = CContext::create();
        let mut reference_out = ptr::null_mut();
        let mut callback_state =
            CallbackState { fun: Some(on_fragment), ret: None };

        ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_reference_and_fragment_from_string(
                ctx,
                fetchers.as_ptr(),
                settings.as_ptr(),
                flags.as_ptr(),
                reference.as_ptr().cast(),
                reference.len(),
                &mut reference_out,
                Some(callback::<F, T>),
                (&mut callback_state as *mut CallbackState<F, T>).cast(),
            )
        })?;

        let inner = NonNull::new(reference_out).expect(
            "nix_flake_reference_and_fragment_from_string returned null \
             without setting an error",
        );
        let fragment =
            callback_state.ret.expect("fragment callback was called");

        Ok((Self { _ctx: ctx, inner }, fragment))
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut nixb_sys::flake_reference {
        self.inner.as_ptr()
    }
}

impl FlakeReferenceParseFlags {
    /// Creates parsing parameters initialized with Nix's default values.
    #[inline]
    pub fn new(settings: &FlakeSettings) -> Result<Self> {
        let mut ctx = CContext::create();
        let flags = ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_reference_parse_flags_new(ctx, settings.as_ptr())
        })?;

        let inner = NonNull::new(flags).expect(
            "nix_flake_reference_parse_flags_new returned null without \
             setting an error",
        );

        Ok(Self { ctx, inner })
    }

    /// Sets the base directory used to resolve relative flake references.
    #[inline]
    pub fn set_base_directory(&mut self, dir: &str) -> Result<&mut Self> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_reference_parse_flags_set_base_directory(
                ctx,
                self.inner.as_ptr(),
                dir.as_ptr().cast(),
                dir.len(),
            )
        })?;

        Ok(self)
    }

    #[inline]
    fn as_ptr(&self) -> *mut nixb_sys::flake_reference_parse_flags {
        self.inner.as_ptr()
    }
}

impl Drop for FlakeReference {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::flake_reference_free(self.inner.as_ptr()) };
    }
}

impl Drop for FlakeReferenceParseFlags {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            nixb_sys::flake_reference_parse_flags_free(self.inner.as_ptr())
        };
    }
}
