//! TODO: docs.

use core::ffi::{CStr, c_char};
use core::ptr;

use crate::Utf8CStr;
use crate::context::Context;
use crate::error::Result;
use crate::function::{Args, ArgsList, Function};
use crate::value::{IntoValue, Value};

/// TODO: docs.
pub trait PrimOp: for<'a> Function<'a> + 'static {
    #[doc(hidden)]
    const DOCS: &'static CStr;

    #[doc(hidden)]
    const NAME: &'static Utf8CStr;
}

impl<T: IntoValue + Clone> Function<'_> for T {
    type Args = NoArgs;

    #[inline]
    fn call<'eval>(
        &mut self,
        _: (),
        ctx: &mut Context<'eval>,
    ) -> impl Value + use<'eval, T> {
        self.clone().into_value(ctx)
    }
}

#[doc(hidden)]
pub struct NoArgs;

impl Args<'_> for NoArgs {
    type Values = ();

    const NAMES: &'static [*const c_char] = &[ptr::null()];

    #[inline]
    fn values_from_args_list(
        _: ArgsList<'_>,
        _: &mut Context,
    ) -> Result<Self::Values> {
        Ok(())
    }
}
