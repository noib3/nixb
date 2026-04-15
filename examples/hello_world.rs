use nixb::plugin::{ContextExt, Entrypoint};
use nixb::prelude::*;

/// A simple primop.
#[derive(nixb::plugin::PrimOp, Clone, Copy)]
struct HelloWorld;

impl IntoValue for HelloWorld {
    #[inline]
    fn into_value(self, _: &mut Context) -> impl Value + use<> {
        "Hello, world!"
    }
}

#[nixb::plugin::entry]
fn hello_world(ctx: &mut Context<Entrypoint>) {
    ctx.register_primop(HelloWorld);
}
