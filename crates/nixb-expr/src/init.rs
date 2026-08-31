use nixb_c_context::CContext;

/// Initializes the Nix language evaluator by calling `nix_libexpr_init`.
///
/// This must be called before creating an [`EvalState`](crate::EvalState),
/// which the returned sentinel guarantees at the type level.
///
/// This function is idempotent and can be called multiple times.
#[inline]
pub fn init() -> nixb_error::Result<InitSentinel> {
    let mut ctx = CContext::create();
    ctx.with_ptr(|ctx| unsafe { nixb_sys::libexpr_init(ctx) })?;
    Ok(InitSentinel { ctx })
}

/// Proof that [`init`] was called.
pub struct InitSentinel {
    pub(crate) ctx: CContext,
}

impl Clone for InitSentinel {
    fn clone(&self) -> Self {
        Self { ctx: CContext::create() }
    }
}

/// `nix_libexpr_init` transitively runs `nix_libstore_init`, so this sentinel
/// is strictly stronger than `nixb-store`'s.
impl From<InitSentinel> for nixb_store::InitSentinel {
    #[inline]
    fn from(init: InitSentinel) -> Self {
        Self::new(init.ctx)
    }
}
