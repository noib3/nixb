//! TODO: docs.

#![no_std]

// - nix_libstore_init
// - nix_libstore_init_no_load_config
// - nix_store_open
// - nix_store_get_fs_closure
// - nix_store_free

extern crate alloc;

#[cfg(feature = "context-c")]
pub mod c_store_context;
mod context;

pub use context::StoreContext;
