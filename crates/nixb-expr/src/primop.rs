//! Primitive operations.

use core::ffi::CStr;
use core::ptr;

use nixb_c_context::CContext;

use crate::Utf8CStr;
use crate::function::Function;

/// Registers a primitive operation in all the subsequently created evaluator
/// states.
#[cfg(feature = "embed")]
#[inline]
pub fn register_primop<P: PrimOp>(
    primop: P,
    init: &mut crate::InitSentinel,
) -> nixb_error::Result<()> {
    primop.register(&mut init.ctx)
}

/// A Nix primitive operation.
pub trait PrimOp: Function + 'static {
    #[doc(hidden)]
    const DOCS: Option<&'static CStr>;

    #[doc(hidden)]
    const NAME: &'static Utf8CStr;

    #[doc(hidden)]
    #[inline]
    fn register(self, ctx: &mut CContext) -> nixb_error::Result<()>
    where
        Self: Sized,
    {
        let primop_ptr = ctx.with_ptr(|ctx| unsafe {
            nixb_sys::alloc_primop(
                ctx,
                Some(Self::callback()),
                self.args_arity().into(),
                Self::NAME.as_c_str().as_ptr(),
                self.args_names().as_ptr().cast_mut(),
                Self::DOCS.map(CStr::as_ptr).unwrap_or(ptr::null()),
                self.into_userdata(),
            )
        })?;

        ctx.with_ptr(|ctx| unsafe {
            nixb_sys::register_primop(ctx, primop_ptr)
        })?;

        ctx.with_ptr(|ctx| unsafe {
            nixb_sys::gc_decref(ctx, primop_ptr.cast())
        })
        .map(|_| ())
    }
}
