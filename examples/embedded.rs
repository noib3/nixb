//! Embeds Nix in a standalone executable instead of being loaded as a
//! plugin: opens a store, creates an evaluator state, and evaluates an
//! expression.

use core::ffi::CStr;

use nixb::expr::EvalState;
use nixb::store::Store;

fn main() -> nixb::Result<()> {
    let init = nixb::expr::init()?;

    let mut store = Store::open(init.into(), c"dummy://", [])?;

    // `init` is idempotent, and opening the store consumed the previous
    // sentinel.
    let init = nixb::expr::init()?;

    let lookup_path: [&CStr; 0] = [];

    let mut state = EvalState::new(init, lookup_path, &mut store)?;

    let mut ctx = state.context();

    let two = ctx.eval::<i64>(c"1 + 1")?;

    assert_eq!(two, 2);

    println!("1 + 1 = {two}");

    Ok(())
}
