//! TODO: docs.

use core::ffi::CStr;

use nixb_expr::function::Function;
use nixb_expr::Utf8CStr;

/// TODO: docs.
pub trait PrimOp: Function + 'static {
    #[doc(hidden)]
    const DOCS: Option<&'static CStr>;

    #[doc(hidden)]
    const NAME: &'static Utf8CStr;
}
