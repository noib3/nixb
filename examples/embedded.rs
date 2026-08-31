//! Embeds Nix in a standalone executable, opens a store, creates an evaluator
//! state, and evaluates an expression.

use nixb::expr::EvalState;
use nixb::prelude::{Context, IntoValue, Value};
use nixb::store::Store;

#[derive(nixb::expr::PrimOp, Clone, Copy)]
struct EmbeddedPrimOp;

fn main() -> nixb::Result<()> {
    let mut init = nixb::expr::init()?;
    nixb::expr::register_primop(EmbeddedPrimOp, &mut init)?;

    let mut store = Store::open(init.clone().into(), c"dummy://", [])?;

    let mut state = EvalState::new(init, [], &mut store)?;

    let mut ctx = state.context();

    let two = ctx.eval::<i64>(c"1 + 1")?;
    let answer = ctx.eval::<i64>(c"builtins.embeddedPrimOp")?;

    assert_eq!(two, 2);
    assert_eq!(answer, 42);

    println!("1 + 1 = {two}");

    Ok(())
}

impl IntoValue for EmbeddedPrimOp {
    #[inline]
    fn into_value(self, _: &mut Context) -> impl Value + use<> {
        42
    }
}
