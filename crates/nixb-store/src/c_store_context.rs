//! TODO: docs.

use core::ffi::CStr;
use core::ptr;

use nixb_contexts::c_context::CContext;

/// TODO: docs.
pub struct CStoreContext {
    ctx: CContext,
    store: *mut nixb_sys::Store,
}

/// TODO: docs.
pub struct InitSentinel {}

/// TODO: docs.
pub enum StoreParam {}

/// TODO: docs.
#[inline]
pub fn init<const LOAD_CONFIG: bool>(
    ctx: &mut CContext,
) -> nixb_result::Result<InitSentinel> {
    ctx.with_ptr(|ctx| {
        if LOAD_CONFIG {
            unsafe { nixb_sys::libstore_init(ctx) }
        } else {
            unsafe { nixb_sys::libstore_init_no_load_config(ctx) }
        }
    })?;
    Ok(InitSentinel {})
}

impl CStoreContext {
    /// TODO: docs.
    #[inline]
    pub fn get_fs_closure(
        &mut self,
        _store_path: (),
        _fun: (),
        _opts: (),
    ) -> nixb_result::Result<()> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_fs_closure(
                ctx,
                self.store,
                ptr::null_mut(),
                false,
                false,
                false,
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
        Ok(Self { ctx, store })
    }
}

impl Drop for CStoreContext {
    fn drop(&mut self) {
        unsafe { nixb_sys::store_free(self.store) };
    }
}
