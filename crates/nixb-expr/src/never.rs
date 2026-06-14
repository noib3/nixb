use core::ffi::CStr;
use core::ops::Deref;

use crate::attrset::Key;
use crate::context::Context;
use crate::tuple::Tuple;
use crate::value::{UninitValue, Value, ValueKind};

/// An uninhabited type used in tuple positions that can never exist.
#[derive(Debug, Copy, Clone)]
pub enum Never {}

impl Value for Never {
    #[inline]
    fn kind(&self) -> ValueKind {
        match *self {}
    }

    #[inline]
    fn write(self, _: UninitValue, _: &mut Context) -> nixb_error::Result<()> {
        match self {}
    }
}

impl Tuple for Never {
    const LEN: usize = unreachable!();
    type First = Self;
    type Last = Self;
    type FromFirst = Self;
    type UpToLast = Self;
    type Borrow<'a> = Self;

    #[cold]
    fn borrow(&self) -> Self {
        unreachable!()
    }

    #[cold]
    fn split_first(self) -> (Self::First, Self::FromFirst) {
        unreachable!()
    }
    #[cold]
    fn split_last(self) -> (Self::UpToLast, Self::Last) {
        unreachable!()
    }
}

impl Key for Never {
    #[inline]
    fn with_cstr<F>(&self, _: impl FnOnce(&CStr) -> F) -> F {
        match *self {}
    }
}

impl Deref for Never {
    type Target = Self;

    #[inline]
    fn deref(&self) -> &Self::Target {
        match *self {}
    }
}
