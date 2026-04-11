//! TODO: docs.

use crate::attrset::NixAttrset;
use crate::context::Context;
use crate::prelude::NixLambda;
use crate::value::Borrowed;

/// TODO: docs.
pub struct Builtins<'eval> {
    inner: NixAttrset<Borrowed<'eval>>,
}

impl<'eval> Builtins<'eval> {
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
