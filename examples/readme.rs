use nixb::plugin::{ContextExt, Entrypoint};
use nixb::prelude::*;

/// A cool Nix plugin.
#[derive(nixb::plugin::PrimOp, Clone, Copy)]
struct MyPlugin;

#[derive(Default, nixb::expr::Attrset)]
struct MyAttrset {
    field1: String,
    field2: u8,
}

impl IntoValue for MyPlugin {
    fn into_value(self, _: &mut Context) -> impl Value + use<> {
        attrset! {
            listSameType: ["foo", "baz", "baz"],
            listDifferentTypes: list!["string", 42],
            myAttrset: MyAttrset::default(),
            mkHello: function(|name: String| format!("Hello {name}!")),
            lazyEval2: thunk(|| {
                std::thread::sleep(std::time::Duration::from_secs(2));
                42
            }),
        }
    }
}

#[nixb::plugin::entry]
fn myplugin(ctx: &mut Context<Entrypoint>) {
    ctx.register_primop(MyPlugin);
}
