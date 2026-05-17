//! TODO: docs.

use core::ffi::{CStr, c_void};
use core::{mem, ptr};

use nixb_c_context::CContext;
use nixb_error::Result;

use crate::{
    GetFsClosureOpts,
    InitSentinel,
    NixDerivation,
    StoreParam,
    StorePath,
};

/// TODO: docs.
pub struct Store {
    ctx: CContext,
    inner: *mut nixb_sys::Store,
}

impl Store {
    /// Adds the given derivation to this store.
    #[inline]
    pub fn add_derivation(
        &mut self,
        derivation: NixDerivation,
    ) -> Result<StorePath> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::add_derivation(ctx, self.inner, derivation.as_ptr())
            })
            .map(StorePath::new)
    }

    /// Copies the closure of the given [`StorePath`] from this store to the
    /// destination store.
    #[inline]
    pub fn copy_closure(
        &mut self,
        dest: &mut Self,
        store_path: &StorePath,
    ) -> Result<()> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::store_copy_closure(
                    ctx,
                    self.inner,
                    dest.inner,
                    store_path.as_ptr(),
                )
            })
            .map(|_err| ())
    }

    /// Copies the the given [`StorePath`] from this store to the destination
    /// store.
    #[expect(clippy::too_many_arguments)]
    #[inline]
    pub fn copy_path(
        &mut self,
        dest: &mut Self,
        store_path: &StorePath,
        should_repair: bool,
        should_check_sigs: bool,
    ) -> Result<()> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::store_copy_path(
                    ctx,
                    self.inner,
                    dest.inner,
                    store_path.as_ptr(),
                    should_repair,
                    should_check_sigs,
                )
            })
            .map(|_err| ())
    }

    /// Creates a [`NixDerivation`] from a JSON representation of that
    /// derivation.
    #[inline]
    pub fn derivation_from_json(
        &mut self,
        json: impl AsRef<CStr>,
    ) -> Result<NixDerivation> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::derivation_from_json(
                    ctx,
                    self.inner,
                    json.as_ref().as_ptr(),
                )
            })
            .map(NixDerivation::new)
    }

    /// TODO: docs.
    #[inline]
    pub fn derivation_from_store_path(
        &mut self,
        store_path: &StorePath,
    ) -> Result<NixDerivation> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::store_drv_from_store_path(
                    ctx,
                    self.inner,
                    store_path.as_ptr(),
                )
            })
            .map(NixDerivation::new)
    }

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

    /// Checks if the given [`StorePath`] is valid (i.e. whether a corresponding
    /// store object and its closure of references exist in this store).
    #[inline]
    pub fn is_valid_path(&mut self, store_path: &StorePath) -> Result<bool> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_is_valid_path(ctx, self.inner, store_path.as_ptr())
        })
    }

    /// TODO: docs.
    #[inline]
    pub fn open(
        init: InitSentinel,
        uri: impl AsRef<CStr>,
        _params: impl IntoIterator<Item = StoreParam>,
    ) -> Result<Self> {
        let mut ctx = init.ctx;
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

    /// Queries the full store path given the hash part of a valid store path.
    /// Returns `Ok(None)` if the hash is valid but no matching path is found.
    #[inline]
    pub fn query_path_from_hash_part(
        &mut self,
        hash: impl AsRef<CStr>,
    ) -> Result<Option<StorePath>> {
        self.ctx
            .with_ptr(|ctx| unsafe {
                nixb_sys::store_query_path_from_hash_part(
                    ctx,
                    self.inner,
                    hash.as_ref().as_ptr(),
                )
            })
            .map(|ptr| (!ptr.is_null()).then(|| StorePath::new(ptr)))
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
