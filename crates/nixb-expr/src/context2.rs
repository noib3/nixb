use core::ffi::{CStr, c_uint};
use core::ptr::{self, NonNull};

use nixb_result::Result;

use crate::attrset::{NixAttrset, NixDerivation};
use crate::callable::{Callable, NixLambda};
use crate::value::{NixValue, ValueOwner};

/// TODO: docs.
pub trait ExprContext: nixb_context::Context {
    /// TODO: docs.
    fn get_attr_byname_lazy<'a, O: ValueOwner>(
        &mut self,
        attr: &'a NixAttrset<O>,
        key: &CStr,
    ) -> Option<NixValue<O::Borrow<'a>>>;

    /// TODO: docs.
    fn get_attrs_size<O: ValueOwner>(&mut self, attr: &NixAttrset<O>)
    -> c_uint;

    /// TODO: docs.
    fn realise_derivation<O: ValueOwner>(
        &mut self,
        drv: &NixDerivation<O>,
    ) -> Result<()>;
}

impl ExprContext for crate::context::Context<'_> {
    #[inline]
    fn get_attr_byname_lazy<'a, O: ValueOwner>(
        &mut self,
        attr: &'a NixAttrset<O>,
        key: &CStr,
    ) -> Option<NixValue<O::Borrow<'a>>> {
        let value_raw = unsafe {
            nixb_cpp::get_attr_byname_lazy_no_incref(
                attr.inner.as_ptr(),
                self.as_ptr(),
                key.as_ptr(),
            )
        };

        NonNull::new(value_raw)
            .map(|ptr| unsafe { O::Borrow::new(ptr) })
            .map(NixValue::new)
    }

    #[inline]
    fn get_attrs_size<O: ValueOwner>(
        &mut self,
        attr: &NixAttrset<O>,
    ) -> c_uint {
        // 'nix_get_attrs_size' errors when the value pointer is null or when
        // the value is not initizialized, but having a NixValue guarantees
        // neither of those can happen, so we can use a null context.
        unsafe {
            nixb_sys::get_attrs_size(ptr::null_mut(), attr.inner.as_ptr())
        }
    }

    #[inline]
    fn realise_derivation<O: ValueOwner>(
        &mut self,
        drv: &NixDerivation<O>,
    ) -> Result<()> {
        let expr = c"drv: \"${drv}\"";
        let string_drv =
            self.eval::<NixLambda>(expr)?.call(drv.inner.borrow(), self)?;
        let value = string_drv.into_inner();
        let realised_str = self.with_raw_and_state(|ctx, state| unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            {
                nixb_cpp::string_realise(
                    ctx,
                    state.as_ptr(),
                    value.as_ptr(),
                    true,
                )
            }

            #[cfg(feature = "nix-2-34")]
            nixb_sys::string_realise(ctx, state.as_ptr(), value.as_ptr(), true)
        })?;
        unsafe {
            nixb_sys::realised_string_free(realised_str);
        }
        Ok(())
    }
}

impl nixb_context::Context for crate::context::Context<'_> {
    #[inline]
    fn as_ptr(&mut self) -> *mut nixb_sys::c_context {
        todo!()
    }
}
