#![expect(missing_docs)]

fn main() {
    pkg_config::Config::new()
        .probe("nix-fetchers-c")
        .expect("Could not find nix-fetchers-c via pkg-config");
}
