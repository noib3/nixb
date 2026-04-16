//! TODO: docs.

use alloc::borrow::ToOwned;
use core::ffi::CStr;
use core::ptr::{self, NonNull};
use core::slice;

/// TODO: docs.
pub struct CContext {
    ptr: NonNull<nixb_sys::c_context>,
}

impl CContext {
    /// TODO: docs.
    #[track_caller]
    #[inline]
    pub fn create() -> Self {
        let Some(ptr) = NonNull::new(unsafe { nixb_sys::c_context_create() })
        else {
            panic!("couldn't allocate new C context");
        };
        Self { ptr }
    }

    /// TODO: docs.
    #[inline]
    pub fn with_ptr<T>(
        &mut self,
        fun: impl FnOnce(*mut nixb_sys::c_context) -> T,
    ) -> nixb_result::Result<T> {
        let ptr = self.ptr.as_ptr();
        let ret = fun(ptr);
        check_err(ptr).map(|()| ret)
    }
}

fn check_err(ctx: *mut nixb_sys::c_context) -> nixb_result::Result<()> {
    let kind = match unsafe { nixb_sys::err_code(ctx) } {
        nixb_sys::err_NIX_OK => return Ok(()),
        nixb_sys::err_NIX_ERR_UNKNOWN => nixb_result::ErrorKind::Unknown,
        nixb_sys::err_NIX_ERR_OVERFLOW => nixb_result::ErrorKind::Overflow,
        nixb_sys::err_NIX_ERR_KEY => nixb_result::ErrorKind::Key,
        nixb_sys::err_NIX_ERR_NIX_ERROR => nixb_result::ErrorKind::Nix,
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

    Err(nixb_result::Error::new(kind, err_msg.to_owned()))
}
