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
