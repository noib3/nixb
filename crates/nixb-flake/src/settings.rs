use core::cell::RefCell;
use core::ptr::NonNull;

use nixb_c_context::CContext;
use nixb_error::Result;
use nixb_expr::eval_state::EvalStateBuilder;

/// Settings controlling Nix flake behavior.
pub struct FlakeSettings {
    ctx: RefCell<CContext>,
    ptr: NonNull<nixb_sys::flake_settings>,
}

impl FlakeSettings {
    /// Adds flake builtins such as `builtins.getFlake` to an evaluator
    /// builder.
    ///
    /// This does not put the resulting evaluator state in pure mode.
    #[inline]
    pub fn add_to_eval_state_builder(
        &self,
        builder: &mut EvalStateBuilder,
    ) -> Result<()> {
        self.ctx.borrow_mut().with_ptr(|ctx| unsafe {
            nixb_sys::flake_settings_add_to_eval_state_builder(
                ctx,
                self.as_ptr(),
                builder.as_ptr(),
            )
        })?;

        Ok(())
    }

    /// Creates settings initialized with Nix's default values.
    #[inline]
    pub fn new() -> Result<Self> {
        let mut ctx = CContext::create();
        let settings =
            ctx.with_ptr(|ctx| unsafe { nixb_sys::flake_settings_new(ctx) })?;

        let ptr = NonNull::new(settings).expect(
            "nix_flake_settings_new returned null without setting an error",
        );

        Ok(Self { ctx: RefCell::new(ctx), ptr })
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut nixb_sys::flake_settings {
        self.ptr.as_ptr()
    }
}

impl Drop for FlakeSettings {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::flake_settings_free(self.ptr.as_ptr()) };
    }
}
