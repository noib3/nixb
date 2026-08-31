use nixb_c_context::CContext;
use nixb_expr::primop::PrimOp;

/// TODO: docs.
pub struct Plugin {
    ctx: CContext,
}

impl Plugin {
    /// TODO: docs.
    #[track_caller]
    #[inline]
    pub fn register_primop<P: PrimOp>(&mut self, primop: P) -> &mut Self {
        if let Err(err) = primop.register(&mut self.ctx) {
            panic!("couldn't register primop {:?}: {err}", P::NAME);
        }
        self
    }

    #[doc(hidden)]
    #[inline]
    pub fn new(ctx: CContext) -> Self {
        Self { ctx }
    }
}
