//! Embeds Nix in a standalone executable, opens a store, creates an evaluator
//! state, and evaluates an expression.

use nixb::expr::EvalState;
use nixb::store::Store;

fn main() -> nixb::Result<()> {
    let init = nixb::expr::init()?;

    let mut store = Store::open(init.clone().into(), c"dummy://", [])?;

    let mut state = EvalState::new(init, [], &mut store)?;

    let mut ctx = state.context();

    let two = ctx.eval::<i64>(c"1 + 1")?;

    assert_eq!(two, 2);

    println!("1 + 1 = {two}");

    Ok(())
}
