//! TODO: docs.

use core::ffi::{CStr, c_char, c_uint, c_void};
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

    /// Calls the given function with the `storeDir` of this store (typically
    /// `"/nix/store"`) and returns its output.
    #[inline]
    pub fn get_storedir<T, F>(&mut self, fun: F) -> Result<T>
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
            let storedir = unsafe { str::from_utf8_unchecked(bytes) };
            let state =
                unsafe { &mut *user_data.cast::<CallbackState<F, T>>() };
            let fun = state.fun.take().expect("it's set");
            state.ret = Some(fun(storedir));
        }

        let mut state = CallbackState { fun: Some(fun), ret: None };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_storedir(
                ctx,
                self.inner,
                Some(callback::<F, T>),
                (&mut state as *mut CallbackState<F, T>).cast(),
            )
        })?;

        Ok(state.ret.expect("callback was called"))
    }

    /// Calls the given function with this store's URI and returns its output.
    #[inline]
    pub fn get_uri<T, F>(&mut self, fun: F) -> Result<T>
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
            let uri = unsafe { str::from_utf8_unchecked(bytes) };
            let state =
                unsafe { &mut *user_data.cast::<CallbackState<F, T>>() };
            let fun = state.fun.take().expect("it's set");
            state.ret = Some(fun(uri));
        }

        let mut state = CallbackState { fun: Some(fun), ret: None };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_uri(
                ctx,
                self.inner,
                Some(callback::<F, T>),
                (&mut state as *mut CallbackState<F, T>).cast(),
            )
        })?;

        Ok(state.ret.expect("callback was called"))
    }

    /// Calls the given function with this store's version and returns its
    /// output.
    ///
    /// If the store doesn't have a version (like the dummy store), calls the
    /// given function with `None`.
    #[inline]
    pub fn get_version<T, F>(&mut self, fun: F) -> Result<T>
    where
        F: FnOnce(Option<&str>) -> T,
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
            F: FnOnce(Option<&str>) -> T,
        {
            let version = if n == 0 {
                None
            } else {
                let bytes = unsafe {
                    core::slice::from_raw_parts(start.cast::<u8>(), n as usize)
                };
                Some(unsafe { str::from_utf8_unchecked(bytes) })
            };

            let state =
                unsafe { &mut *user_data.cast::<CallbackState<F, T>>() };
            let fun = state.fun.take().expect("it's set");
            state.ret = Some(fun(version));
        }

        let mut state = CallbackState { fun: Some(fun), ret: None };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_get_version(
                ctx,
                self.inner,
                Some(callback::<F, T>),
                (&mut state as *mut CallbackState<F, T>).cast(),
            )
        })?;

        Ok(state.ret.expect("callback was called"))
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

    /// Calls the given function with the physical location of the given
    /// [`StorePath`] and returns its output.
    ///
    /// A store may reside at a different location than its `storeDir` suggests.
    /// This situation is called a relocated store. Relocated stores are used
    /// during NixOS installation, as well as in restricted computing
    /// environments that don't offer a writable `"/nix/store"`.
    ///
    /// Not all types of stores support this operation.
    #[inline]
    pub fn real_path<T, F>(
        &mut self,
        store_path: &StorePath,
        fun: F,
    ) -> Result<T>
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
            let real_path = unsafe { str::from_utf8_unchecked(bytes) };
            let state =
                unsafe { &mut *user_data.cast::<CallbackState<F, T>>() };
            let fun = state.fun.take().expect("it's set");
            state.ret = Some(fun(real_path));
        }

        let mut state = CallbackState { fun: Some(fun), ret: None };

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_real_path(
                ctx,
                self.inner,
                store_path.as_ptr(),
                Some(callback::<F, T>),
                (&mut state as *mut CallbackState<F, T>).cast(),
            )
        })?;

        Ok(state.ret.expect("callback was called"))
    }

    /// Realises the given [`StorePath`], calling the given function once for
    /// each realised output with its name and the realised output path.
    ///
    /// This method is blocking.
    ///
    /// On error, the function is never called.
    #[inline]
    pub fn realise<F>(
        &mut self,
        store_path: &StorePath,
        mut fun: F,
    ) -> Result<()>
    where
        F: FnMut(&str, &StorePath),
    {
        unsafe extern "C" fn callback<F>(
            userdata: *mut c_void,
            outname: *const c_char,
            out: *const nixb_sys::StorePath,
        ) where
            F: FnMut(&str, &StorePath),
        {
            let fun = unsafe { &mut *userdata.cast::<F>() };
            let outname = unsafe { CStr::from_ptr(outname) };
            let outname =
                outname.to_str().expect("Nix output names are valid UTF-8");
            let out = StorePath::new(out.cast_mut());
            (fun)(outname, &out);
            mem::forget(out);
        }

        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::store_realise(
                ctx,
                self.inner,
                store_path.as_ptr(),
                (&mut fun as *mut F).cast(),
                Some(callback::<F>),
            )
        })?;

        Ok(())
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
