use core::ffi::CStr;
use core::ptr;

use nixb_expr::context::Context;

use crate::primop::PrimOp;

/// TODO: docs.
pub trait ContextExt {
    /// Adds the given primop to the `builtins` attribute set.
    fn register_primop<P: PrimOp>(&mut self, primop: P) -> &mut Self;
}

/// TODO: docs.
pub struct Entrypoint {}

impl ContextExt for Context<'_, Entrypoint> {
    #[track_caller]
    #[inline]
    fn register_primop<P: PrimOp>(&mut self, primop: P) -> &mut Self {
        let try_block = || {
            let primop_ptr = self.with_raw(|ctx| unsafe {
                nixb_sys::alloc_primop(
                    ctx,
                    Some(P::callback()),
                    primop.args_arity().into(),
                    P::NAME.as_c_str().as_ptr(),
                    primop.args_names().as_ptr().cast_mut(),
                    P::DOCS.map(CStr::as_ptr).unwrap_or(ptr::null()),
                    primop.into_userdata(),
                )
            })?;

            self.with_raw(|ctx| unsafe {
                nixb_sys::register_primop(ctx, primop_ptr)
            })?;

            self.with_raw(|ctx| unsafe {
                nixb_sys::gc_decref(ctx, primop_ptr.cast())
            })
        };

        if let Err(err) = try_block() {
            panic!("couldn't register primop {:?}: {err}", P::NAME);
        }

        self
    }
}
