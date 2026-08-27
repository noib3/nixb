#![expect(missing_docs)]

use std::env;

const EMBED_FEATURE: &str = "CARGO_FEATURE_EMBED";

fn main() {
    if env::var_os(EMBED_FEATURE).is_some() {
        pkg_config::Config::new()
            .probe("nix-store-c")
            .expect("Could not find nix-store-c via pkg-config");
    }

    println!("cargo:rerun-if-env-changed={EMBED_FEATURE}");
}
