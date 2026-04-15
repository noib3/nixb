use core::ptr::NonNull;

use nixb_expr::context::Context;

use crate::context::Entrypoint;

pub type EntrypointFun = fn(&mut Context<Entrypoint>);

#[doc(hidden)]
#[inline]
pub unsafe fn entrypoint(entrypoint: EntrypointFun) {
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
