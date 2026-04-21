#![expect(missing_docs)]

use std::env;

const EXPR_FEATURE: &str = "CARGO_FEATURE_EXPR";
const NIX_2_33_FEATURE: &str = "CARGO_FEATURE_NIX_2_33";
const NIX_2_34_FEATURE: &str = "CARGO_FEATURE_NIX_2_34";
const NIX_2_35_FEATURE: &str = "CARGO_FEATURE_NIX_2_35";
const STORE_FEATURE: &str = "CARGO_FEATURE_STORE";

fn main() {
    let target_nix = NixVersion::selected();
    let has_expr = env::var_os(EXPR_FEATURE).is_some();
    let has_store = env::var_os(STORE_FEATURE).is_some();

    assert!(
        has_expr || has_store,
        "nixb-cpp requires at least one of the `expr` or `store` features"
    );

    let mut build = cc::Build::new();

    if has_expr {
        let nix_expr = pkg_config::Config::new()
            .cargo_metadata(false)
            .probe("nix-expr")
            .expect("Could not find nix-expr via pkg-config");

        for include_path in &nix_expr.include_paths {
            build.include(include_path);
        }

        build.file("cpp/wrapper.cpp").file("cpp/function.cpp");
    }

    if has_store {
        let nix_store = pkg_config::Config::new()
            .cargo_metadata(false)
            .probe("nix-store")
            .expect("Could not find nix-store via pkg-config");

        for include_path in &nix_store.include_paths {
            build.include(include_path);
        }

        build.file("cpp/store.cpp");
    }

    build
        .cpp(true)
        .define(target_nix.as_cpp_define(), None)
        .flag("-std=c++23")
        .compile("nixb_cpp");

    println!("cargo:rerun-if-changed=cpp/wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp/function.cpp");
    println!("cargo:rerun-if-changed=cpp/store.cpp");
    println!("cargo:rerun-if-env-changed={EXPR_FEATURE}");
    println!("cargo:rerun-if-env-changed={NIX_2_33_FEATURE}");
    println!("cargo:rerun-if-env-changed={NIX_2_34_FEATURE}");
    println!("cargo:rerun-if-env-changed={NIX_2_35_FEATURE}");
    println!("cargo:rerun-if-env-changed={STORE_FEATURE}");
}

#[derive(Copy, Clone)]
enum NixVersion {
    Nix233,
    Nix234,
    Nix235,
}

impl NixVersion {
    fn as_cpp_define(self) -> &'static str {
        match self {
            Self::Nix233 => "NIX_2_33",
            Self::Nix234 => "NIX_2_34",
            Self::Nix235 => "NIX_2_35",
        }
    }

    fn selected() -> Self {
        if env::var_os(NIX_2_34_FEATURE).is_some() {
            Self::Nix234
        } else if env::var_os(NIX_2_35_FEATURE).is_some() {
            Self::Nix235
        } else if env::var_os(NIX_2_33_FEATURE).is_some() {
            Self::Nix233
        } else {
            unreachable!()
        }
    }
}
