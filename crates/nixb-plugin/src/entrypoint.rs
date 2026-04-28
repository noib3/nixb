use nixb_c_context::CContext;

use crate::Plugin;

pub type EntrypointFun = fn(&mut Plugin);

#[doc(hidden)]
#[inline]
pub unsafe fn entrypoint(entrypoint: EntrypointFun) {
    #[cfg(feature = "dlopen")]
    crate::dlopen::open();
    entrypoint(&mut Plugin::new(CContext::create()))
}
