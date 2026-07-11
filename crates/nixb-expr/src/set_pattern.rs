//! Implementation details for the [`SetPattern`](nixb_macros::SetPattern)
//! derive macro.

use core::ffi::{CStr, c_uint};

use nixb_error::{Error, Result};

use crate::attrset::{MergeableAttrset, NixAttrset};
use crate::context::Context;
use crate::value::{Borrowed, NixValue, TryFromValue};

/// A type which can match part or all of a Nix function set pattern.
pub trait SetPattern<'a>: Sized {
    /// Whether unmatched attributes are accepted when this pattern is matched
    /// at the top level.
    const ELLIPSIS: bool;

    /// TODO: docs.
    const HAS_FLATTENED_FIELDS: bool;

    /// Returns how many formals in this pattern have the name `key`.
    fn formal_match_count(key: &CStr) -> usize;

    /// Matches this pattern against `input` without checking for unmatched
    /// attributes.
    fn match_pattern<const CHECK_DUPLICATES: bool>(
        input: &mut SetPatternInput<'a, CHECK_DUPLICATES>,
        ctx: &mut Context,
    ) -> Result<Self>;
}

/// An attribute set being matched by one or more set-pattern fragments.
pub struct SetPatternInput<'a, const CHECK_DUPLICATES: bool> {
    attrset: NixAttrset<Borrowed<'a>>,
    formal_match_count: fn(&CStr) -> usize,
    matched: c_uint,
}

impl<'a, const CHECK_DUPLICATES: bool> SetPatternInput<'a, CHECK_DUPLICATES> {
    /// Creates a set-pattern input around `attrset`.
    #[inline]
    pub fn new(
        attrset: NixAttrset<Borrowed<'a>>,
        formal_match_count: fn(&CStr) -> usize,
    ) -> Self {
        Self { attrset, formal_match_count, matched: 0 }
    }

    /// Claims `key` and returns its converted value when present.
    pub fn take<T>(
        &mut self,
        key: &'static CStr,
        ctx: &mut Context,
    ) -> Result<Option<T>>
    where
        T: TryFromValue<NixValue<Borrowed<'a>>>,
    {
        if CHECK_DUPLICATES {
            match (self.formal_match_count)(key) {
                1 => {},
                0 => {
                    return Err(Error::from_message(core::format_args!(
                        "set pattern attempted to take undeclared formal '{}'",
                        key.to_string_lossy(),
                    )));
                },
                _ => {
                    return Err(Error::from_message(core::format_args!(
                        "duplicate formal function argument '{}'",
                        key.to_string_lossy(),
                    )));
                },
            }
        }

        let value = self.attrset.get_opt(key, ctx)?;
        self.matched += u32::from(value.is_some());
        Ok(value)
    }

    /// Checks that every input attribute was claimed unless `ellipsis` is
    /// enabled.
    pub fn finish(&self, ellipsis: bool, ctx: &mut Context) -> Result<()> {
        if ellipsis || self.attrset.len(ctx) == self.matched {
            return Ok(());
        }

        let mut unexpected = None;
        self.attrset.for_each_key(
            |key, _| {
                if unexpected.is_none() && (self.formal_match_count)(key) == 0 {
                    unexpected = Some(Error::from_message(core::format_args!(
                        "function called with unexpected argument '{}'",
                        key.to_string_lossy(),
                    )));
                }
            },
            ctx,
        );

        let Some(unexpected) = unexpected else {
            core::unreachable!(
                "attribute count differs, so an unexpected argument exists",
            );
        };
        Err(unexpected)
    }
}

impl<'a, T: SetPattern<'a>> TryFromValue<NixValue<Borrowed<'a>>> for T {
    fn try_from_value(
        value: NixValue<Borrowed<'a>>,
        ctx: &mut Context,
    ) -> Result<Self> {
        NixAttrset::try_from_value(value, ctx)
            .and_then(|attrset| Self::try_from_value(attrset, ctx))
    }
}

impl<'a, T: SetPattern<'a>> TryFromValue<NixAttrset<Borrowed<'a>>> for T {
    fn try_from_value(
        attrset: NixAttrset<Borrowed<'a>>,
        ctx: &mut Context,
    ) -> Result<Self> {
        #[inline]
        fn inner<'a, T: SetPattern<'a>, const CHECK_DUPLICATES: bool>(
            attrset: NixAttrset<Borrowed<'a>>,
            ctx: &mut Context,
        ) -> Result<T> {
            let mut input = SetPatternInput::<CHECK_DUPLICATES>::new(
                attrset,
                T::formal_match_count,
            );
            let this = T::match_pattern(&mut input, ctx)?;
            input.finish(T::ELLIPSIS, ctx)?;
            Ok(this)
        }

        if T::HAS_FLATTENED_FIELDS {
            inner::<T, true>(attrset, ctx)
        } else {
            inner::<T, false>(attrset, ctx)
        }
    }
}
