//! TODO: docs.

use alloc::ffi::CString;

use crate::attrset::NixAttrset;
use crate::callable::NixLambda;
use crate::context::Context;
use crate::value::Borrowed;

/// TODO: docs.
pub struct Builtins<'eval> {
    inner: NixAttrset<Borrowed<'eval>>,
}

impl<'eval> Builtins<'eval> {
    /// Returns a handle to the `builtins.currentSystem` string.
    #[inline]
    pub fn current_system(&self, ctx: &mut Context) -> CString {
        self.inner
            .get(c"currentSystem", ctx)
            .expect("builtins.currentSystem exists and it's a string")
    }

    /// Returns a handle to the `builtins.derivation` function.
    #[inline]
    pub fn derivation(&self, ctx: &mut Context) -> NixLambda<Borrowed<'eval>> {
        self.inner
            .get(c"derivation", ctx)
            .expect("builtins.derivation exists and it's a function")
    }

    /// Returns a handle to the `builtins.derivationStrict` function.
    #[inline]
    pub fn derivation_strict(
        &self,
        ctx: &mut Context,
    ) -> NixLambda<Borrowed<'eval>> {
        self.inner
            .get(c"derivationStrict", ctx)
            .expect("builtins.derivationStrict exists and it's a function")
    }

    /// Returns a handle to the `builtins.fetchGit` function.
    #[inline]
    pub fn fetch_git(&self, ctx: &mut Context) -> NixLambda<Borrowed<'eval>> {
        self.inner
            .get(c"fetchGit", ctx)
            .expect("builtins.fetchGit exists and it's a function")
    }

    /// Returns a handle to the `builtins.path` function.
    #[inline]
    pub fn path(&self, ctx: &mut Context) -> NixLambda<Borrowed<'eval>> {
        self.inner
            .get(c"path", ctx)
            .expect("builtins.path exists and it's a function")
    }

    /// Returns a handle to the `builtins.throw` function.
    #[inline]
    pub fn throw(&self, ctx: &mut Context) -> NixLambda<Borrowed<'eval>> {
        self.inner
            .get(c"throw", ctx)
            .expect("builtins.throw exists and it's a function")
    }

    #[inline]
    pub(crate) fn new(inner: NixAttrset<Borrowed<'eval>>) -> Self {
        Self { inner }
    }
}

impl<'eval> From<Builtins<'eval>> for NixAttrset<Borrowed<'eval>> {
    #[inline]
    fn from(builtins: Builtins<'eval>) -> Self {
        builtins.inner
    }
}
