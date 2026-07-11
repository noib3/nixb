use core::ffi::CStr;

use crate::attrset::Key;
use crate::context::Context;
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
    fn write(self, _: UninitValue, _: &mut Context) {
        match self {}
    }
}

impl Key for Never {
    #[inline]
    fn with_cstr<F>(&self, _: impl FnOnce(&CStr) -> F) -> F {
        match *self {}
    }
}
