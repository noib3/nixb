//! TODO: docs.

use core::ffi::CStr;

use nixb_expr::Utf8CStr;
use nixb_expr::function::Function;

/// TODO: docs.
pub trait PrimOp: for<'a> Function<'a> + 'static {
    #[doc(hidden)]
    const DOCS: Option<&'static CStr>;

    #[doc(hidden)]
    const NAME: &'static Utf8CStr;
}
