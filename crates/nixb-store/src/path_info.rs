use core::ffi::{c_char, c_uint, c_void};
use core::mem::ManuallyDrop;
use core::num::NonZeroU64;
use core::ops::ControlFlow;
use core::slice;

use nixb_contexts::c_context::CContext;
use nixb_error::Result;

use crate::StorePath;

/// Metadata about a store path.
pub struct PathInfo {
    ctx: CContext,
    inner: *mut nixb_sys::path_info,
}

impl PathInfo {
    /// TODO: docs.
    #[inline]
    pub fn get_deriver(&mut self) -> Result<Option<StorePath>> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::path_info_get_deriver(ctx, self.inner)
            })
            .map(|ptr| (!ptr.is_null()).then(|| StorePath::new(ptr)))
    }

    /// TODO: docs.
    #[inline]
    pub fn get_nar_size(&mut self) -> Result<Option<NonZeroU64>> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::path_info_get_nar_size(ctx, self.inner)
            })
            .map(NonZeroU64::new)
    }

    /// TODO: docs.
    #[inline]
    pub fn with_ca<F, R>(&mut self, fun: F) -> Result<Option<R>>
    where
        F: FnOnce(&str) -> R,
    {
        struct CallbackState<F, R> {
            fun: Option<F>,
            result: Option<R>,
        }

        unsafe extern "C" fn callback<F, R>(
            start: *const c_char,
            n: c_uint,
            userdata: *mut c_void,
        ) where
            F: FnOnce(&str) -> R,
        {
            let state = unsafe { &mut *userdata.cast::<CallbackState<F, R>>() };
            let fun = state.fun.take().expect("callback is not called twice");
            let ca_bytes = unsafe {
                slice::from_raw_parts(start.cast::<u8>(), n as usize)
            };
            // SAFETY: the CA is always valid UTF-8.
            let ca = unsafe { core::str::from_utf8_unchecked(ca_bytes) };
            state.result = Some(fun(ca));
        }

        let mut state = CallbackState { fun: Some(fun), result: None };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::path_info_get_ca(
                ctx,
                self.inner,
                Some(callback::<F, R>),
                (&mut state as *mut CallbackState<F, R>).cast(),
            );
        })?;

        Ok(state.result)
    }

    /// Calls the given function with the NAR hash of the store path.
    ///
    /// The hash is passed as a string with a `sha256:` prefix followed by a Nix
    /// base-32 digest (e.g. `"sha256:1b8m03r63zqhnjf7l5nh..."`), which is the
    /// same format used in narinfo files.
    #[inline]
    pub fn with_nar_hash<F, R>(&mut self, fun: F) -> Result<R>
    where
        F: FnOnce(&str) -> R,
    {
        struct CallbackState<F, R> {
            fun: Option<F>,
            result: Option<R>,
        }

        unsafe extern "C" fn callback<F, R>(
            start: *const c_char,
            n: c_uint,
            userdata: *mut c_void,
        ) where
            F: FnOnce(&str) -> R,
        {
            let state = unsafe { &mut *userdata.cast::<CallbackState<F, R>>() };
            let fun = state.fun.take().expect("nar hash callback called twice");
            let nar_hash_bytes = unsafe {
                slice::from_raw_parts(start.cast::<u8>(), n as usize)
            };
            // SAFETY: a NAR hash is composed of a hash algorithm name plus a
            // Nix base-32 digest, both of which are valid UTF-8.
            let nar_hash =
                unsafe { core::str::from_utf8_unchecked(nar_hash_bytes) };
            state.result = Some(fun(nar_hash));
        }

        let mut state = CallbackState { fun: Some(fun), result: None };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::path_info_get_nar_hash(
                ctx,
                self.inner,
                Some(callback::<F, R>),
                (&mut state as *mut CallbackState<F, R>).cast(),
            )
        })?;

        Ok(state.result.expect("nar hash callback was not called"))
    }

    /// TODO: docs.
    #[inline]
    pub fn with_references<F, B>(&mut self, fun: F) -> Result<ControlFlow<B>>
    where
        F: FnMut(&StorePath) -> ControlFlow<B>,
    {
        const CALLBACK_EXIT: nixb_sys::err = nixb_sys::err_NIX_ERR_UNKNOWN;

        struct CallbackState<F, B> {
            fun: F,
            break_value: ControlFlow<B>,
        }

        unsafe extern "C" fn callback<F, B>(
            userdata: *mut c_void,
            store_path: *const nixb_sys::StorePath,
        ) -> nixb_sys::err
        where
            F: FnMut(&StorePath) -> ControlFlow<B>,
        {
            let state = unsafe { &mut *userdata.cast::<CallbackState<F, B>>() };
            let store_path = StorePath::new(store_path.cast_mut());
            match (state.fun)(&ManuallyDrop::new(store_path)) {
                ControlFlow::Continue(()) => nixb_sys::err_NIX_OK,
                break_value => {
                    state.break_value = break_value;
                    CALLBACK_EXIT
                },
            }
        }

        let mut state =
            CallbackState { fun, break_value: ControlFlow::Continue(()) };

        let ret = self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::path_info_get_references(
                ctx,
                self.inner,
                (&mut state as *mut CallbackState<F, B>).cast(),
                Some(callback::<F, B>),
            )
        })?;

        match ret {
            nixb_sys::err_NIX_OK => Ok(ControlFlow::Continue(())),
            CALLBACK_EXIT => Ok(state.break_value),
            other => unreachable!("unexpected callback status: {other}"),
        }
    }

    /// TODO: docs.
    #[inline]
    pub fn with_sigs<F, B>(&mut self, fun: F) -> Result<ControlFlow<B>>
    where
        F: FnMut(&[u8]) -> ControlFlow<B>,
    {
        const CALLBACK_EXIT: nixb_sys::err = nixb_sys::err_NIX_ERR_UNKNOWN;

        struct CallbackState<F, B> {
            fun: F,
            break_value: ControlFlow<B>,
        }

        unsafe extern "C" fn callback<F, B>(
            userdata: *mut c_void,
            sig: *const c_char,
            sig_len: c_uint,
        ) -> nixb_sys::err
        where
            F: FnMut(&[u8]) -> ControlFlow<B>,
        {
            let state = unsafe { &mut *userdata.cast::<CallbackState<F, B>>() };
            let sig = unsafe {
                slice::from_raw_parts(sig.cast::<u8>(), sig_len as usize)
            };
            match (state.fun)(sig) {
                ControlFlow::Continue(()) => nixb_sys::err_NIX_OK,
                break_value => {
                    state.break_value = break_value;
                    CALLBACK_EXIT
                },
            }
        }

        let mut state =
            CallbackState { fun, break_value: ControlFlow::Continue(()) };

        let ret = self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::path_info_get_sigs(
                ctx,
                self.inner,
                (&mut state as *mut CallbackState<F, B>).cast(),
                Some(callback::<F, B>),
            )
        })?;

        match ret {
            nixb_sys::err_NIX_OK => Ok(ControlFlow::Continue(())),
            CALLBACK_EXIT => Ok(state.break_value),
            other => unreachable!("unexpected callback status: {other}"),
        }
    }

    #[inline]
    pub(crate) fn new(ctx: CContext, inner: *mut nixb_sys::path_info) -> Self {
        assert!(!inner.is_null());
        Self { ctx, inner }
    }
}

impl Drop for PathInfo {
    fn drop(&mut self) {
        unsafe { nixb_sys::path_info_free(self.inner) };
    }
}
