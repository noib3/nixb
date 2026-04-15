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
pub mod error;
pub mod function;
mod into_result;
pub mod list;
mod never;
pub mod thunk;
pub mod tuple;
mod utf8_cstr;
pub mod value;

pub use into_result::IntoResult;
pub use nixb_macros::{Attrset, TryFromValue, Value};
pub use utf8_cstr::Utf8CStr;
