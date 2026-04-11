//! This module loads the Nix C API shared libraries into the current process
//! before the plugin makes its first `nix_*` call.
//!
//! For some reason, the `nix` executable does not export the symbols from the
//! `libnix*c` shared libraries that implement the C API, so we cannot rely on
//! those symbols being resolved by the host process.
//!
//! To solve this, we:
//!
//! 1. inspect the already loaded shared libraries to determine the runtime
//!    Nix version;
//! 2. verify that the runtime Nix version matches the one the plugin was
//!    compiled against;
//! 3. query the Nix database for the exact `nix-*-c` outputs for that runtime
//!    version;
//! 4. `dlopen`those shared libraries with global symbol visibility so that
//!    calls to `nix_*` functions can resolve correctly;
//!
//! The module exports a single function, [`open`], which must be the first
//! function called after the plugin is loaded.

use core::fmt::Write;
use core::ops::ControlFlow;
use core::{fmt, mem};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const DYLIBS: [(&str, &str); 4] = [
    ("nix-util-c", "nixutilc"),
    ("nix-store-c", "nixstorec"),
    ("nix-expr-c", "nixexprc"),
    ("nix-flake-c", "nixflakec"),
];

const NIX_DB_PATH: &str = "/nix/var/nix/db/db.sqlite";

const TARGET_NIX_VERSION: &str = {
    #[cfg(not(feature = "nix-2-33"))]
    {
        "2.32"
    }
    #[cfg(all(feature = "nix-2-33", not(feature = "nix-2-34")))]
    {
        "2.33"
    }
    #[cfg(feature = "nix-2-34")]
    {
        "2.34"
    }
};

#[inline]
#[track_caller]
pub(crate) fn open() {
    static DLOPEN_RESULT: OnceLock<Result<(), DyLibOpenError>> =
        OnceLock::new();

    if let Err(err) = DLOPEN_RESULT.get_or_init(open_impl) {
        panic!("Couldn't load the Nix C API dylibs: {err}");
    }
}

fn open_impl() -> Result<(), DyLibOpenError> {
    let nix_version = loaded_nix_version()?;

    if !nix_version.starts_with(TARGET_NIX_VERSION) {
        return Err(DyLibOpenError::NixVersionMismatch {
            loaded_version: nix_version,
        });
    }

    let db_path = Path::new(NIX_DB_PATH);

    if !db_path.exists() {
        return Err(DyLibOpenError::NixDbNotFound);
    }

    let connection = rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    )
    .map_err(DyLibOpenError::OpenNixDb)?;

    let mut stmt = connection
        .prepare("SELECT path FROM ValidPaths WHERE path LIKE '%' || ?1")
        .map_err(DyLibOpenError::QueryNixDb)?;

    let mut lib_filename = String::new();
    let mut store_suffix = String::new();

    for (pkg, lib_name) in DYLIBS {
        write!(&mut store_suffix, "-{pkg}-{nix_version}").expect("cannot fail");

        let Some(first_row) = stmt
            .query_map(rusqlite::params![store_suffix], |row| {
                row.get::<_, String>(0)
            })
            .map_err(DyLibOpenError::QueryNixDb)?
            .next()
        else {
            return Err(DyLibOpenError::StoreOutputNotFound {
                pkg: pkg.to_owned(),
                version: nix_version,
            });
        };

        write!(
            &mut lib_filename,
            "{}{lib_name}{}",
            env::consts::DLL_PREFIX,
            env::consts::DLL_SUFFIX,
        )
        .expect("cannot fail");

        let store_path = first_row.map_err(DyLibOpenError::QueryNixDb)?;

        let lib_path = Path::new(&store_path).join("lib").join(&lib_filename);

        if !lib_path.exists() {
            return Err(DyLibOpenError::DyLibNotFound { lib_path });
        }

        let handle = unsafe {
            libloading::os::unix::Library::open(
                Some(&lib_path),
                libc::RTLD_NOW | libc::RTLD_GLOBAL,
            )
        }
        .map_err(|err| DyLibOpenError::LoadDyLib { lib_path, err })?;

        // Dropping the handle would call `dlclose`, but we need the library to
        // stay open for the duration of the program.
        mem::forget(handle);

        lib_filename.clear();
        store_suffix.clear();
    }

    Ok(())
}

fn loaded_nix_version() -> Result<String, DyLibOpenError> {
    let markers = [
        "-nix-expr-",
        "-nix-util-",
        "-nix-store-",
        "-nix-flake-",
        "-nix-main-",
    ];

    platform::loaded_image_paths(|lib_path| {
        // We look for library paths of the form:
        //   /nix/store/<hash><marker><version>/lib/..
        for marker in markers {
            let Some(marker_start) =
                memchr::memmem::find(lib_path.to_bytes(), marker.as_bytes())
            else {
                continue;
            };

            let version_start = marker_start + marker.len();

            let Some(version_len) =
                memchr::memchr(b'/', &lib_path.to_bytes()[version_start..])
            else {
                continue;
            };

            let Ok(version_str) = str::from_utf8(
                &lib_path.to_bytes()
                    [version_start..version_start + version_len],
            ) else {
                continue;
            };

            return ControlFlow::Break(version_str.to_owned());
        }

        ControlFlow::Continue(())
    })
    .ok_or(DyLibOpenError::UnknownNixVersion)
}

#[derive(Debug)]
enum DyLibOpenError {
    DyLibNotFound { lib_path: PathBuf },
    LoadDyLib { lib_path: PathBuf, err: libloading::Error },
    NixDbNotFound,
    NixVersionMismatch { loaded_version: String },
    OpenNixDb(rusqlite::Error),
    QueryNixDb(rusqlite::Error),
    StoreOutputNotFound { pkg: String, version: String },
    UnknownNixVersion,
}

impl fmt::Display for DyLibOpenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DyLibNotFound { lib_path } => {
                write!(
                    f,
                    "couldn't find a shared library at {}",
                    lib_path.display()
                )
            },
            Self::LoadDyLib { lib_path: dylib_path, err } => {
                write!(
                    f,
                    "couldn't dlopen library at {}: {err}",
                    dylib_path.display()
                )
            },
            Self::NixDbNotFound => {
                write!(f, "Nix DB not found at {NIX_DB_PATH}",)
            },
            Self::NixVersionMismatch { loaded_version } => {
                write!(
                    f,
                    "plugin compiled for {TARGET_NIX_VERSION} but loaded by \
                     {loaded_version}"
                )
            },
            Self::OpenNixDb(err) => {
                write!(f, "couldn't open Nix DB at {NIX_DB_PATH}: {err}")
            },
            Self::QueryNixDb(err) => {
                write!(f, "couldn't query Nix DB at {NIX_DB_PATH}: {err}")
            },
            Self::StoreOutputNotFound { pkg, version } => {
                write!(
                    f,
                    "couldn't find /nix/store output for {pkg}-{version} in \
                     the Nix DB"
                )
            },
            Self::UnknownNixVersion => {
                write!(
                    f,
                    "couldn't determine Nix version from loaded shared \
                     libraries"
                )
            },
        }
    }
}

impl core::error::Error for DyLibOpenError {}

#[cfg(target_os = "macos")]
mod platform {
    use core::ffi::{CStr, c_char};
    use core::ops::ControlFlow;

    /// Calls the given function with the path of each loaded shared library in
    /// the current process.
    pub(super) fn loaded_image_paths<T>(
        mut fun: impl FnMut(&CStr) -> ControlFlow<T>,
    ) -> Option<T> {
        (0..unsafe { _dyld_image_count() })
            .map(|idx| unsafe { _dyld_get_image_name(idx) })
            .filter(|ptr| !ptr.is_null())
            .map(|ptr| unsafe { CStr::from_ptr(ptr) })
            .find_map(|cstr| fun(cstr).break_value())
    }

    unsafe extern "C" {
        fn _dyld_image_count() -> u32;
        fn _dyld_get_image_name(index: u32) -> *const c_char;
    }
}

#[cfg(target_os = "linux")]
mod platform {
    use core::ffi::{CStr, c_void};
    use core::ops::ControlFlow;

    use libc::{c_char, c_int, dl_iterate_phdr, dl_phdr_info, size_t};

    /// Calls the given function with the path of each loaded shared library in
    /// the current process.
    pub(super) fn loaded_image_paths<T>(
        fun: impl FnMut(&CStr) -> ControlFlow<T>,
    ) -> Option<T> {
        struct State<F, T> {
            fun: F,
            break_value: Option<T>,
        }

        unsafe extern "C" fn callback<F, T>(
            info: *mut dl_phdr_info,
            _size: size_t,
            data: *mut c_void,
        ) -> c_int
        where
            F: FnMut(&CStr) -> ControlFlow<T>,
        {
            let state = unsafe { &mut *data.cast::<State<F, T>>() };
            let path = unsafe { (*info).dlpi_name.cast::<c_char>() };
            if path.is_null() || unsafe { *path } == 0 {
                return 0;
            }
            let path = unsafe { CStr::from_ptr(path) };
            match (state.fun)(path) {
                ControlFlow::Continue(()) => 0,
                ControlFlow::Break(value) => {
                    state.break_value = Some(value);
                    1
                },
            }
        }

        let mut state = State { fun, break_value: None };

        unsafe {
            dl_iterate_phdr(
                Some(callback::<_, T>),
                (&mut state as *mut State<_, _>).cast(),
            );
        }

        state.break_value
    }
}
