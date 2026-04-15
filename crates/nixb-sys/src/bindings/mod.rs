//! Generated bindings for the Nix C API.
//!
//! To update the files in this directory, run:
//! ```sh
//! nix run .#update-sys-bindings
//! ```

#![allow(clippy::use_self)]
#![allow(missing_docs)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(rustdoc::bare_urls)]
#![allow(rustdoc::broken_intra_doc_links)]
#![allow(rustdoc::invalid_html_tags)]

#[cfg(not(feature = "nix-2-32"))]
compile_error!(
    "Enable one of the Nix version features: nix-2-32, nix-2-33, nix-2-34"
);

#[cfg(all(feature = "nix-2-32", not(feature = "nix-2-33")))]
#[rustfmt::skip]
#[path = "2_32.rs"]
mod selected;

#[cfg(all(feature = "nix-2-33", not(feature = "nix-2-34")))]
#[rustfmt::skip]
#[path = "2_33.rs"]
mod selected;

#[cfg(feature = "nix-2-34")]
#[rustfmt::skip]
#[path = "2_34.rs"]
mod selected;

pub use selected::*;
