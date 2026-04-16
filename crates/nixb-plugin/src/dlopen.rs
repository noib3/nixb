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
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rusqlite::OptionalExtension;

const DYLIBS: [Dylib; 4] = [
    Dylib { package: "nix-util-c", lib_name: "nixutilc", refs: &["nix-util"] },
    Dylib {
        package: "nix-store-c",
        lib_name: "nixstorec",
        refs: &["nix-util", "nix-store"],
    },
    Dylib {
        package: "nix-expr-c",
        lib_name: "nixexprc",
        refs: &["nix-util", "nix-store", "nix-expr"],
    },
    Dylib {
        package: "nix-flake-c",
        lib_name: "nixflakec",
        refs: &["nix-expr", "nix-flake"],
    },
];

const LOADED_NIX_DYLIBS: [(&str, &str); 5] = [
    ("nix-main", "-nix-main-"),
    ("nix-util", "-nix-util-"),
    ("nix-store", "-nix-store-"),
    ("nix-expr", "-nix-expr-"),
    ("nix-flake", "-nix-flake-"),
];

const RUNTIME_REF_PACKAGES: [&str; 4] =
    ["nix-util", "nix-store", "nix-expr", "nix-flake"];

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
    #[cfg(all(feature = "nix-2-34", not(feature = "nix-2-35")))]
    {
        "2.34"
    }
    #[cfg(feature = "nix-2-35")]
    {
        "2.35"
    }
};

struct Dylib {
    package: &'static str,
    lib_name: &'static str,
    refs: &'static [&'static str],
}

struct LoadedNixDylib {
    package: &'static str,
    store_path: String,
    version: String,
}

struct LoadedNixRuntime {
    version: String,
    store_paths: HashMap<&'static str, String>,
}

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
    let loaded_nix = loaded_nix_runtime()?;

    if !loaded_nix.version.starts_with(TARGET_NIX_VERSION) {
        return Err(DyLibOpenError::NixVersionMismatch {
            loaded_version: loaded_nix.version,
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

    let mut ref_stmt = connection
        .prepare(
            "SELECT 1 FROM Refs refs JOIN ValidPaths referrer ON referrer.id \
             = refs.referrer JOIN ValidPaths reference ON reference.id = \
             refs.reference WHERE referrer.path = ?1 AND reference.path = ?2 \
             LIMIT 1",
        )
        .map_err(DyLibOpenError::QueryNixDb)?;

    let mut lib_filename = String::new();
    let mut store_suffix = String::new();

    for dylib in DYLIBS {
        write!(&mut store_suffix, "-{}-{}", dylib.package, loaded_nix.version)
            .expect("cannot fail");

        let expected_refs = dylib
            .refs
            .iter()
            .map(|pkg| {
                loaded_nix.store_paths.get(pkg).cloned().ok_or_else(|| {
                    DyLibOpenError::LoadedDyLibNotFound {
                        pkg: (*pkg).to_owned(),
                        version: loaded_nix.version.clone(),
                    }
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let candidates = stmt
            .query_map(rusqlite::params![store_suffix], |row| {
                row.get::<_, String>(0)
            })
            .map_err(DyLibOpenError::QueryNixDb)?;

        let mut store_path = None;

        'candidate: for candidate in candidates {
            let candidate = candidate.map_err(DyLibOpenError::QueryNixDb)?;

            for expected_ref in &expected_refs {
                let has_ref = ref_stmt
                    .query_row(
                        rusqlite::params![candidate, expected_ref],
                        |_| Ok(()),
                    )
                    .optional()
                    .map_err(DyLibOpenError::QueryNixDb)?
                    .is_some();

                if !has_ref {
                    continue 'candidate;
                }
            }

            store_path = Some(candidate);
            break;
        }

        let Some(store_path) = store_path else {
            return Err(DyLibOpenError::StoreOutputVariantNotFound {
                pkg: dylib.package.to_owned(),
                version: loaded_nix.version.clone(),
                refs: expected_refs,
            });
        };

        write!(
            &mut lib_filename,
            "{}{}{}",
            env::consts::DLL_PREFIX,
            dylib.lib_name,
            env::consts::DLL_SUFFIX,
        )
        .expect("cannot fail");

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

fn loaded_nix_runtime() -> Result<LoadedNixRuntime, DyLibOpenError> {
    let mut loaded_dylibs = Vec::new();

    platform::loaded_image_paths(|lib_path| {
        if let Some(parsed) = parse_loaded_nix_dylib(lib_path) {
            loaded_dylibs.push(parsed);
        }

        ControlFlow::<()>::Continue(())
    });

    let version = loaded_dylibs
        .iter()
        .rev()
        .find_map(|dylib| {
            (dylib.package == "nix-main"
                && dylib.version.starts_with(TARGET_NIX_VERSION))
            .then(|| dylib.version.clone())
        })
        .or_else(|| {
            loaded_dylibs.iter().rev().find_map(|dylib| {
                dylib
                    .version
                    .starts_with(TARGET_NIX_VERSION)
                    .then(|| dylib.version.clone())
            })
        })
        .ok_or(DyLibOpenError::UnknownNixVersion)?;

    let store_paths = loaded_dylibs
        .into_iter()
        .filter(|dylib| {
            dylib.version == version
                && RUNTIME_REF_PACKAGES.contains(&dylib.package)
        })
        .map(|dylib| (dylib.package, dylib.store_path))
        .collect::<HashMap<_, _>>();

    for pkg in RUNTIME_REF_PACKAGES {
        if !store_paths.contains_key(pkg) {
            return Err(DyLibOpenError::LoadedDyLibNotFound {
                pkg: pkg.to_owned(),
                version: version.clone(),
            });
        }
    }

    Ok(LoadedNixRuntime { version, store_paths })
}

fn parse_loaded_nix_dylib(lib_path: &std::ffi::CStr) -> Option<LoadedNixDylib> {
    let lib_path = lib_path.to_bytes();

    // We look for library paths of the form:
    //   /nix/store/<hash><marker><version>/lib/..
    for (package, marker) in LOADED_NIX_DYLIBS {
        let Some(marker_start) =
            memchr::memmem::find(lib_path, marker.as_bytes())
        else {
            continue;
        };

        let version_start = marker_start + marker.len();

        let Some(version_len) =
            memchr::memchr(b'/', &lib_path[version_start..])
        else {
            continue;
        };

        let Ok(version) = str::from_utf8(
            &lib_path[version_start..version_start + version_len],
        ) else {
            continue;
        };

        let Ok(store_path) =
            str::from_utf8(&lib_path[..version_start + version_len])
        else {
            continue;
        };

        return Some(LoadedNixDylib {
            package,
            store_path: store_path.to_owned(),
            version: version.to_owned(),
        });
    }

    None
}

#[derive(Debug)]
enum DyLibOpenError {
    DyLibNotFound {
        lib_path: PathBuf,
    },
    LoadedDyLibNotFound {
        pkg: String,
        version: String,
    },
    LoadDyLib {
        lib_path: PathBuf,
        err: libloading::Error,
    },
    NixDbNotFound,
    NixVersionMismatch {
        loaded_version: String,
    },
    OpenNixDb(rusqlite::Error),
    QueryNixDb(rusqlite::Error),
    StoreOutputVariantNotFound {
        pkg: String,
        version: String,
        refs: Vec<String>,
    },
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
            Self::LoadedDyLibNotFound { pkg, version } => {
                write!(
                    f,
                    "couldn't find the already-loaded {pkg}-{version} runtime \
                     dylib"
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
            Self::StoreOutputVariantNotFound { pkg, version, refs } => {
                write!(
                    f,
                    "couldn't find /nix/store output for {pkg}-{version} in \
                     the Nix DB that references {}",
                    refs.join(", ")
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
    pub(super) fn loaded_image_paths<F, T>(fun: F) -> Option<T>
    where
        F: FnMut(&CStr) -> ControlFlow<T>,
    {
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
                Some(callback::<F, T>),
                (&mut state as *mut State<F, T>).cast(),
            );
        }

        state.break_value
    }
}
