#![expect(missing_docs)]

use std::env;

const EMBED_FEATURE: &str = "CARGO_FEATURE_EMBED";

fn main() {
    if let Ok(version) = rustc_version::version_meta()
        && version.channel == rustc_version::Channel::Nightly
    {
        println!("cargo:rustc-cfg=nightly");
    }

    if env::var_os(EMBED_FEATURE).is_some() {
        pkg_config::Config::new()
            .probe("nix-expr-c")
            .expect("Could not find nix-expr-c via pkg-config");
    }

    println!("cargo:rerun-if-env-changed={EMBED_FEATURE}");
}
