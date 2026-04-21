//! C++ bindings for Nix that expose the C++ API with C ABI.
//!
//! This crate provides thin C++ wrapper functions that allow Rust code to
//! access Nix C++ API features not available in the C API, such as allocating
//! values within primop callbacks.

#![no_std]

#[cfg(feature = "expr")]
mod expr;
#[cfg(feature = "expr")]
mod function;
#[cfg(feature = "store")]
mod store;

#[cfg(feature = "expr")]
pub use expr::*;
#[cfg(feature = "expr")]
pub use function::{FunctionCallback, init_function};
#[cfg(feature = "store")]
pub use store::store_clone;
