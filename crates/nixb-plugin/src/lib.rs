//! TODO: docs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(feature = "dlopen")]
mod dlopen;
mod entrypoint;
mod plugin;
pub mod primop;

#[doc(hidden)]
pub use entrypoint::entrypoint;
pub use nixb_macros::{PrimOp, entry};
pub use plugin::Plugin;
