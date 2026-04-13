{
  coreutils,
  git,
  rust-script,
  writeShellApplication,
  # --
  rust,
}:

writeShellApplication {
  name = "update-bindings-generator-lockfile";
  runtimeInputs = [
    coreutils
    git
    rust
    rust-script
  ];
  text = ''
    repo_root="$(git rev-parse --show-toplevel)"
    tmpdir="$(mktemp -d)"
    trap 'rm -rf "$tmpdir"' EXIT

    rust-script --package --pkg-path "$tmpdir/pkg" "$repo_root/nix/sys_bindings_generator.rs" > /dev/null
    cargo generate-lockfile --manifest-path "$tmpdir/pkg/Cargo.toml"
    cp "$tmpdir/pkg/Cargo.lock" "$repo_root/nix/sys_bindings_generator_Cargo.lock"
  '';
}
