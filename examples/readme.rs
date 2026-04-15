use nixb::prelude::*;

/// A cool Nix plugin.
#[derive(nixb::PrimOp, Clone, Copy)]
struct MyPlugin;

#[derive(Default, nixb::Attrset)]
struct MyAttrset {
    field1: String,
    field2: u8,
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
            mkHello: function::<String>(|name| format!("Hello {name}!")),
            lazyEval2: thunk(|| 42),
        }
    }
}

#[nixb::plugin]
fn myplugin(ctx: &mut Context<Entrypoint>) {
    ctx.register_primop(MyPlugin);
}
