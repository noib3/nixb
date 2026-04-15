//! TODO: docs.

#![allow(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(const_type_name))]
#![cfg_attr(nightly, feature(generic_const_exprs))]

extern crate alloc;

#[cfg(feature = "dlopen")]
mod dlopen;
mod entry;
pub mod primop;
pub use nixb_macros::{PrimOp, plugin};

#[doc(hidden)]
pub use entry::entry;
