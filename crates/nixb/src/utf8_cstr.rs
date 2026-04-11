use core::ffi::CStr;
use core::fmt;
use core::str::Utf8Error;

/// A wrapper around a [`CStr`] that is guaranteed to contain valid UTF-8.
#[repr(transparent)]
pub struct Utf8CStr(CStr);

impl Utf8CStr {
    /// Returns the underlying [`CStr`].
    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        &self.0
    }

    /// Returns the UTF-8 string slice contained in this [`Utf8CStr`].
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: the inner CStr is guaranteed to contain valid UTF-8.
        unsafe { str::from_utf8_unchecked(self.as_c_str().to_bytes()) }
    }

    /// Creates a new [`Utf8CStr`] from the given [`CStr`], without checking
    /// whether it contains valid UTF-8.
    #[inline]
    pub fn new(cstr: &CStr) -> Result<&Self, Utf8Error> {
        cstr.to_str().map(|_| {
            // SAFETY: `cstr` contains valid UTF-8.
            unsafe { Self::new_unchecked(cstr) }
        })
    }

    /// Creates a new [`Utf8CStr`] from the given [`CStr`], without checking
    /// whether it contains valid UTF-8.
    ///
    /// # Safety
    ///
    /// The caller must ensure that the given `CStr` contains valid UTF-8.
    #[inline]
    pub const unsafe fn new_unchecked(cstr: &CStr) -> &Self {
        debug_assert!(
            cstr.to_str().is_ok(),
            "Utf8CStr::new_unchecked called with invalid UTF-8 CStr"
        );
        // SAFETY: the caller guarantees that `cstr` contains valid UTF-8, and
        // `Self` is #[repr(transparent)] over `CStr`.
        unsafe { &*(cstr as *const CStr as *const Self) }
    }
}

impl fmt::Debug for Utf8CStr {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(f)
    }
}

impl AsRef<Self> for Utf8CStr {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl AsRef<CStr> for Utf8CStr {
    #[inline]
    fn as_ref(&self) -> &CStr {
        self.as_c_str()
    }
}
