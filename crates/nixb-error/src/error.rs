use alloc::borrow::Cow;
use alloc::ffi::CString;
use core::ffi::CStr;
use core::fmt;

/// TODO: docs.
#[derive(Clone, Debug)]
pub struct Error {
    kind: ErrorKind,
    message: Cow<'static, CStr>,
}

/// TODO: docs.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    /// An unknown error occurred.
    ///
    /// This error code is returned when an unknown error occurred during the
    /// function execution.
    Unknown,

    /// An overflow error occurred.
    ///
    /// This error code is returned when an overflow error occurred during the
    /// function execution.
    Overflow,

    /// A key/index access error occurred in C API functions.
    ///
    /// This error code is returned when accessing a key, index, or identifier
    /// that does not exist in C API functions. Common scenarios include:
    ///
    /// - setting keys that don't exist;
    /// - list indices that are out of bounds;
    /// - attribute names that don't exist;
    /// - attribute indices that are out of bounds;
    ///
    /// This error typically indicates incorrect usage or assumptions about
    /// data structure contents, rather than internal Nix evaluation errors.
    Key,

    /// A generic Nix error occurred.
    ///
    /// This error code is returned when a generic Nix error occurred during
    /// the function execution.
    Nix,
}

impl Error {
    /// TODO: docs.
    #[track_caller]
    #[inline]
    #[cfg(feature = "std")]
    pub fn from_message(message: impl fmt::Display) -> Self {
        let message_str = message.to_string();

        let message = match CString::new(message_str) {
            Ok(cstring) => Cow::Owned(cstring),

            Err(nul_err) => {
                let nul_byte_idx = nul_err.nul_position();
                // SAFETY: the bytes where created from a string, so it's safe
                // to turn them back into one.
                let message_str =
                    unsafe { String::from_utf8_unchecked(nul_err.into_vec()) };
                panic!(
                    "error message {message_str:?} contains NUL byte at index \
                     {nul_byte_idx}, so it can't be converted into a Nix Error"
                )
            },
        };

        Self { kind: ErrorKind::Nix, message }
    }

    /// Returns the error's kind.
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// TODO: docs.
    #[inline]
    pub fn map_message<NewMessage: Into<Cow<'static, CStr>>>(
        self,
        f: impl FnOnce(Cow<'static, CStr>) -> NewMessage,
    ) -> Self {
        Self::new(self.kind, f(self.message))
    }

    /// Returns the error's message.
    #[inline]
    pub fn message(&self) -> &CStr {
        &self.message
    }

    /// TODO: docs.
    #[inline]
    pub fn new(
        kind: ErrorKind,
        message: impl Into<Cow<'static, CStr>>,
    ) -> Self {
        Self { kind, message: message.into() }
    }
}

impl ErrorKind {
    /// TODO: docs.
    #[inline]
    pub fn code(self) -> nixb_sys::err {
        match self {
            Self::Unknown => nixb_sys::err_NIX_ERR_UNKNOWN,
            Self::Overflow => nixb_sys::err_NIX_ERR_OVERFLOW,
            Self::Key => nixb_sys::err_NIX_ERR_KEY,
            Self::Nix => nixb_sys::err_NIX_ERR_NIX_ERROR,
        }
    }

    /// TODO: docs.
    #[inline]
    pub fn from_code(err_code: nixb_sys::err) -> Option<Self> {
        match err_code {
            nixb_sys::err_NIX_OK => None,
            nixb_sys::err_NIX_ERR_UNKNOWN => Some(Self::Unknown),
            nixb_sys::err_NIX_ERR_OVERFLOW => Some(Self::Overflow),
            nixb_sys::err_NIX_ERR_KEY => Some(Self::Key),
            nixb_sys::err_NIX_ERR_NIX_ERROR => Some(Self::Nix),
            other => unreachable!("unknown error code: {other}"),
        }
    }
}

impl fmt::Display for Error {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Nix => self.message.to_string_lossy().fmt(f),
            other_kind => other_kind.fmt(f),
        }
    }
}

impl core::error::Error for Error {}

impl fmt::Display for ErrorKind {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match *self {
            Self::Unknown => "an unknown error occurred",
            Self::Overflow => "an overflow error occurred",
            Self::Key => "a key/index access error occurred",
            Self::Nix => "a generic Nix error occurred",
        })
    }
}

impl From<core::convert::Infallible> for Error {
    #[inline]
    fn from(err: core::convert::Infallible) -> Self {
        match err {}
    }
}

impl From<alloc::ffi::NulError> for Error {
    #[inline]
    fn from(err: alloc::ffi::NulError) -> Self {
        Self::from_message(err)
    }
}

impl From<alloc::ffi::IntoStringError> for Error {
    #[inline]
    fn from(_: alloc::ffi::IntoStringError) -> Self {
        Self::new(ErrorKind::Nix, c"C string contained non-utf8 bytes")
    }
}

impl From<core::str::Utf8Error> for Error {
    #[inline]
    fn from(err: core::str::Utf8Error) -> Self {
        Self::from_message(err)
    }
}
