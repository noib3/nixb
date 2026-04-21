//! TODO: docs.

use core::ffi::{CStr, c_void};
use core::{mem, ptr};

use nixb_contexts::c_context::CContext;
use nixb_error::Result;

use crate::{GetFsClosureOpts, InitSentinel, StoreParam, StorePath};

/// TODO: docs.
pub struct Store {
    ctx: CContext,
    inner: *mut nixb_sys::Store,
}

impl Store {
    /// TODO: docs.
    #[inline]
    pub fn get_fs_closure<F>(
        &mut self,
        store_path: &StorePath,
        fun: F,
        opts: GetFsClosureOpts,
    ) -> Result<()>
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
            F: FnMut(&mut Store, &StorePath),
        {
            let state = unsafe { &mut *userdata.cast::<CallbackState<F>>() };
            let mut this = Store::new(CContext::new(ctx), state.store);
            let store_path = StorePath::new(store_path.cast_mut());
            (state.fun)(&mut this, &store_path);
            mem::forget(this);
            mem::forget(store_path);
        }

        let mut state = CallbackState { store: self.inner, fun };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_fs_closure(
                ctx,
                self.inner,
                store_path.as_ptr(),
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
    ) -> Result<Self> {
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
    ) -> Result<StorePath> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::store_parse_path(
                    ctx,
                    self.inner,
                    store_path.as_ref().as_ptr(),
                )
            })
            .map(StorePath::new)
    }

    #[inline]
    fn new(ctx: CContext, store: *mut nixb_sys::Store) -> Self {
        Self { ctx, inner: store }
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe { nixb_sys::store_free(self.inner) };
    }
}
