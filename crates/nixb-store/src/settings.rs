use core::ffi::{CStr, c_char, c_uint, c_void};

use nixb_error::Result;

use crate::InitSentinel;

/// Calls the given function with the value of a global Nix setting and returns
/// its output.
///
/// The `init` argument is the sentinel returned by [`init`](crate::init).
/// Calling `init::<true>()` loads settings from configuration files, while
/// `init::<false>()` uses their default values.
#[inline]
pub fn get_setting<T, F>(
    key: impl AsRef<CStr>,
    fun: F,
    init: &mut InitSentinel,
) -> Result<T>
where
    F: FnOnce(&[u8]) -> T,
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
        F: FnOnce(&[u8]) -> T,
    {
        let bytes = unsafe {
            core::slice::from_raw_parts(start.cast::<u8>(), n as usize)
        };
        let state = unsafe { &mut *user_data.cast::<CallbackState<F, T>>() };
        let fun = state.fun.take().expect("it's set");
        state.ret = Some(fun(bytes));
    }

    let mut state = CallbackState { fun: Some(fun), ret: None };

    init.ctx.with_ptr(|ctx| unsafe {
        nixb_sys::setting_get(
            ctx,
            key.as_ref().as_ptr(),
            Some(callback::<F, T>),
            (&mut state as *mut CallbackState<F, T>).cast(),
        )
    })?;

    Ok(state.ret.expect("callback was called"))
}

/// Sets the value of a global Nix setting.
#[inline]
pub fn set_setting(
    key: impl AsRef<CStr>,
    value: impl AsRef<CStr>,
    init: &mut InitSentinel,
) -> Result<()> {
    init.ctx
        .with_ptr(|ctx| unsafe {
            nixb_sys::setting_set(
                ctx,
                key.as_ref().as_ptr(),
                value.as_ref().as_ptr(),
            )
        })
        .map(|_err| ())
}
