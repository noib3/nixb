//! TODO: docs.

use alloc::borrow::{Cow, ToOwned};
use alloc::ffi::CString;
use alloc::vec::Vec;
use core::ffi::{CStr, c_char, c_uint, c_void};
use core::marker::PhantomData;
use core::ptr::{self, NonNull};
use core::{fmt, slice};

use nixb_error::{Error, Result};

use crate::context::Context;
use crate::error::{TryFromI64Error, TypeMismatchError};
use crate::list::{List, NixList};
use crate::tuple::{RecursiveTuple, Tuple};

/// TODO: docs.
pub trait Value {
    /// Returns the kind of this value.
    fn kind(&self) -> ValueKind;

    /// Writes this value into the given destination.
    fn write(self, dest: UninitValue, ctx: &mut Context);

    /// TODO: docs.
    #[inline(always)]
    fn force_inline(&mut self, _ctx: &mut Context) -> Result<()> {
        Ok(())
    }
}

/// TODO: docs.
pub trait BoolValue: Value + Sized {
    /// # Safety
    ///
    /// This method should only be called after a successful call to
    /// [`kind`](Value::kind) returns [`ValueKind::Bool`].
    unsafe fn into_bool(self, ctx: &mut Context) -> Result<bool>;
}

/// TODO: docs.
pub trait IntValue: Value + Sized {
    /// # Safety
    ///
    /// This method should only be called after a successful call to
    /// [`kind`](Value::kind) returns [`ValueKind::Int`].
    unsafe fn into_int(self, ctx: &mut Context) -> i64;
}

/// TODO: docs.
pub trait StringValue: Value + Sized {
    /// TODO: docs.
    type String;

    /// # Safety
    ///
    /// This method should only be called after a successful call to
    /// [`kind`](Value::kind) returns [`ValueKind::String`].
    unsafe fn into_string(self, ctx: &mut Context) -> Result<Self::String>;
}

/// TODO: docs.
pub trait PathValue: Value + Sized {
    /// TODO: docs.
    type Path: AsRef<CStr>;

    /// # Safety
    ///
    /// This method should only be called after a successful call to
    /// [`kind`](Value::kind) returns [`ValueKind::Path`].
    unsafe fn into_path_string(self, ctx: &mut Context) -> Result<Self::Path>;
}

/// A trait for types that can be infallibly converted into [`Value`]s.
pub trait IntoValue {
    /// Converts `self` into a [`Value`].
    fn into_value<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl Value + use<'eval, Self>;
}

/// A trait for types that can be fallibly converted from [`Value`]s.
pub trait TryFromValue<V: Value>: Sized {
    /// TODO: docs.
    fn try_from_value(value: V, ctx: &mut Context) -> Result<Self>;
}

/// TODO: docs.
pub trait Values:
    Tuple<
        First: IntoValue,
        Last: IntoValue,
        FromFirst = <Self as Values>::FromFirst,
        UpToLast = <Self as Values>::UpToLast,
    >
{
    /// TODO: docs.
    type FromFirst: Values;

    /// TODO: docs.
    type UpToLast: Values;
}

/// TODO: docs.
pub trait ValueOwner: Into<Owned> {
    /// TODO: docs.
    type Borrow<'a>: ValueOwner + IsBorrowedOfAtLeast<'a>
    where
        Self: 'a;

    /// TODO: docs.
    fn borrow<'a>(&'a self) -> Self::Borrow<'a>;

    /// TODO: docs.
    ///
    /// # Safety
    ///
    /// TODO: docs.
    unsafe fn new(value_ptr: NonNull<nixb_sys::Value>) -> Self;

    /// TODO: docs.
    fn value_ptr(&self) -> NonNull<nixb_sys::Value>;
}

#[doc(hidden)]
pub trait IsBorrowedOfAtLeast<'a> {
    fn into_borrowed(self) -> Borrowed<'a>;
}

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct NixValue<Owner = Owned> {
    owner: Owner,
}

/// TODO: docs.
pub struct Owned {
    ptr: NonNull<nixb_sys::Value>,
}

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct Borrowed<'a> {
    ptr: NonNull<nixb_sys::Value>,
    _lifetime: PhantomData<&'a nixb_sys::Value>,
}

/// TODO: docs.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Null;

/// TODO: docs.
#[derive(Copy, Clone)]
pub struct UninitValue {
    value_ptr: NonNull<nixb_sys::Value>,
}

/// TODO: docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ValueKind {
    /// TODO: docs.
    Attrset,

    /// TODO: docs.
    Bool,

    /// TODO: docs.
    External,

    /// TODO: docs.
    Float,

    /// TODO: docs.
    Function,

    /// TODO: docs.
    Int,

    /// TODO: docs.
    List,

    /// TODO: docs.
    Null,

    /// TODO: docs.
    Path,

    /// TODO: docs.
    String,

    /// TODO: docs.
    Thunk,
}

/// TODO: docs.
#[derive(Debug, Copy, Clone)]
pub struct IntoValueFn<F, T> {
    f: F,
    _output: PhantomData<T>,
}

impl<Owner: ValueOwner> NixValue<Owner> {
    /// TODO: docs.
    #[inline]
    pub fn as_borrowed(&self) -> NixValue<Borrowed<'_>> {
        NixValue::new(self.owner.borrow().into_borrowed())
    }

    /// TODO: docs.
    #[inline]
    pub fn borrow(&self) -> NixValue<Owner::Borrow<'_>> {
        NixValue::new(self.owner.borrow())
    }

    /// TODO: docs.
    #[inline]
    pub fn into_owned(self) -> NixValue<Owned> {
        NixValue::new(self.owner.into())
    }

    #[inline]
    pub(crate) fn as_ptr(&self) -> *mut nixb_sys::Value {
        self.owner.value_ptr().as_ptr()
    }

    #[inline]
    pub(crate) fn new(owner: Owner) -> Self {
        Self { owner }
    }

    #[inline]
    pub(crate) fn owner(&self) -> &Owner {
        &self.owner
    }

    /// Calls the given callback with the string held by this value.
    ///
    /// # Safety
    ///
    /// The caller must first ensure that this value's kind is
    /// [`ValueKind::String`].
    #[inline]
    pub(crate) unsafe fn with_string(
        &self,
        mut fun: impl FnMut(&CStr),
        ctx: &mut Context,
    ) -> Result<()> {
        unsafe extern "C" fn get_string_callback(
            start: *const c_char,
            n: c_uint,
            fun_ref: *mut c_void,
        ) {
            let num_bytes_including_nul = n + 1;
            let bytes = unsafe {
                slice::from_raw_parts(
                    start as *const u8,
                    num_bytes_including_nul as usize,
                )
            };
            let c_str = unsafe { CStr::from_bytes_with_nul_unchecked(bytes) };
            let fun = unsafe { &mut **(fun_ref as *mut &mut dyn FnMut(&CStr)) };
            fun(c_str);
        }

        let mut fun_ref = &mut fun as &mut dyn FnMut(&CStr);

        ctx.with_raw(|ctx| unsafe {
            nixb_sys::get_string(
                ctx,
                self.as_ptr(),
                Some(get_string_callback),
                &mut fun_ref as *mut &mut dyn FnMut(&CStr) as *mut c_void,
            );
        })
    }
}

impl UninitValue {
    /// TODO: docs.
    #[inline(always)]
    pub fn as_non_null(self) -> NonNull<nixb_sys::Value> {
        self.value_ptr
    }

    /// TODO: docs.
    #[inline(always)]
    pub fn as_ptr(self) -> *mut nixb_sys::Value {
        self.value_ptr.as_ptr()
    }

    /// TODO: docs.
    ///
    /// # Safety
    ///
    /// TODO: docs.
    #[inline(always)]
    pub(crate) unsafe fn new(value_ptr: NonNull<nixb_sys::Value>) -> Self {
        Self { value_ptr }
    }
}

impl<F, T> IntoValueFn<F, T>
where
    F: FnOnce(&mut Context) -> T,
    T: Value,
{
    /// Creates a new [`IntoValueFn`] that wraps the given closure.
    #[inline]
    pub fn new(f: F) -> Self {
        Self { f, _output: PhantomData }
    }
}

impl Value for Null {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Null
    }

    #[inline]
    fn write(self, dest: UninitValue, _: &mut Context) {
        // `nix_init_null` errors when:
        //
        // 1. the destination pointer is null;
        // 2. the destination value is already initialized.
        //
        // Having an `UninitValue` guards against both, so neither can happen.
        unsafe {
            nixb_sys::init_null(ptr::null_mut(), dest.as_ptr());
        }
    }
}

impl<Owner> fmt::Debug for NixValue<Owner> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("NixValue").finish_non_exhaustive()
    }
}

impl<Owner: ValueOwner> Value for NixValue<Owner> {
    #[inline]
    fn force_inline(&mut self, ctx: &mut Context) -> Result<()> {
        ctx.with_raw_and_state(|ctx, state| unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            nixb_cpp::force_value(ctx, state.as_ptr(), self.as_ptr());

            #[cfg(feature = "nix-2-34")]
            nixb_sys::value_force(ctx, state.as_ptr(), self.as_ptr());
        })
    }

    #[inline]
    fn kind(&self) -> ValueKind {
        // 'nix_get_type' errors when the value pointer is null or when the
        // value is not initizialized, but having a NixValue guarantees neither
        // of those can happen, so we can use a null context.
        let r#type =
            unsafe { nixb_sys::get_type(ptr::null_mut(), self.as_ptr()) };

        match r#type {
            nixb_sys::ValueType_NIX_TYPE_ATTRS => ValueKind::Attrset,
            nixb_sys::ValueType_NIX_TYPE_BOOL => ValueKind::Bool,
            nixb_sys::ValueType_NIX_TYPE_EXTERNAL => ValueKind::External,
            nixb_sys::ValueType_NIX_TYPE_FLOAT => ValueKind::Float,
            nixb_sys::ValueType_NIX_TYPE_FUNCTION => ValueKind::Function,
            nixb_sys::ValueType_NIX_TYPE_INT => ValueKind::Int,
            nixb_sys::ValueType_NIX_TYPE_LIST => ValueKind::List,
            nixb_sys::ValueType_NIX_TYPE_NULL => ValueKind::Null,
            nixb_sys::ValueType_NIX_TYPE_PATH => ValueKind::Path,
            nixb_sys::ValueType_NIX_TYPE_STRING => ValueKind::String,
            nixb_sys::ValueType_NIX_TYPE_THUNK => ValueKind::Thunk,
            other => unreachable!("invalid ValueType: {other}"),
        }
    }

    #[inline]
    fn write(self, dest: UninitValue, _: &mut Context) {
        // 'nix_copy_value' errors when:
        //
        // 1. the destination pointer is null;
        // 2. the destination value is already initialized;
        // 3. the source pointer is null;
        // 4. the source value is not initialized.
        //
        // Having an UninitValue guards against 1) and 2), and having a
        // NixValue guards again that 3) and 4.
        unsafe {
            nixb_sys::copy_value(ptr::null_mut(), dest.as_ptr(), self.as_ptr());
        };
    }
}

impl<Owner: ValueOwner> BoolValue for NixValue<Owner> {
    #[inline]
    unsafe fn into_bool(self, _: &mut Context) -> Result<bool> {
        Ok(unsafe { nixb_sys::get_bool(ptr::null_mut(), self.as_ptr()) })
    }
}

impl<Owner: ValueOwner> IntValue for NixValue<Owner> {
    #[inline]
    unsafe fn into_int(self, _: &mut Context) -> i64 {
        unsafe { nixb_sys::get_int(ptr::null_mut(), self.as_ptr()) }
    }
}

impl<Owner: ValueOwner> StringValue for NixValue<Owner> {
    type String = CString;

    #[inline]
    unsafe fn into_string(self, ctx: &mut Context) -> Result<Self::String> {
        let mut c_string = CString::default();
        unsafe { self.with_string(|c_str| c_string = c_str.to_owned(), ctx)? };
        Ok(c_string)
    }
}

impl<'a> PathValue for NixValue<Borrowed<'a>> {
    type Path = &'a CStr;

    #[inline]
    unsafe fn into_path_string(self, _: &mut Context) -> Result<Self::Path> {
        let c_str_ptr = unsafe {
            nixb_sys::get_path_string(ptr::null_mut(), self.as_ptr())
        };

        // SAFETY: the [docs] guarantee that the returned pointer is
        // valid for as long as the value is alive.
        //
        // [docs]: https://hydra.nixos.org/build/313564006/download/1/html/group__value__extract.html#ga3420055c22accfd07cc5537210d748a9
        Ok(unsafe { CStr::from_ptr(c_str_ptr) })
    }
}

impl<Owner: ValueOwner> TryFromValue<Self> for NixValue<Owner> {
    #[inline]
    fn try_from_value(value: Self, _: &mut Context) -> Result<Self> {
        Ok(value)
    }
}

impl Clone for Owned {
    #[inline]
    fn clone(&self) -> Self {
        unsafe {
            nixb_sys::value_incref(ptr::null_mut(), self.value_ptr().as_ptr());
        }
        Self { ptr: self.ptr }
    }
}

impl Drop for Owned {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            nixb_sys::value_decref(ptr::null_mut(), self.value_ptr().as_ptr());
        }
    }
}

impl ValueOwner for Owned {
    type Borrow<'a> = Borrowed<'a>;

    #[inline]
    fn borrow<'a>(&'a self) -> Self::Borrow<'a> {
        Borrowed { ptr: self.ptr, _lifetime: PhantomData }
    }

    #[inline]
    unsafe fn new(value_ptr: NonNull<nixb_sys::Value>) -> Self {
        Self { ptr: value_ptr }
    }

    #[inline]
    fn value_ptr(&self) -> NonNull<nixb_sys::Value> {
        self.ptr
    }
}

impl From<Borrowed<'_>> for Owned {
    #[inline]
    fn from(borrowed: Borrowed<'_>) -> Self {
        unsafe {
            nixb_sys::value_incref(ptr::null_mut(), borrowed.ptr.as_ptr())
        };
        Self { ptr: borrowed.ptr }
    }
}

impl<'borrow> ValueOwner for Borrowed<'borrow> {
    type Borrow<'a>
        = Self
    where
        'borrow: 'a;

    #[inline]
    fn borrow<'a>(&'a self) -> Self::Borrow<'a> {
        *self
    }

    #[inline]
    unsafe fn new(value_ptr: NonNull<nixb_sys::Value>) -> Self {
        Self { ptr: value_ptr, _lifetime: PhantomData }
    }

    #[inline]
    fn value_ptr(&self) -> NonNull<nixb_sys::Value> {
        self.ptr
    }
}

impl<'a, 'borrow: 'a> IsBorrowedOfAtLeast<'a> for Borrowed<'borrow> {
    #[inline]
    fn into_borrowed(self) -> Borrowed<'a> {
        self
    }
}

impl Value for bool {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Bool
    }

    #[inline]
    fn write(self, dest: UninitValue, _: &mut Context) {
        // `nix_init_bool` errors when:
        //
        // 1. the destination pointer is null;
        // 2. the destination value is already initialized.
        //
        // Having an `UninitValue` guards against both, so neither can happen.
        unsafe {
            nixb_sys::init_bool(ptr::null_mut(), dest.as_ptr(), self);
        }
    }
}

macro_rules! impl_value_for_int {
    ($ty:ty) => {
        impl Value for $ty {
            #[inline]
            fn kind(&self) -> ValueKind {
                ValueKind::Int
            }

            #[inline]
            fn write(self, dest: UninitValue, _: &mut Context) {
                // `nix_init_int` errors when:
                //
                // 1. the destination pointer is null;
                // 2. the destination value is already initialized.
                //
                // Having an `UninitValue` guards against both, so neither can
                // happen.
                unsafe {
                    nixb_sys::init_int(
                        ptr::null_mut(),
                        dest.as_ptr(),
                        self.into(),
                    );
                }
            }
        }

        impl IntValue for $ty {
            #[inline]
            unsafe fn into_int(self, _: &mut Context) -> i64 {
                self.into()
            }
        }
    };
}

impl_value_for_int!(u8);
impl_value_for_int!(u16);
impl_value_for_int!(u32);
impl_value_for_int!(i8);
impl_value_for_int!(i16);
impl_value_for_int!(i32);
impl_value_for_int!(i64);

macro_rules! impl_value_for_big_int {
    ($ty:ty) => {
        impl Value for $ty {
            #[inline]
            fn kind(&self) -> ValueKind {
                ValueKind::Int
            }

            #[inline]
            fn write(self, dest: UninitValue, ctx: &mut Context) {
                match i64::try_from(self) {
                    value => value.write(dest, ctx),
                    _ => panic!(
                        "{self}{} cannot be represented as a Nix integer",
                        stringify!($ty),
                    ),
                }
            }
        }
    };
}

impl_value_for_big_int!(usize);
impl_value_for_big_int!(isize);
impl_value_for_big_int!(u64);

macro_rules! impl_value_for_float {
    ($ty:ty) => {
        impl Value for $ty {
            #[inline]
            fn kind(&self) -> ValueKind {
                ValueKind::Float
            }

            #[inline]
            fn write(self, dest: UninitValue, _: &mut Context) {
                // `nix_init_float` errors when:
                //
                // 1. the destination pointer is null;
                // 2. the destination value is already initialized.
                //
                // Having an `UninitValue` guards against both, so neither can
                // happen.
                unsafe {
                    nixb_sys::init_float(
                        ptr::null_mut(),
                        dest.as_ptr(),
                        self.into(),
                    );
                }
            }
        }
    };
}

impl_value_for_float!(f32);
impl_value_for_float!(f64);

impl Value for &CStr {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::String
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        // `nix_init_string` errors when:
        //
        // 1. the destination pointer is null;
        // 2. the destination value is already initialized;
        // 3. allocating storage for the string fails.
        //
        // Having an `UninitValue` guards against 1) and 2), while 3) is an
        // allocation failure which we panic on.
        ctx.with_raw(|ctx| unsafe {
            nixb_sys::init_string(ctx, dest.as_ptr(), self.as_ptr());
        })
        .unwrap_or_else(|err| panic!("failed to allocate Nix string: {err}"));
    }
}

impl Value for CString {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::String
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        self.as_c_str().write(dest, ctx)
    }
}

impl Value for &str {
    #[inline(always)]
    fn kind(&self) -> ValueKind {
        ValueKind::String
    }

    #[inline(always)]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        match CString::new(self) {
            Ok(c_str) => c_str.write(dest, ctx),
            Err(nul_err) => {
                let until_nul_byte = &self[..nul_err.nul_position() + 1];
                // SAFETY: the string we just sliced has a trailign NUL byte.
                let c_str = unsafe {
                    CStr::from_bytes_with_nul_unchecked(
                        until_nul_byte.as_bytes(),
                    )
                };
                c_str.write(dest, ctx)
            },
        }
    }
}

impl Value for alloc::string::String {
    #[inline(always)]
    fn kind(&self) -> ValueKind {
        ValueKind::String
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        self.as_str().write(dest, ctx)
    }
}

impl<T: Value> Value for Option<T> {
    #[inline]
    fn kind(&self) -> ValueKind {
        match self {
            Some(value) => value.kind(),
            None => ValueKind::Null,
        }
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        match self {
            Some(value) => value.write(dest, ctx),
            None => Null.write(dest, ctx),
        }
    }
}

impl<'a, T: ?Sized + ToOwned> Value for Cow<'a, T>
where
    &'a T: Value,
    T::Owned: Value,
{
    #[inline]
    fn kind(&self) -> ValueKind {
        match self {
            Self::Borrowed(value) => (*value).kind(),
            Self::Owned(value) => value.kind(),
        }
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        match self {
            Self::Borrowed(value) => value.write(dest, ctx),
            Self::Owned(value) => value.write(dest, ctx),
        }
    }
}

impl<T: IntoValue> Value for Vec<T> {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::List
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        List::write(self, dest, ctx)
    }
}

impl<const N: usize, T: IntoValue> Value for [T; N] {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::List
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        List::write(self, dest, ctx)
    }
}

#[cfg(feature = "std")]
impl Value for &std::path::Path {
    #[inline(always)]
    fn kind(&self) -> ValueKind {
        ValueKind::Path
    }

    #[inline(always)]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        let bytes = self.as_os_str().as_encoded_bytes();

        let c_string =
            CString::new(bytes).map(Cow::Owned).unwrap_or_else(|nul_err| {
                let until_nul_byte = &bytes[..nul_err.nul_position() + 1];
                // SAFETY: the array we just sliced has a trailign NUL byte.
                Cow::Borrowed(unsafe {
                    CStr::from_bytes_with_nul_unchecked(until_nul_byte)
                })
            });

        // `nix_init_path_string` errors when:
        //
        // 1. the destination pointer is null;
        // 2. the destination value is already initialized;
        // 3. canonicalizing or storing the path fails to allocate;
        // 4. the path exceeds an internal container's maximum capacity.
        //
        // Having an `UninitValue` guards against 1) and 2), while 3) and 4) are
        // allocation/capacity failures which we panic on.
        ctx.with_raw_and_state(|ctx, state| unsafe {
            #[cfg(not(feature = "nix-2-34"))]
            nixb_cpp::init_path_string(
                ctx,
                state.as_ptr(),
                dest.as_ptr(),
                c_string.as_ptr(),
            );

            #[cfg(feature = "nix-2-34")]
            nixb_sys::init_path_string(
                ctx,
                state.as_ptr(),
                dest.as_ptr(),
                c_string.as_ptr(),
            );
        })
        .unwrap_or_else(|err| {
            panic!("failed to allocate or canonicalize Nix path: {err}")
        });
    }
}

#[cfg(feature = "std")]
impl PathValue for &std::path::Path {
    type Path = CString;

    #[inline]
    unsafe fn into_path_string(self, _: &mut Context) -> Result<Self::Path> {
        CString::new(self.as_os_str().as_encoded_bytes()).map_err(Into::into)
    }
}

#[cfg(feature = "std")]
impl Value for std::path::PathBuf {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::Path
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        self.as_path().write(dest, ctx)
    }
}

#[cfg(feature = "std")]
impl PathValue for std::path::PathBuf {
    type Path = CString;

    #[inline]
    unsafe fn into_path_string(self, ctx: &mut Context) -> Result<Self::Path> {
        unsafe { self.as_path().into_path_string(ctx) }
    }
}

#[cfg(feature = "compact_str")]
impl Value for compact_str::CompactString {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::String
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        self.as_str().write(dest, ctx)
    }
}

#[cfg(feature = "smallvec")]
impl<T: IntoValue, const N: usize> Value for smallvec::SmallVec<[T; N]> {
    #[inline]
    fn kind(&self) -> ValueKind {
        ValueKind::List
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        List::write(self, dest, ctx)
    }
}

#[cfg(feature = "either")]
impl<L: Value, R: Value> Value for either::Either<L, R> {
    #[inline]
    fn kind(&self) -> ValueKind {
        match self {
            Self::Left(left) => left.kind(),
            Self::Right(right) => right.kind(),
        }
    }

    #[inline]
    fn write(self, dest: UninitValue, ctx: &mut Context) {
        match self {
            Self::Left(left) => left.write(dest, ctx),
            Self::Right(right) => right.write(dest, ctx),
        }
    }
}

impl<T: Value> IntoValue for T {
    #[inline(always)]
    fn into_value(self, _: &mut Context) -> Self {
        self
    }
}

impl<F, T> IntoValue for IntoValueFn<F, T>
where
    F: FnOnce(&mut Context) -> T,
    T: IntoValue,
{
    #[inline]
    fn into_value<'eval>(
        self,
        ctx: &mut Context<'eval>,
    ) -> impl Value + use<'eval, F, T> {
        (self.f)(ctx).into_value(ctx)
    }
}

impl<V: BoolValue> TryFromValue<V> for bool {
    #[inline]
    fn try_from_value(mut value: V, ctx: &mut Context) -> Result<Self> {
        value.force_inline(ctx)?;

        match value.kind() {
            // SAFETY: the value's kind is a boolean.
            ValueKind::Bool => unsafe { value.into_bool(ctx) },
            other => Err(TypeMismatchError {
                expected: ValueKind::Bool,
                found: other,
            }
            .into()),
        }
    }
}

impl<V: IntValue> TryFromValue<V> for i64 {
    #[inline]
    fn try_from_value(mut value: V, ctx: &mut Context) -> Result<Self> {
        value.force_inline(ctx)?;

        match value.kind() {
            // SAFETY: the value's kind is an integer.
            ValueKind::Int => Ok(unsafe { value.into_int(ctx) }),
            other => Err(TypeMismatchError {
                expected: ValueKind::Int,
                found: other,
            }
            .into()),
        }
    }
}

macro_rules! impl_try_from_value_for_int {
    ($ty:ty) => {
        impl<V: IntValue> TryFromValue<V> for $ty {
            #[inline]
            fn try_from_value(value: V, ctx: &mut Context) -> Result<Self> {
                let int = i64::try_from_value(value, ctx)?;

                int.try_into()
                    .map_err(|_| TryFromI64Error::<$ty>::new(int).into())
            }
        }
    };
}

impl_try_from_value_for_int!(i8);
impl_try_from_value_for_int!(i16);
impl_try_from_value_for_int!(i32);
impl_try_from_value_for_int!(i128);
impl_try_from_value_for_int!(isize);

impl_try_from_value_for_int!(u8);
impl_try_from_value_for_int!(u16);
impl_try_from_value_for_int!(u32);
impl_try_from_value_for_int!(u64);
impl_try_from_value_for_int!(u128);
impl_try_from_value_for_int!(usize);

macro_rules! impl_try_from_string_value {
    ($ty:ty) => {
        impl<V: StringValue> TryFromValue<V> for $ty
        where
            V::String: TryInto<Self, Error: Into<Error>>,
        {
            #[inline]
            fn try_from_value(mut value: V, ctx: &mut Context) -> Result<Self> {
                value.force_inline(ctx)?;

                match value.kind() {
                    ValueKind::String => {
                        // SAFETY: the value's kind is a string.
                        let string = unsafe { value.into_string(ctx)? };
                        string.try_into().map_err(Into::into)
                    },
                    other => Err(TypeMismatchError {
                        expected: ValueKind::String,
                        found: other,
                    }
                    .into()),
                }
            }
        }
    };
}

impl_try_from_string_value!(&CStr);
impl_try_from_string_value!(&str);
impl_try_from_string_value!(CString);
impl_try_from_string_value!(alloc::string::String);

impl<V: Value, T> TryFromValue<V> for Option<T>
where
    V: Value,
    T: TryFromValue<V>,
{
    #[inline]
    fn try_from_value(mut value: V, ctx: &mut Context) -> Result<Self> {
        value.force_inline(ctx)?;
        match value.kind() {
            ValueKind::Null => Ok(None),
            _ => T::try_from_value(value, ctx).map(Some),
        }
    }
}

impl<Owner: ValueOwner, T> TryFromValue<NixValue<Owner>> for Vec<T>
where
    T: TryFromValue<NixValue<Owner>>,
    Self: TryFromValue<NixList<Owner>>,
{
    #[inline]
    fn try_from_value(
        value: NixValue<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        NixList::try_from_value(value, ctx)
            .and_then(|list| Self::try_from_value(list, ctx))
    }
}

#[cfg(all(unix, feature = "std"))]
impl<'a, V: PathValue<Path = &'a CStr>> TryFromValue<V>
    for &'a std::path::Path
{
    #[inline]
    fn try_from_value(mut value: V, ctx: &mut Context) -> Result<Self> {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::Path;

        value.force_inline(ctx)?;

        match value.kind() {
            ValueKind::Path => {
                // SAFETY: the value's kind is a path.
                let c_str = unsafe { value.into_path_string(ctx)? };
                let os_str = OsStr::from_bytes(c_str.to_bytes());
                Ok(Path::new(os_str))
            },
            other => Err(TypeMismatchError {
                expected: ValueKind::Path,
                found: other,
            }
            .into()),
        }
    }
}

#[cfg(all(unix, feature = "std"))]
impl<'a, V: PathValue> TryFromValue<V> for Cow<'a, std::path::Path>
where
    V::Path: Into<Cow<'a, CStr>>,
{
    #[inline]
    fn try_from_value(mut value: V, ctx: &mut Context) -> Result<Self> {
        use alloc::borrow::Cow;
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;
        use std::path::Path;

        value.force_inline(ctx)?;

        match value.kind() {
            ValueKind::Path => {
                // SAFETY: the value's kind is a path.
                match unsafe { value.into_path_string(ctx)? }.into() {
                    Cow::Borrowed(c_str) => {
                        let os_str = OsStr::from_bytes(c_str.to_bytes());
                        Ok(Cow::Borrowed(Path::new(os_str)))
                    },
                    Cow::Owned(c_string) => {
                        let os_str = OsStr::from_bytes(c_string.to_bytes());
                        Ok(Cow::Owned(Path::new(os_str).to_owned()))
                    },
                }
            },
            other => Err(TypeMismatchError {
                expected: ValueKind::Path,
                found: other,
            }
            .into()),
        }
    }
}

#[cfg(feature = "std")]
impl<'a, V: PathValue> TryFromValue<V> for std::path::PathBuf
where
    V::Path: Into<Cow<'a, CStr>>,
{
    #[inline]
    fn try_from_value(value: V, ctx: &mut Context) -> Result<Self> {
        <Cow<'_, std::path::Path>>::try_from_value(value, ctx)
            .map(Cow::into_owned)
    }
}

#[cfg(feature = "compact_str")]
impl<Owner: ValueOwner> TryFromValue<NixValue<Owner>>
    for compact_str::CompactString
{
    #[inline]
    fn try_from_value(
        mut value: NixValue<Owner>,
        ctx: &mut Context,
    ) -> Result<Self> {
        value.force_inline(ctx)?;

        match value.kind() {
            ValueKind::String => {
                let mut res = Ok(Self::const_new(""));
                // SAFETY: the value's kind is a string.
                unsafe {
                    value.with_string(
                        |c_str| res = c_str.to_str().map(Into::into),
                        ctx,
                    )?
                };
                res.map_err(Into::into)
            },
            other => Err(TypeMismatchError {
                expected: ValueKind::String,
                found: other,
            }
            .into()),
        }
    }
}

impl Values for () {
    type FromFirst = Self;
    type UpToLast = Self;
}

impl<T> Values for [T; 0] {
    type FromFirst = Self;
    type UpToLast = Self;
}

impl<T> Values for T
where
    T: RecursiveTuple<First: IntoValue, Last: IntoValue>,
    <T as Tuple>::FromFirst: Values,
    <T as Tuple>::UpToLast: Values,
{
    type FromFirst = <T as Tuple>::FromFirst;
    type UpToLast = <T as Tuple>::UpToLast;
}
