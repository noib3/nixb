//! TODO: docs.

use core::fmt;
use core::marker::PhantomData;

use crate::value::ValueKind;

/// The type of error that can occur when trying to convert a generic value
/// to a specific type.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct TypeMismatchError {
    /// The expected value kind.
    pub expected: ValueKind,

    /// The found value kind.
    pub found: ValueKind,
}

/// The type of error that can occur when trying to convert an `i64` into a
/// different integer type.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct TryFromI64Error<Int> {
    n: i64,
    int: PhantomData<Int>,
}

impl<Int> TryFromI64Error<Int> {
    #[inline]
    pub(crate) fn new(n: i64) -> Self {
        Self { n, int: PhantomData }
    }
}

impl fmt::Display for TypeMismatchError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "type mismatch: expected {:?}, found {:?}",
            self.expected, self.found
        )
    }
}

impl core::error::Error for TypeMismatchError {}

impl<Int> fmt::Debug for TryFromI64Error<Int> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TryFromI64Error")
            .field("n", &self.n)
            .field("int", &core::any::type_name::<Int>())
            .finish()
    }
}

impl<Int> fmt::Display for TryFromI64Error<Int> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "integer conversion failed: cannot convert {}i64 into target type \
             {}",
            self.n,
            core::any::type_name::<Int>()
        )
    }
}

impl<Int> core::error::Error for TryFromI64Error<Int> {}

impl<Int> From<TryFromI64Error<Int>> for nixb_error::Error {
    #[inline]
    fn from(err: TryFromI64Error<Int>) -> Self {
        Self::from_message(err)
    }
}

impl From<TypeMismatchError> for nixb_error::Error {
    #[inline]
    fn from(err: TypeMismatchError) -> Self {
        Self::from_message(err)
    }
}
