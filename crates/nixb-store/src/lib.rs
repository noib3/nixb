//! TODO: docs.

#![no_std]

extern crate alloc;

mod get_fs_closure_opts;
mod init;
mod nix_derivation;
#[cfg(feature = "nix-2-35")]
mod path_info;
mod settings;
mod store;
mod store_param;
mod store_path;

pub use get_fs_closure_opts::GetFsClosureOpts;
pub use init::{InitSentinel, init};
pub use nix_derivation::NixDerivation;
#[cfg(feature = "nix-2-35")]
pub use path_info::PathInfo;
pub use settings::{get_setting, set_setting};
pub use store::Store;
pub use store_param::StoreParam;
pub use store_path::StorePath;
