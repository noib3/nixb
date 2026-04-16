//! TODO: docs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod context;
#[cfg(feature = "dlopen")]
mod dlopen;
mod entrypoint;
pub mod primop;
pub use context::{ContextExt, Entrypoint};
#[doc(hidden)]
pub use entrypoint::entrypoint;
pub use nixb_macros::{PrimOp, entry};
