use nixb_c_context::CContext;

use crate::Plugin;

pub type EntrypointFun = fn(&mut Plugin);

#[doc(hidden)]
#[inline]
pub unsafe fn entrypoint(entrypoint: EntrypointFun) {
    entrypoint(&mut Plugin::new(CContext::create()))
}
