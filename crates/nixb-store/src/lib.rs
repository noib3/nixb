//! TODO: docs.

#![no_std]

extern crate alloc;

mod get_fs_closure_opts;
mod init;
mod store;
mod store_param;
mod store_path;

pub use get_fs_closure_opts::GetFsClosureOpts;
pub use init::{InitSentinel, init};
pub use store::Store;
pub use store_param::StoreParam;
pub use store_path::StorePath;
