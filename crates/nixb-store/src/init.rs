use nixb_c_context::CContext;

/// TODO: docs.
#[inline]
pub fn init<const LOAD_CONFIG: bool>() -> nixb_error::Result<InitSentinel> {
    let mut ctx = CContext::create();

    ctx.with_ptr(|ctx| {
        if LOAD_CONFIG {
            unsafe { nixb_sys::libstore_init(ctx) }
        } else {
            unsafe { nixb_sys::libstore_init_no_load_config(ctx) }
        }
    })?;

    Ok(InitSentinel { ctx })
}

/// TODO: docs.
pub struct InitSentinel {
    pub(crate) ctx: CContext,
}

impl InitSentinel {
    /// Creates a sentinel without initializing `libstore`.
    ///
    /// Only meant to be used by other `nixb-*` crates whose initialization
    /// routines are documented by the C API to transitively run
    /// `nix_libstore_init` (like `nixb-expr`'s).
    #[doc(hidden)]
    #[inline]
    pub fn new(ctx: CContext) -> Self {
        Self { ctx }
    }
}
