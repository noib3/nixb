/// TODO: docs.
pub struct NixDerivation {
    inner: *mut nixb_sys::derivation,
}

impl NixDerivation {
    #[inline]
    pub(crate) fn new(ptr: *mut nixb_sys::derivation) -> Self {
        assert!(!ptr.is_null());
        Self { inner: ptr }
    }
}

impl Clone for NixDerivation {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(unsafe { nixb_sys::derivation_clone(self.inner) })
    }
}

impl Drop for NixDerivation {
    fn drop(&mut self) {
        unsafe {
            nixb_sys::derivation_free(self.inner);
        }
    }
}
