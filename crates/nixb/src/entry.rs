use core::ptr::NonNull;

use crate::prelude::{Context, Entrypoint};

pub type EntrypointFun = fn(&mut Context<Entrypoint>);

#[doc(hidden)]
#[inline]
pub unsafe fn entry(entrypoint: EntrypointFun) {
    #[cfg(feature = "dlopen")]
    crate::dlopen::open();

    match NonNull::new(unsafe { nixb_sys::c_context_create() }) {
        Some(ctx) => {
            entrypoint(&mut Context::new(ctx, Entrypoint {}));
            unsafe { nixb_sys::c_context_free(ctx.as_ptr()) };
        },
        None => panic!("couldn't allocate new 'nix_c_context'"),
    }
}
