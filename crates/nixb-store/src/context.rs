//! TODO: docs.

use core::ffi::{CStr, c_void};
use core::ptr;

use nixb_contexts::c_context::CContext;

use crate::{GetFsClosureOpts, InitSentinel, StoreParam, StorePath};

/// TODO: docs.
pub struct CStoreContext {
    inner: CContext,
    store: *mut nixb_sys::Store,
}

impl CStoreContext {
    /// TODO: docs.
    #[inline]
    pub fn get_fs_closure<F>(
        &mut self,
        store_path: &StorePath,
        fun: F,
        opts: GetFsClosureOpts,
    ) -> nixb_result::Result<()>
    where
        F: FnMut(&mut Self, &StorePath),
    {
        struct CallbackState<F> {
            store: *mut nixb_sys::Store,
            fun: F,
        }

        unsafe extern "C" fn callback<F>(
            ctx: *mut nixb_sys::c_context,
            userdata: *mut c_void,
            store_path: *const nixb_sys::StorePath,
        ) where
            F: FnMut(&mut CStoreContext, &StorePath),
        {
            let ctx = CContext::new(ctx);
            let state = unsafe { &mut *userdata.cast::<CallbackState<F>>() };
            let mut store_ctx = CStoreContext::new(ctx, state.store);
            let store_path = StorePath { inner: store_path.cast_mut() };
            (state.fun)(&mut store_ctx, &store_path);
        }

        let mut state = CallbackState { store: self.store, fun };

        self.inner.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_fs_closure(
                ctx,
                self.store,
                store_path.inner,
                opts.flip_direction,
                opts.include_outputs,
                opts.include_derivers,
                (&mut state as *mut CallbackState<_>).cast(),
                Some(callback::<F>),
            );
        })
    }

    /// TODO: docs.
    #[inline]
    pub fn open(
        _: InitSentinel,
        uri: impl AsRef<CStr>,
        _params: impl IntoIterator<Item = StoreParam>,
        mut ctx: CContext,
    ) -> nixb_result::Result<Self> {
        let store = ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_open(ctx, uri.as_ref().as_ptr(), ptr::null_mut())
        })?;
        Ok(Self::new(ctx, store))
    }

    /// TODO: docs.
    #[inline]
    pub fn parse_path(
        &mut self,
        store_path: impl AsRef<CStr>,
    ) -> nixb_result::Result<StorePath> {
        self.inner
            .with_ptr(|ctx| unsafe {
                nixb_sys::store_parse_path(
                    ctx,
                    self.store,
                    store_path.as_ref().as_ptr(),
                )
            })
            .map(|path_ptr| StorePath { inner: path_ptr })
    }

    #[inline]
    fn new(inner: CContext, store: *mut nixb_sys::Store) -> Self {
        Self { inner, store }
    }
}

impl Drop for CStoreContext {
    fn drop(&mut self) {
        unsafe { nixb_sys::store_free(self.store) };
    }
}
