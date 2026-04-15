//! TODO: docs.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

/// TODO: docs.
pub type Result<T> = core::result::Result<T, Error>;

mod error;

pub use error::{Error, ErrorKind};
