use nixb_c_context::CContext;
use nixb_expr::context::Context;

use crate::context::Entrypoint;

pub type EntrypointFun = fn(&mut Context<Entrypoint>);

#[doc(hidden)]
#[inline]
pub unsafe fn entrypoint(entrypoint: EntrypointFun) {
    #[cfg(feature = "dlopen")]
    crate::dlopen::open();

    let c_context = CContext::create();
    entrypoint(&mut Context::new(c_context, Entrypoint {}))
}
