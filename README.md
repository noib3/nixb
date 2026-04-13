## Quick start

1) create a new Rust crate which compiles into a shared library

```sh
cargo new --lib myplugin
cargo add nixb
cargo <add dylib to targets>
```

2) write a plugin:

```rust
// src/lib.rs

use nixb::prelude::*;

/// A cool Nix plugin.
#[derive(nixb::PrimOp, Clone, Copy)]
struct MyPlugin;

#[derive(Default, nixb::Attrset)]
struct MyAttrset {
    field1: String,
    field2: u8,
}

fn mk_hello(name: String, _ctx: &mut Context) -> String {
    format!("Hello {name}!")
}

fn expensive_computation(_ctx: &mut Context) -> u8 {
    std::thread::sleep(std::time::Duration::from_secs(2));
    42
}

impl IntoValue for MyPlugin {
    fn into_value(self, _: &mut Context) -> impl Value + use<> {
        attrset! {
            listSameType: ["foo", "baz", "baz"],
            listDifferentTypes: list!["string", 42],
            myAttrset: MyAttrset::default(),
            mkHello: function::<String>(mk_hello),
            lazyEval: Thunk::into_value(expensive_computation),
        }
    }
}

#[nixb::plugin]
fn myplugin(ctx: &mut Context<Entrypoint>) {
    ctx.register_primop(MyPlugin);
}
```

3) Build the plugin and launch a REPL with the plugin's API:

```sh
cargo b && nix repl --option plugin-files ./target/debug/libmyplugin.{so|dylib}
```

```nix
nix-repl> p = builtins.myPlugin

nix-repl> p.listSameType
[
  "foo"
  "baz"
  "baz"
]

nix-repl> p.listDifferentTypes
[
  "string"
  42
]

nix-repl> p.myAttrset
{
  field1 = "";
  field2 = 0;
}

nix-repl> p.mkHello "John"
"Hello John!"

nix-repl> p.lazyEval
42
```
