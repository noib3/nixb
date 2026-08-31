#![expect(missing_docs)]

fn main() {
    pkg_config::Config::new()
        .probe("nix-flake-c")
        .expect("Could not find nix-flake-c via pkg-config");
}
