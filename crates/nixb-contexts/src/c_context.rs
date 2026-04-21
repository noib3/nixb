//! TODO: docs.

use alloc::borrow::ToOwned;
use core::ffi::CStr;
use core::{ptr, slice};

/// TODO: docs.
pub struct CContext {
    ptr: *mut nixb_sys::c_context,
}

impl CContext {
    /// TODO: docs.
    #[track_caller]
    #[inline]
    pub fn create() -> Self {
        Self::new(unsafe { nixb_sys::c_context_create() })
    }

    /// TODO: docs.
    #[track_caller]
    #[inline]
    pub fn new(ptr: *mut nixb_sys::c_context) -> Self {
        assert!(!ptr.is_null());
        Self { ptr }
    }

    /// TODO: docs.
    #[inline]
    pub fn with_ptr<T>(
        &mut self,
        fun: impl FnOnce(*mut nixb_sys::c_context) -> T,
    ) -> nixb_error::Result<T> {
        let ret = fun(self.ptr);
        check_err(self.ptr).map(|()| ret)
    }
}

impl Drop for CContext {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::c_context_free(self.ptr) };
    }
}

fn check_err(ctx: *mut nixb_sys::c_context) -> nixb_error::Result<()> {
    let kind = match unsafe { nixb_sys::err_code(ctx) } {
        nixb_sys::err_NIX_OK => return Ok(()),
        nixb_sys::err_NIX_ERR_UNKNOWN => nixb_error::ErrorKind::Unknown,
        nixb_sys::err_NIX_ERR_OVERFLOW => nixb_error::ErrorKind::Overflow,
        nixb_sys::err_NIX_ERR_KEY => nixb_error::ErrorKind::Key,
        nixb_sys::err_NIX_ERR_NIX_ERROR => nixb_error::ErrorKind::Nix,
        other => unreachable!("invalid error code: {other}"),
    };

    let mut err_msg_len = 0;

    let err_msg_ptr =
        unsafe { nixb_sys::err_msg(ptr::null_mut(), ctx, &mut err_msg_len) };

    let bytes = unsafe {
        slice::from_raw_parts(
            err_msg_ptr as *const u8,
            (err_msg_len + 1) as usize,
        )
    };

    let err_msg = unsafe { CStr::from_bytes_with_nul_unchecked(bytes) };

    Err(nixb_error::Error::new(kind, err_msg.to_owned()))
}
