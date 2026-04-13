{
  coreutils,
  git,
  writeShellApplication,
  # --
  sys-bindings,
}:

writeShellApplication {
  name = "update-sys-bindings";
  runtimeInputs = [
    coreutils
    git
  ];
  text = ''
    repo_root="$(git rev-parse --show-toplevel)"
    bindings_dir="$repo_root/crates/nixb-sys/src/bindings"

    for path in "$bindings_dir"/*; do
      if [ "$(basename "$path")" != "mod.rs" ]; then
        rm -rf "$path"
      fi
    done

    cp ${sys-bindings}/*.rs "$bindings_dir/"
  '';
}
