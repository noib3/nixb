use nixb_contexts::c_context::CContext;

/// TODO: docs.
#[inline]
pub fn init<const LOAD_CONFIG: bool>(
    ctx: &mut CContext,
) -> nixb_error::Result<InitSentinel> {
    ctx.with_ptr(|ctx| {
        if LOAD_CONFIG {
            unsafe { nixb_sys::libstore_init(ctx) }
        } else {
            unsafe { nixb_sys::libstore_init_no_load_config(ctx) }
        }
    })?;
    Ok(InitSentinel {})
}

/// TODO: docs.
pub struct InitSentinel {}
