//! TODO: docs.

#![allow(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(const_type_name))]
#![cfg_attr(nightly, feature(generic_const_exprs))]

extern crate alloc;

pub mod attrset;
pub mod builtins;
pub mod callable;
pub mod context;
#[cfg(feature = "dlopen")]
mod dlopen;
mod entry;
pub mod error;
pub mod function;
mod into_result;
pub mod list;
mod never;
pub mod primop;
pub mod thunk;
pub mod tuple;
mod utf8_cstr;
pub mod value;

#[doc(hidden)]
pub use entry::entry;
pub use into_result::IntoResult;
pub use nixb_macros::{Attrset, PrimOp, TryFromValue, Value, entry};
#[doc(hidden)]
pub use nixb_sys as sys;
pub use utf8_cstr::Utf8CStr;

pub mod prelude {
    //! TODO: docs.

    pub use crate::Utf8CStr;
    pub use crate::attrset::*;
    pub use crate::callable::*;
    pub use crate::context::*;
    pub use crate::error::*;
    pub use crate::function::*;
    pub use crate::list::*;
    pub use crate::primop::*;
    pub use crate::thunk::*;
    pub use crate::value::*;
}
