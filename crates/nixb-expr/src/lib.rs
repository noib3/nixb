//! TODO: docs.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(const_type_name))]

extern crate alloc;

pub mod attrset;
pub mod builtins;
pub mod callable;
pub mod context;
pub mod error;
#[cfg(feature = "embed")]
pub mod eval_state;
pub mod function;
#[cfg(feature = "embed")]
mod init;
mod into_result;
pub mod list;
mod never;
pub mod primop;
#[doc(hidden)]
pub mod set_pattern;
pub mod thunk;
pub mod tuple;
mod utf8_cstr;
pub mod value;

#[cfg(feature = "embed")]
pub use eval_state::EvalState;
#[cfg(feature = "embed")]
pub use init::{InitSentinel, init};
pub use into_result::IntoResult;
pub use never::Never;
pub use nixb_macros::{Attrset, PrimOp, SetPattern, Value};
#[cfg(feature = "embed")]
pub use primop::register_primop;
pub use utf8_cstr::Utf8CStr;
