#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! name = "nixb-sys-generator"
//! version = "0.0.0"
//! edition = "2024"
//!
//! [[bin]]
//! name = "nixb-sys-generator"
//! path = "src/main.rs"
//!
//! [dependencies]
//! bindgen = "0.72"
//! doxygen-bindgen = "0.1"
//! ```

#![expect(missing_docs)]

use std::path::Path;
use std::{env, fs};

use bindgen::callbacks::{ItemInfo, ParseCallbacks};

fn main() -> Result<(), String> {
    let nix_source = env::var("NIX_SOURCE").expect("$NIX_SOURCE set");
    let nix_version = env::var("NIX_VERSION").expect("$NIX_VERSION set");
    let output_file = env::var("OUTPUT_FILE").expect("$OUTPUT_FILE set");

    let mut builder = bindgen::Builder::default()
        .use_core()
        .formatter(bindgen::Formatter::Rustfmt)
        .disable_header_comment()
        .raw_line(format!(
            "//! This file is generated from Nix {nix_version} headers. Do \
             not edit it manually."
        ))
        .raw_line("")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .parse_callbacks(Box::new(ProcessComments))
        .parse_callbacks(Box::new(StripNixPrefix));

    builder = builder.header_contents(
        "wrapper.h",
        r#"
      #include <nix_api_expr.h>
      #include <nix_api_external.h>
      #include <nix_api_flake.h>
      #include <nix_api_store.h>
      #include <nix_api_util.h>
      #include <nix_api_value.h>
    "#,
    );

    let nix_source = Path::new(&nix_source);

    let include_paths = [
        nix_source.join("src/libexpr-c"),
        nix_source.join("src/libfetchers-c"),
        nix_source.join("src/libflake-c"),
        nix_source.join("src/libstore-c"),
        nix_source.join("src/libutil-c"),
    ];

    for include_path in include_paths {
        builder = builder.clang_arg(format!("-I{}", include_path.display()));
    }

    let bindings = builder
        .generate()
        .map_err(|err| format!("Couldn't generate bindings: {err}"))?;

    let output_file = Path::new(&output_file);

    if let Some(parent) = output_file.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "Couldn't create output directory {}: {err}",
                parent.display()
            )
        })?;
    }

    fs::write(output_file, bindings.to_string()).map_err(|err| {
        format!(
            "Couldn't write generated bindings to {}: {err}",
            output_file.display()
        )
    })?;

    Ok(())
}

#[derive(Debug)]
struct ProcessComments;

#[derive(Debug)]
struct StripNixPrefix;

impl ParseCallbacks for ProcessComments {
    fn process_comment(&self, comment: &str) -> Option<String> {
        match doxygen_bindgen::transform(comment) {
            Ok(res) => Some(res),
            Err(err) => {
                eprintln!(
                    "Problem processing doxygen comment: {comment}\n{err}"
                );
                None
            },
        }
    }
}

impl ParseCallbacks for StripNixPrefix {
    fn item_name(&self, item_info: ItemInfo<'_>) -> Option<String> {
        item_info.name.strip_prefix("nix_").map(ToOwned::to_owned)
    }
}
