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

impl IntoValue for MyPlugin {
    fn into_value(self, _: &mut Context) -> impl Value + use<> {
        attrset! {
            listSameType: ["foo", "baz", "baz"],
            listDifferentTypes: list!["string", 42],
            myAttrset: MyAttrset::default(),
            mkHello: function::<String>(mk_hello),
        }
    }
}

#[nixb::plugin]
fn myplugin(ctx: &mut Context<Entrypoint>) {
    ctx.register_primop(MyPlugin);
}
