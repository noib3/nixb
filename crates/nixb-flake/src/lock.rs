use core::ffi::CStr;
use core::ptr::NonNull;

use nixb_c_context::CContext;
use nixb_error::Result;
use nixb_expr::attrset::NixAttrset;
use nixb_expr::context::Context;
use nixb_expr::value::{NixValue, Owned, TryFromValue, ValueOwner};

use crate::reference::FlakeReference;
use crate::settings::FlakeSettings;

/// A flake whose input graph has been locked.
pub struct LockedFlake {
    inner: NonNull<nixb_sys::locked_flake>,
}

/// Parameters controlling how a flake is locked.
pub struct FlakeLockFlags {
    ctx: CContext,
    inner: NonNull<nixb_sys::flake_lock_flags>,
}

impl LockedFlake {
    /// Evaluates and returns this flake's output attribute set.
    #[inline]
    pub fn output_attrs(
        &self,
        settings: &FlakeSettings,
        ctx: &mut Context<'_>,
    ) -> Result<NixAttrset> {
        let value = ctx.with_raw_and_state(|raw_ctx, state| unsafe {
            nixb_sys::locked_flake_get_output_attrs(
                raw_ctx,
                settings.as_ptr(),
                state,
                self.inner.as_ptr(),
            )
        })?;

        let value = NonNull::new(value).expect(
            "nix_locked_flake_get_output_attrs returned null without setting \
             an error",
        );
        let value = NixValue::new(unsafe { Owned::new(value) });

        NixAttrset::try_from_value(value, ctx)
    }

    #[inline]
    pub(crate) fn new(inner: NonNull<nixb_sys::locked_flake>) -> Self {
        Self { inner }
    }
}

impl FlakeLockFlags {
    /// Overrides a non-empty input path with another flake reference.
    ///
    /// Unless the flags are in check mode, adding an override switches them
    /// to virtual mode so that the override is not written to disk.
    #[inline]
    pub fn add_input_override(
        &mut self,
        input_path: impl AsRef<CStr>,
        override_ref: &FlakeReference,
    ) -> Result<&mut Self> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_lock_flags_add_input_override(
                ctx,
                self.inner.as_ptr(),
                input_path.as_ref().as_ptr(),
                override_ref.as_ptr(),
            )
        })?;

        Ok(self)
    }

    /// Creates lock parameters initialized with Nix's default values.
    #[inline]
    pub fn new(settings: &FlakeSettings) -> Result<Self> {
        let mut ctx = CContext::create();
        let flags = ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_lock_flags_new(ctx, settings.as_ptr())
        })?;

        let inner = NonNull::new(flags).expect(
            "nix_flake_lock_flags_new returned null without setting an error",
        );

        Ok(Self { ctx, inner })
    }

    /// Makes locking fail if the existing lock needs to be updated.
    #[inline]
    pub fn set_mode_check(&mut self) -> Result<&mut Self> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_lock_flags_set_mode_check(ctx, self.inner.as_ptr())
        })?;

        Ok(self)
    }

    /// Allows updating the lock in memory without writing it to disk.
    #[inline]
    pub fn set_mode_virtual(&mut self) -> Result<&mut Self> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_lock_flags_set_mode_virtual(
                ctx,
                self.inner.as_ptr(),
            )
        })?;

        Ok(self)
    }

    /// Allows updating and writing the lock file when necessary.
    #[inline]
    pub fn set_mode_write_as_needed(&mut self) -> Result<&mut Self> {
        self.ctx.with_ptr(|ctx| unsafe {
            nixb_sys::flake_lock_flags_set_mode_write_as_needed(
                ctx,
                self.inner.as_ptr(),
            )
        })?;

        Ok(self)
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut nixb_sys::flake_lock_flags {
        self.inner.as_ptr()
    }
}

impl Drop for LockedFlake {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::locked_flake_free(self.inner.as_ptr()) };
    }
}

impl Drop for FlakeLockFlags {
    #[inline]
    fn drop(&mut self) {
        unsafe { nixb_sys::flake_lock_flags_free(self.inner.as_ptr()) };
    }
}
