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

#[cfg(not(feature = "nix-2-35"))]
compile_error!(
    "Enable one of the Nix version features: nix-2-35"
);

#[cfg(feature = "nix-2-35")]
#[rustfmt::skip]
#[path = "2_35.rs"]
mod selected;

pub use selected::*;
