//! TODO: docs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod entrypoint;
mod plugin;

#[doc(hidden)]
pub use entrypoint::entrypoint;
pub use nixb_expr::{PrimOp, primop};
pub use nixb_macros::entry;
pub use plugin::Plugin;
