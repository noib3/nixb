//! TODO: docs.

use core::ffi::CStr;
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
    pub fn get_fs_closure(
        &mut self,
        store_path: &StorePath,
        _fun: (),
        opts: GetFsClosureOpts,
    ) -> nixb_result::Result<()> {
        self.inner.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_fs_closure(
                ctx,
                self.store,
                store_path.inner,
                opts.flip_direction,
                opts.include_outputs,
                opts.include_derivers,
                ptr::null_mut(),
                None,
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
        Ok(Self { inner: ctx, store })
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
}

impl Drop for CStoreContext {
    fn drop(&mut self) {
        unsafe { nixb_sys::store_free(self.store) };
    }
}
