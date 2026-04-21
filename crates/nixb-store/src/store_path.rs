/// TODO: docs.
pub struct StorePath {
    inner: *mut nixb_sys::StorePath,
}

impl StorePath {
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

impl Drop for StorePath {
    fn drop(&mut self) {
        unsafe { nixb_sys::store_path_free(self.inner) };
    }
}
