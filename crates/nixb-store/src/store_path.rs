use core::ffi::{c_char, c_uint, c_void};
use core::slice;

use nixb_contexts::c_context::CContext;
use nixb_error::Result;

/// TODO: docs.
pub struct StorePath {
    inner: *mut nixb_sys::StorePath,
}

impl StorePath {
    /// TODO: docs.
    #[inline]
    pub fn hash(&self) -> Result<[u8; 20]> {
        let mut hash = nixb_sys::store_path_hash_part { bytes: [0; 20] };

        CContext::create().with_ptr(|ctx| unsafe {
            nixb_sys::store_path_hash(ctx, self.inner, &mut hash)
        })?;

        Ok(hash.bytes)
    }

    /// TODO: docs.
    #[inline]
    pub fn with_name<T, F>(&self, fun: F) -> T
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
            let name_bytes = unsafe {
                slice::from_raw_parts(start.cast::<u8>(), n as usize)
            };
            // SAFETY: store path names can only contain alphanumeric characters
            // and a handful or ASCII symbols, so the name is valid UTF-8.
            let name = unsafe { str::from_utf8_unchecked(name_bytes) };
            let state =
                unsafe { &mut *user_data.cast::<CallbackState<F, T>>() };
            let fun = state.fun.take().expect("it's set");
            state.ret = Some(fun(name));
        }

        let mut state = CallbackState { fun: Some(fun), ret: None };

        unsafe {
            nixb_sys::store_path_name(
                self.inner,
                Some(callback::<F, T>),
                (&mut state as *mut CallbackState<F, T>).cast(),
            );
        }

        state.ret.expect("callback was called")
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut nixb_sys::StorePath {
        self.inner
    }

    #[inline]
    pub(crate) fn new(ptr: *mut nixb_sys::StorePath) -> Self {
        assert!(!ptr.is_null());
        Self { inner: ptr }
    }
}

impl Clone for StorePath {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(unsafe { nixb_sys::store_path_clone(self.inner) })
    }
}

impl Drop for StorePath {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::store_path_free(self.inner) };
    }
}
