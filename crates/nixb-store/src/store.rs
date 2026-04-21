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
        mut fun: F,
        opts: GetFsClosureOpts,
    ) -> Result<()>
    where
        F: FnMut(&StorePath),
    {
        unsafe extern "C" fn callback<F>(
            _ctx: *mut nixb_sys::c_context,
            userdata: *mut c_void,
            store_path: *const nixb_sys::StorePath,
        ) where
            F: FnMut(&StorePath),
        {
            let fun = unsafe { &mut *userdata.cast::<F>() };
            let store_path = StorePath::new(store_path.cast_mut());
            (fun)(&store_path);
            mem::forget(store_path);
        }

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_fs_closure(
                ctx,
                self.inner,
                store_path.as_ptr(),
                opts.flip_direction,
                opts.include_outputs,
                opts.include_derivers,
                (&mut fun as *mut F).cast(),
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

impl Clone for Store {
    #[inline]
    fn clone(&self) -> Self {
        let store = unsafe { nixb_cpp::store_clone(self.inner) };
        assert!(!store.is_null());
        Self::new(CContext::create(), store)
    }
}

// SAFETY: Nix's C store wrapper owns a `nix::ref<nix::Store>` and Nix itself
// passes store refs to worker threads.
unsafe impl Send for Store {}

// SAFETY: `&Store` does not permit calling into Nix: the raw pointer is
// private, and all methods require `&mut self`.
unsafe impl Sync for Store {}

impl Drop for Store {
    fn drop(&mut self) {
        unsafe { nixb_sys::store_free(self.inner) };
    }
}
