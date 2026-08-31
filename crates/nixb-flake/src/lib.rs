//! Bindings to Nix's flake reference, locking, and evaluation APIs.

#![no_std]

extern crate alloc;

mod lock;
mod reference;
mod settings;

pub use lock::{FlakeLockFlags, LockedFlake};
pub use reference::{FlakeReference, FlakeReferenceParseFlags};
pub use settings::FlakeSettings;
