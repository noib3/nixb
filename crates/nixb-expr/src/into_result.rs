use core::convert::Infallible;

use crate::value::IntoValue;

/// TODO: docs.
pub trait IntoResult {
    /// TODO: docs.
    type Output;

    /// TODO: docs.
    type Error;

    /// TODO: docs.
    fn into_result(self) -> Result<Self::Output, Self::Error>;
}

// NOTE: the `T: IntoValue` bound is not needed to implement `IntoResult`, it's
// only there to avoid having to specify the type of error when returning a
// result from a closure Thunk, e.g.:
//
// ```rust
// fn is_thunk<T, F: Thunk<T>>(_f: F) {}
//
// fn foo() {
//     is_thunk(|ctx: &mut Context| Ok("This is lazy!"));
// }
// ```
impl<T: IntoValue> IntoResult for T {
    type Output = Self;
    type Error = Infallible;

    #[inline(always)]
    fn into_result(self) -> Result<Self::Output, Self::Error> {
        Ok(self)
    }
}

impl<T, E> IntoResult for Result<T, E> {
    type Output = T;
    type Error = E;

    #[inline(always)]
    fn into_result(self) -> Self {
        self
    }
}
