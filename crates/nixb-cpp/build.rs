#![expect(missing_docs)]

use std::env;

const EMBED_FEATURE: &str = "CARGO_FEATURE_EMBED";
const EXPR_FEATURE: &str = "CARGO_FEATURE_EXPR";
const STORE_FEATURE: &str = "CARGO_FEATURE_STORE";

fn main() {
    let has_embed = env::var_os(EMBED_FEATURE).is_some();
    let has_expr = env::var_os(EXPR_FEATURE).is_some();
    let has_store = env::var_os(STORE_FEATURE).is_some();

    assert!(
        has_expr || has_store,
        "nixb-cpp requires at least one of the `expr` or `store` features"
    );

    let mut build = cc::Build::new();

    if has_expr {
        let nix_expr = pkg_config::Config::new()
            .cargo_metadata(has_embed)
            .probe("nix-expr")
            .expect("Could not find nix-expr via pkg-config");

        for include_path in &nix_expr.include_paths {
            build.include(include_path);
        }

        build.file("cpp/wrapper.cpp").file("cpp/function.cpp");
    }

    if has_store {
        let nix_store = pkg_config::Config::new()
            .cargo_metadata(has_embed)
            .probe("nix-store")
            .expect("Could not find nix-store via pkg-config");

        for include_path in &nix_store.include_paths {
            build.include(include_path);
        }

        build.file("cpp/store.cpp");
    }

    build.cpp(true).flag("-std=c++23").compile("nixb_cpp");

    println!("cargo:rerun-if-changed=cpp/wrapper.cpp");
    println!("cargo:rerun-if-changed=cpp/function.cpp");
    println!("cargo:rerun-if-changed=cpp/store.cpp");
    println!("cargo:rerun-if-env-changed={EMBED_FEATURE}");
    println!("cargo:rerun-if-env-changed={EXPR_FEATURE}");
    println!("cargo:rerun-if-env-changed={STORE_FEATURE}");
}
