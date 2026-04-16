//! This file is generated from Nix 2.33.4 headers. Do not edit it manually.


pub const __bool_true_false_are_defined: u32 = 1;
pub const true_: u32 = 1;
pub const false_: u32 = 0;
#[doc = "No error occurred.\nThis error code is returned when no error has occurred during the function\nexecution."]
pub const err_NIX_OK: err = 0;
#[doc = "An unknown error occurred.\nThis error code is returned when an unknown error occurred during the\nfunction execution."]
pub const err_NIX_ERR_UNKNOWN: err = -1;
#[doc = "An overflow error occurred.\nThis error code is returned when an overflow error occurred during the\nfunction execution."]
pub const err_NIX_ERR_OVERFLOW: err = -2;
#[doc = "A key/index access error occurred in C API functions.\nThis error code is returned when accessing a key, index, or identifier that\ndoes not exist in C API functions. Common scenarios include:\n- Setting keys that don't exist (nix_setting_get, nix_setting_set)\n- List indices that are out of bounds (nix_get_list_byidx*)\n- Attribute names that don't exist (nix_get_attr_byname*)\n- Attribute indices that are out of bounds (nix_get_attr_byidx*, nix_get_attr_name_byidx)\nThis error typically indicates incorrect usage or assumptions about data structure\ncontents, rather than internal Nix evaluation errors.\n> **Note** This error code should ONLY be returned by C API functions themselves,\nnot by underlying Nix evaluation. For example, evaluating `{}.foo` in Nix\nwill throw a normal error (NIX_ERR_NIX_ERROR), not NIX_ERR_KEY."]
pub const err_NIX_ERR_KEY: err = -3;
#[doc = "A generic Nix error occurred.\nThis error code is returned when a generic Nix error occurred during the\nfunction execution."]
pub const err_NIX_ERR_NIX_ERROR: err = -4;
#[doc = "Type for error codes in the Nix system\nThis type can have one of several predefined constants:\n- NIX_OK: No error occurred (0)\n- NIX_ERR_UNKNOWN: An unknown error occurred (-1)\n- NIX_ERR_OVERFLOW: An overflow error occurred (-2)\n- NIX_ERR_KEY: A key/index access error occurred in C API functions (-3)\n- NIX_ERR_NIX_ERROR: A generic Nix error occurred (-4)"]
pub type err = ::core::ffi::c_int;
pub const verbosity_NIX_LVL_ERROR: verbosity = 0;
pub const verbosity_NIX_LVL_WARN: verbosity = 1;
pub const verbosity_NIX_LVL_NOTICE: verbosity = 2;
pub const verbosity_NIX_LVL_INFO: verbosity = 3;
pub const verbosity_NIX_LVL_TALKATIVE: verbosity = 4;
pub const verbosity_NIX_LVL_CHATTY: verbosity = 5;
pub const verbosity_NIX_LVL_DEBUG: verbosity = 6;
pub const verbosity_NIX_LVL_VOMIT: verbosity = 7;
#[doc = "Verbosity level\n> **Note** This should be kept in sync with the C++ implementation (nix::Verbosity)"]
pub type verbosity = ::core::ffi::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct c_context {
    _unused: [u8; 0],
}
#[doc = "Called to get the value of a string owned by Nix.\nThe `start` data is borrowed and the function must not assume that the buffer persists after it returns.\n\n# Arguments\n\n* `start` [in]  - the string to copy.\n* `n` [in]  - the string length.\n* `user_data` [in]  - optional, arbitrary data, passed to the nix_get_string_callback when it's called."]
pub type get_string_callback = ::core::option::Option<
    unsafe extern "C" fn(
        start: *const ::core::ffi::c_char,
        n: ::core::ffi::c_uint,
        user_data: *mut ::core::ffi::c_void,
    ),
>;
unsafe extern "C" {
    #[doc = "Allocate a new nix_c_context.\n@throws std::bad_alloc\n\n# Returns\n\nallocated nix_c_context, owned by the caller. Free using\n`nix_c_context_free`."]
    #[link_name = "\u{1}_nix_c_context_create"]
    pub fn c_context_create() -> *mut c_context;
}
unsafe extern "C" {
    #[doc = "Free a nix_c_context. Does not fail.\n\n# Arguments\n\n* `context` [out]  - The context to free, mandatory."]
    #[link_name = "\u{1}_nix_c_context_free"]
    pub fn c_context_free(context: *mut c_context);
}
unsafe extern "C" {
    #[doc = "Initializes nix_libutil and its dependencies.\nThis function can be called multiple times, but should be called at least\nonce prior to any other nix function.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n\n# Returns\n\nNIX_OK if the initialization is successful, or an error code\notherwise."]
    #[link_name = "\u{1}_nix_libutil_init"]
    pub fn libutil_init(context: *mut c_context) -> err;
}
unsafe extern "C" {
    #[doc = "@defgroup settings Nix configuration settings\n/\n/**\nRetrieves a setting from the nix global configuration.\nThis function requires nix_libutil_init() to be called at least once prior to\nits use.\n\n# Arguments\n\n* `context` [out]  - optional, Stores error information\n* `key` [in]  - The key of the setting to retrieve.\n* `callback` [in]  - Called with the setting value.\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called.\n\n# See also\n\n> [`nix_get_string_callback`]\n\n# Returns\n\nNIX_ERR_KEY if the setting is unknown, or NIX_OK if the setting was retrieved\nsuccessfully."]
    #[link_name = "\u{1}_nix_setting_get"]
    pub fn setting_get(
        context: *mut c_context,
        key: *const ::core::ffi::c_char,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Sets a setting in the nix global configuration.\nUse \"extra-<setting name>\" to append to the setting's value.\nSettings only apply for new State%s. Call nix_plugins_init() when you are\ndone with the settings to load any plugins.\n\n# Arguments\n\n* `context` [out]  - optional, Stores error information\n* `key` [in]  - The key of the setting to set.\n* `value` [in]  - The value to set for the setting.\n\n# Returns\n\nNIX_ERR_KEY if the setting is unknown, or NIX_OK if the setting was\nset successfully."]
    #[link_name = "\u{1}_nix_setting_set"]
    pub fn setting_set(
        context: *mut c_context,
        key: *const ::core::ffi::c_char,
        value: *const ::core::ffi::c_char,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Retrieves the nix library version.\nDoes not fail.\n\n# Returns\n\nA static string representing the version of the nix library."]
    #[link_name = "\u{1}_nix_version_get"]
    pub fn version_get() -> *const ::core::ffi::c_char;
}
unsafe extern "C" {
    #[doc = "@addtogroup errors\n/\n/**\nRetrieves the most recent error message from a context.\n@pre This function should only be called after a previous nix function has\nreturned an error.\n\n# Arguments\n\n* `context` [out]  - optional, the context to store errors in if this function\nfails\n* `ctx` [in]  - the context to retrieve the error message from\n* `n` [out]  - optional: a pointer to an unsigned int that is set to the\nlength of the error.\n\n# Returns\n\nnullptr if no error message was ever set,\na borrowed pointer to the error message otherwise, which is valid\nuntil the next call to a Nix function, or until the context is\ndestroyed."]
    #[link_name = "\u{1}_nix_err_msg"]
    pub fn err_msg(
        context: *mut c_context,
        ctx: *const c_context,
        n: *mut ::core::ffi::c_uint,
    ) -> *const ::core::ffi::c_char;
}
unsafe extern "C" {
    #[doc = "Retrieves the error message from errorInfo in a context.\nUsed to inspect nix Error messages.\n@pre This function should only be called after a previous nix function has\nreturned a NIX_ERR_NIX_ERROR\n\n# Arguments\n\n* `context` [out]  - optional, the context to store errors in if this function\nfails\n* `read_context` [in]  - the context to retrieve the error message from.\n* `callback` [in]  - Called with the error message.\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called.\n\n# See also\n\n> [`nix_get_string_callback`]\n\n# Returns\n\nNIX_OK if there were no errors, an error code otherwise."]
    #[link_name = "\u{1}_nix_err_info_msg"]
    pub fn err_info_msg(
        context: *mut c_context,
        read_context: *const c_context,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Retrieves the error name from a context.\nUsed to inspect nix Error messages.\n@pre This function should only be called after a previous nix function has\nreturned a NIX_ERR_NIX_ERROR\n\n# Arguments\n\n* `context` - optional, the context to store errors in if this function\nfails\n* `read_context` [in]  - the context to retrieve the error message from\n* `callback` [in]  - Called with the error name.\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called.\n\n# See also\n\n> [`nix_get_string_callback`]\n\n# Returns\n\nNIX_OK if there were no errors, an error code otherwise."]
    #[link_name = "\u{1}_nix_err_name"]
    pub fn err_name(
        context: *mut c_context,
        read_context: *const c_context,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Retrieves the most recent error code from a nix_c_context\nEquivalent to reading the first field of the context.\nDoes not fail\n\n# Arguments\n\n* `read_context` [in]  - the context to retrieve the error message from\n\n# Returns\n\nmost recent error code stored in the context."]
    #[link_name = "\u{1}_nix_err_code"]
    pub fn err_code(read_context: *const c_context) -> err;
}
unsafe extern "C" {
    #[doc = "Set an error message on a nix context.\nThis should be used when you want to throw an error from a PrimOp callback.\nAll other use is internal to the API.\n\n# Arguments\n\n* `context` - context to write the error message to, required unless C++ exceptions are supported\n* `err` - The error code to set and return\n* `msg` - The error message to set. This string is copied.\n\n# Returns\n\nthe error code set"]
    #[link_name = "\u{1}_nix_set_err_msg"]
    pub fn set_err_msg(context: *mut c_context, err: err, msg: *const ::core::ffi::c_char) -> err;
}
unsafe extern "C" {
    #[doc = "Clear the error message from a nix context.\nThis is performed implicitly by all functions that accept a context, so\nthis won't be necessary in most cases.\nHowever, if you want to clear the error message without calling another\nfunction, you can use this.\nExample use case: a higher order function that helps with error handling,\nto make it more robust in the following scenario:\n1. A previous call failed, and the error was caught and handled.\n2. The context is reused with our error handling helper function.\n3. The callback passed to the helper function doesn't actually make a call to\na Nix function.\n4. The handled error is raised again, from an unrelated call.\nThis failure can be avoided by clearing the error message after handling it."]
    #[link_name = "\u{1}_nix_clear_err"]
    pub fn clear_err(context: *mut c_context);
}
unsafe extern "C" {
    #[doc = "Sets the verbosity level\n\n# Arguments\n\n* `context` [out]  - Optional, additional error context.\n* `level` [in]  - Verbosity level"]
    #[link_name = "\u{1}_nix_set_verbosity"]
    pub fn set_verbosity(context: *mut c_context, level: verbosity) -> err;
}
pub type wchar_t = ::core::ffi::c_int;
pub type max_align_t = f64;
pub type int_least64_t = i64;
pub type uint_least64_t = u64;
pub type int_fast64_t = i64;
pub type uint_fast64_t = u64;
pub type int_least32_t = i32;
pub type uint_least32_t = u32;
pub type int_fast32_t = i32;
pub type uint_fast32_t = u32;
pub type int_least16_t = i16;
pub type uint_least16_t = u16;
pub type int_fast16_t = i16;
pub type uint_fast16_t = u16;
pub type int_least8_t = i8;
pub type uint_least8_t = u8;
pub type int_fast8_t = i8;
pub type uint_fast8_t = u8;
pub type intmax_t = ::core::ffi::c_long;
pub type uintmax_t = ::core::ffi::c_ulong;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct StorePath {
    _unused: [u8; 0],
}
unsafe extern "C" {
    #[doc = "Copy a StorePath\n\n# Arguments\n\n* `p` [in]  - the path to copy\n\n# Returns\n\na new StorePath"]
    #[link_name = "\u{1}_nix_store_path_clone"]
    pub fn store_path_clone(p: *const StorePath) -> *mut StorePath;
}
unsafe extern "C" {
    #[doc = "Deallocate a StorePath\nDoes not fail.\n\n# Arguments\n\n* `p` [in]  - the path to free"]
    #[link_name = "\u{1}_nix_store_path_free"]
    pub fn store_path_free(p: *mut StorePath);
}
unsafe extern "C" {
    #[doc = "Get the path name (e.g. \"<name>\" in /nix/store/<hash>-<name>)\n\n# Arguments\n\n* `store_path` [in]  - the path to get the name from\n* `callback` [in]  - called with the name\n* `user_data` [in]  - arbitrary data, passed to the callback when it's called."]
    #[link_name = "\u{1}_nix_store_path_name"]
    pub fn store_path_name(
        store_path: *const StorePath,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    );
}
#[doc = "A store path hash\nOnce decoded from \"nix32\" encoding, a store path hash is 20 raw bytes."]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct store_path_hash_part {
    pub bytes: [u8; 20usize],
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of store_path_hash_part"][::core::mem::size_of::<store_path_hash_part>() - 20usize];
    ["Alignment of store_path_hash_part"][::core::mem::align_of::<store_path_hash_part>() - 1usize];
    ["Offset of field: store_path_hash_part::bytes"]
        [::core::mem::offset_of!(store_path_hash_part, bytes) - 0usize];
};
unsafe extern "C" {
    #[doc = "Get the path hash (e.g. \"<hash>\" in /nix/store/<hash>-<name>)\nThe hash is returned as raw bytes, decoded from \"nix32\" encoding.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store_path` [in]  - the path to get the hash from\n* `hash_part_out` [out]  - the decoded hash as 20 raw bytes\n\n# Returns\n\nNIX_OK on success, error code on failure"]
    #[link_name = "\u{1}_nix_store_path_hash"]
    pub fn store_path_hash(
        context: *mut c_context,
        store_path: *const StorePath,
        hash_part_out: *mut store_path_hash_part,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Create a StorePath from its constituent parts (hash and name)\nThis function constructs a store path from a hash and name, without needing\na Store reference or the store directory prefix.\n> **Note** Don't forget to free this path using nix_store_path_free()!\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `hash` [in]  - The store path hash (20 raw bytes)\n* `name` [in]  - The store path name (the part after the hash)\n* `name_len` [in]  - Length of the name string\n\n# Returns\n\nowned store path, NULL on error"]
    #[link_name = "\u{1}_nix_store_create_from_parts"]
    pub fn store_create_from_parts(
        context: *mut c_context,
        hash: *const store_path_hash_part,
        name: *const ::core::ffi::c_char,
        name_len: usize,
    ) -> *mut StorePath;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct derivation {
    _unused: [u8; 0],
}
unsafe extern "C" {
    #[doc = "Copy a `nix_derivation`\n\n# Arguments\n\n* `d` [in]  - the derivation to copy\n\n# Returns\n\na new `nix_derivation`"]
    #[link_name = "\u{1}_nix_derivation_clone"]
    pub fn derivation_clone(d: *const derivation) -> *mut derivation;
}
unsafe extern "C" {
    #[doc = "Deallocate a `nix_derivation`\nDoes not fail.\n\n# Arguments\n\n* `drv` [in]  - the derivation to free"]
    #[link_name = "\u{1}_nix_derivation_free"]
    pub fn derivation_free(drv: *mut derivation);
}
unsafe extern "C" {
    #[doc = "Gets the derivation as a JSON string\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `drv` [in]  - The derivation\n* `callback` [in]  - Called with the JSON string\n* `userdata` [in]  - Arbitrary data passed to the callback"]
    #[link_name = "\u{1}_nix_derivation_to_json"]
    pub fn derivation_to_json(
        context: *mut c_context,
        drv: *const derivation,
        callback: get_string_callback,
        userdata: *mut ::core::ffi::c_void,
    ) -> err;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Store {
    _unused: [u8; 0],
}
unsafe extern "C" {
    #[doc = "Initializes the Nix store library\nThis function should be called before creating a Store\nThis function can be called multiple times.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n\n# Returns\n\nNIX_OK if the initialization was successful, an error code otherwise."]
    #[link_name = "\u{1}_nix_libstore_init"]
    pub fn libstore_init(context: *mut c_context) -> err;
}
unsafe extern "C" {
    #[doc = "Like nix_libstore_init, but does not load the Nix configuration.\nThis is useful when external configuration is not desired, such as when running unit tests."]
    #[link_name = "\u{1}_nix_libstore_init_no_load_config"]
    pub fn libstore_init_no_load_config(context: *mut c_context) -> err;
}
unsafe extern "C" {
    #[doc = "Open a nix store.\nStore instances may share state and resources behind the scenes.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `uri` [in]  - @parblock URI of the Nix store, copied.\nIf `NULL`, the store from the settings will be used.\nNote that `\"auto\"` holds a strange middle ground, reading part of the general environment, but not all of it. It\nignores `NIX_REMOTE` and the `store` option. For this reason, `NULL` is most likely the better choice.\nFor supported store URLs, see [*Store URL format* in the Nix Reference\nManual](https://nix.dev/manual/nix/stable/store/types/#store-url-format).\n@endparblock * `params` [in]  - @parblock optional, null-terminated array of key-value pairs, e.g. {{\"endpoint\",\n\"https://s3.local\"}}.\nSee [*Store Types* in the Nix Reference Manual](https://nix.dev/manual/nix/stable/store/types).\n@endparblock # Returns\n\na Store pointer, NULL in case of errors\n\n# See also\n\n> [`nix_store_free`]"]
    #[link_name = "\u{1}_nix_store_open"]
    pub fn store_open(
        context: *mut c_context,
        uri: *const ::core::ffi::c_char,
        params: *mut *mut *const ::core::ffi::c_char,
    ) -> *mut Store;
}
unsafe extern "C" {
    #[doc = "Deallocate a nix store and free any resources if not also held by other Store instances.\nDoes not fail.\n\n# Arguments\n\n* `store` [in]  - the store to free"]
    #[link_name = "\u{1}_nix_store_free"]
    pub fn store_free(store: *mut Store);
}
unsafe extern "C" {
    #[doc = "get the URI of a nix store\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - nix store reference\n* `callback` [in]  - Called with the URI.\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called.\n\n# See also\n\n> [`nix_get_string_callback`]\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_store_get_uri"]
    pub fn store_get_uri(
        context: *mut c_context,
        store: *mut Store,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "get the storeDir of a Nix store, typically `\"/nix/store\"`\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - nix store reference\n* `callback` [in]  - Called with the URI.\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called.\n\n# See also\n\n> [`nix_get_string_callback`]\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_store_get_storedir"]
    pub fn store_get_storedir(
        context: *mut c_context,
        store: *mut Store,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Parse a Nix store path that includes the store dir into a StorePath\n> **Note** Don't forget to free this path using nix_store_path_free()!\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - nix store reference\n* `path` [in]  - Path string to parse, copied\n\n# Returns\n\nowned store path, NULL on error"]
    #[link_name = "\u{1}_nix_store_parse_path"]
    pub fn store_parse_path(
        context: *mut c_context,
        store: *mut Store,
        path: *const ::core::ffi::c_char,
    ) -> *mut StorePath;
}
unsafe extern "C" {
    #[doc = "Check if a StorePath is valid (i.e. that corresponding store object and its closure of references exists in\nthe store)\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - Nix Store reference\n* `path` [in]  - Path to check\n\n# Returns\n\ntrue or false, error info in context"]
    #[link_name = "\u{1}_nix_store_is_valid_path"]
    pub fn store_is_valid_path(
        context: *mut c_context,
        store: *mut Store,
        path: *const StorePath,
    ) -> bool;
}
unsafe extern "C" {
    #[doc = "Get the physical location of a store path\nA store may reside at a different location than its `storeDir` suggests.\nThis situation is called a relocated store.\nRelocated stores are used during NixOS installation, as well as in restricted computing environments that don't offer\na writable `/nix/store`.\nNot all types of stores support this operation.\n\n# Arguments\n\n* `context` [in]  - Optional, stores error information\n* `store` [in]  - nix store reference\n* `path` [in]  - the path to get the real path from\n* `callback` [in]  - called with the real path\n* `user_data` [in]  - arbitrary data, passed to the callback when it's called."]
    #[link_name = "\u{1}_nix_store_real_path"]
    pub fn store_real_path(
        context: *mut c_context,
        store: *mut Store,
        path: *mut StorePath,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Realise a Nix store path\nBlocking, calls callback once for each realised output.\n> **Note** When working with expressions, consider using e.g. nix_string_realise to get the output. `.drvPath` may not be\naccurate or available in the future. See https://github.com/NixOS/nix/issues/6507\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - Nix Store reference\n* `path` [in]  - Path to build\n* `userdata` [in]  - data to pass to every callback invocation\n* `callback` [in]  - called for every realised output\n\n# Returns\n\nNIX_OK if the build succeeded, or an error code if the build/scheduling/outputs/copying/etc failed.\nOn error, the callback is never invoked and error information is stored in context."]
    #[link_name = "\u{1}_nix_store_realise"]
    pub fn store_realise(
        context: *mut c_context,
        store: *mut Store,
        path: *mut StorePath,
        userdata: *mut ::core::ffi::c_void,
        callback: ::core::option::Option<
            unsafe extern "C" fn(
                userdata: *mut ::core::ffi::c_void,
                outname: *const ::core::ffi::c_char,
                out: *const StorePath,
            ),
        >,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "get the version of a nix store.\nIf the store doesn't have a version (like the dummy store), returns an empty string.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - nix store reference\n* `callback` [in]  - Called with the version.\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called.\n\n# See also\n\n> [`nix_get_string_callback`]\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_store_get_version"]
    pub fn store_get_version(
        context: *mut c_context,
        store: *mut Store,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Create a `nix_derivation` from a JSON representation of that derivation.\n> **Note** Unlike `nix_derivation_to_json`, this needs a `Store`. This is because\nover time we expect the internal representation of derivations in Nix to\ndiffer from accepted derivation formats. The store argument is here to help\nany logic needed to convert from JSON to the internal representation, in\nexcess of just parsing.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information.\n* `store` [in]  - nix store reference.\n* `json` [in]  - JSON of the derivation as a string.\n\n# Returns\n\nA new derivation, or NULL on error. Free with `nix_derivation_free` when done using the `nix_derivation`."]
    #[link_name = "\u{1}_nix_derivation_from_json"]
    pub fn derivation_from_json(
        context: *mut c_context,
        store: *mut Store,
        json: *const ::core::ffi::c_char,
    ) -> *mut derivation;
}
unsafe extern "C" {
    #[doc = "Add the given `nix_derivation` to the given store\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information.\n* `store` [in]  - nix store reference. The derivation will be inserted here.\n* `derivation` [in]  - nix_derivation to insert into the given store."]
    #[link_name = "\u{1}_nix_add_derivation"]
    pub fn add_derivation(
        context: *mut c_context,
        store: *mut Store,
        derivation: *mut derivation,
    ) -> *mut StorePath;
}
unsafe extern "C" {
    #[doc = "Copy the closure of `path` from `srcStore` to `dstStore`.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `srcStore` [in]  - nix source store reference\n* `dstStore` [in]  - nix destination store reference\n* `path` [in]  - Path to copy"]
    #[link_name = "\u{1}_nix_store_copy_closure"]
    pub fn store_copy_closure(
        context: *mut c_context,
        srcStore: *mut Store,
        dstStore: *mut Store,
        path: *mut StorePath,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Gets the closure of a specific store path\n> **Note** The callback borrows each StorePath only for the duration of the call.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - nix store reference\n* `store_path` [in]  - The path to compute from\n* `flip_direction` [in]  - If false, compute the forward closure (paths referenced by any store path in the closure).\nIf true, compute the backward closure (paths that reference any store path in the closure).\n* `include_outputs` [in]  - If flip_direction is false: for any derivation in the closure, include its outputs.\nIf flip_direction is true: for any output in the closure, include derivations that produce\nit.\n* `include_derivers` [in]  - If flip_direction is false: for any output in the closure, include the derivation that\nproduced it.\nIf flip_direction is true: for any derivation in the closure, include its outputs.\n* `callback` [in]  - The function to call for every store path, in no particular order\n* `userdata` [in]  - The userdata to pass to the callback"]
    #[link_name = "\u{1}_nix_store_get_fs_closure"]
    pub fn store_get_fs_closure(
        context: *mut c_context,
        store: *mut Store,
        store_path: *const StorePath,
        flip_direction: bool,
        include_outputs: bool,
        include_derivers: bool,
        userdata: *mut ::core::ffi::c_void,
        callback: ::core::option::Option<
            unsafe extern "C" fn(
                context: *mut c_context,
                userdata: *mut ::core::ffi::c_void,
                store_path: *const StorePath,
            ),
        >,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Returns the derivation associated with the store path\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - The nix store\n* `path` [in]  - The nix store path\n\n# Returns\n\nA new derivation, or NULL on error. Free with `nix_derivation_free` when done using the `nix_derivation`."]
    #[link_name = "\u{1}_nix_store_drv_from_store_path"]
    pub fn store_drv_from_store_path(
        context: *mut c_context,
        store: *mut Store,
        path: *const StorePath,
    ) -> *mut derivation;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct eval_state_builder {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct EvalState {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct value {
    _unused: [u8; 0],
}
#[doc = "> **Deprecated** Use nix_value instead"]
pub type Value = value;
unsafe extern "C" {
    #[doc = "Initialize the Nix language evaluator.\n@ingroup libexpr_init\nThis function must be called at least once,\nat some point before constructing a EvalState for the first time.\nThis function can be called multiple times, and is idempotent.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n\n# Returns\n\nNIX_OK if the initialization was successful, an error code otherwise."]
    #[link_name = "\u{1}_nix_libexpr_init"]
    pub fn libexpr_init(context: *mut c_context) -> err;
}
unsafe extern "C" {
    #[doc = "Parses and evaluates a Nix expression from a string.\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - The state of the evaluation.\n* `expr` [in]  - The Nix expression to parse.\n* `path` [in]  - The file path to associate with the expression.\nThis is required for expressions that contain relative paths (such as `./.`) that are resolved relative to the given\ndirectory.\n* `value` [out]  - The result of the evaluation. You must allocate this\nyourself.\n\n# Returns\n\nNIX_OK if the evaluation was successful, an error code otherwise."]
    #[link_name = "\u{1}_nix_expr_eval_from_string"]
    pub fn expr_eval_from_string(
        context: *mut c_context,
        state: *mut EvalState,
        expr: *const ::core::ffi::c_char,
        path: *const ::core::ffi::c_char,
        value: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Calls a Nix function with an argument.\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - The state of the evaluation.\n* `fn` [in]  - The Nix function to call.\n* `arg` [in]  - The argument to pass to the function.\n* `value` [out]  - The result of the function call.\n\n# Returns\n\nNIX_OK if the function call was successful, an error code otherwise.\n\n# See also\n\n> [`nix_init_apply()`] for a similar function that does not performs the call immediately, but stores it as a thunk.\nNote the different argument order."]
    #[link_name = "\u{1}_nix_value_call"]
    pub fn value_call(
        context: *mut c_context,
        state: *mut EvalState,
        fn_: *mut value,
        arg: *mut value,
        value: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Calls a Nix function with multiple arguments.\n@ingroup value_create\nTechnically these are functions that return functions. It is common for Nix\nfunctions to be curried, so this function is useful for calling them.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - The state of the evaluation.\n* `fn` [in]  - The Nix function to call.\n* `nargs` [in]  - The number of arguments.\n* `args` [in]  - The arguments to pass to the function.\n* `value` [out]  - The result of the function call.\n\n# See also\n\n> [`nix_value_call`]     For the single argument primitive.\n> [`NIX_VALUE_CALL`]           For a macro that wraps this function for convenience."]
    #[link_name = "\u{1}_nix_value_call_multi"]
    pub fn value_call_multi(
        context: *mut c_context,
        state: *mut EvalState,
        fn_: *mut value,
        nargs: usize,
        args: *mut *mut value,
        value: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Forces the evaluation of a Nix value.\n@ingroup value_create\nThe Nix interpreter is lazy, and not-yet-evaluated values can be\nof type NIX_TYPE_THUNK instead of their actual value.\nThis function mutates such a `nix_value`, so that, if successful, it has its final type.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - The state of the evaluation.\n* `value` [in,out]  - The Nix value to force.\n@post value is not of type NIX_TYPE_THUNK\n\n# Returns\n\nNIX_OK if the force operation was successful, an error code\notherwise."]
    #[link_name = "\u{1}_nix_value_force"]
    pub fn value_force(context: *mut c_context, state: *mut EvalState, value: *mut value) -> err;
}
unsafe extern "C" {
    #[doc = "Forces the deep evaluation of a Nix value.\nRecursively calls nix_value_force\n\n# See also\n\n> [`nix_value_force`]\n@warning Calling this function on a recursive data structure will cause a\nstack overflow.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - The state of the evaluation.\n* `value` [in,out]  - The Nix value to force.\n\n# Returns\n\nNIX_OK if the deep force operation was successful, an error code\notherwise."]
    #[link_name = "\u{1}_nix_value_force_deep"]
    pub fn value_force_deep(
        context: *mut c_context,
        state: *mut EvalState,
        value: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Create a new nix_eval_state_builder\n@ingroup libexpr_init\nThe settings are initialized to their default value.\nValues can be sourced elsewhere with nix_eval_state_builder_load.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `store` [in]  - The Nix store to use.\n\n# Returns\n\nA new nix_eval_state_builder or NULL on failure. Call nix_eval_state_builder_free() when you're done."]
    #[link_name = "\u{1}_nix_eval_state_builder_new"]
    pub fn eval_state_builder_new(
        context: *mut c_context,
        store: *mut Store,
    ) -> *mut eval_state_builder;
}
unsafe extern "C" {
    #[doc = "Read settings from the ambient environment\n@ingroup libexpr_init\nSettings are sourced from environment variables and configuration files,\nas documented in the Nix manual.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `builder` [out]  - The builder to modify.\n\n# Returns\n\nNIX_OK if successful, an error code otherwise."]
    #[link_name = "\u{1}_nix_eval_state_builder_load"]
    pub fn eval_state_builder_load(
        context: *mut c_context,
        builder: *mut eval_state_builder,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Set the lookup path for `<...>` expressions\n@ingroup libexpr_init\n\n# Arguments\n\n* `context` [in]  - Optional, stores error information\n* `builder` [in]  - The builder to modify.\n* `lookupPath` [in]  - Null-terminated array of strings corresponding to entries in NIX_PATH."]
    #[link_name = "\u{1}_nix_eval_state_builder_set_lookup_path"]
    pub fn eval_state_builder_set_lookup_path(
        context: *mut c_context,
        builder: *mut eval_state_builder,
        lookupPath: *mut *const ::core::ffi::c_char,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Create a new Nix language evaluator state\n@ingroup libexpr_init\nThe builder becomes unusable after this call. Remember to call nix_eval_state_builder_free()\nafter building the state.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `builder` [in]  - The builder to use and free\n\n# Returns\n\nA new Nix state or NULL on failure. Call nix_state_free() when you're done.\n\n# See also\n\n> [`nix_eval_state_builder_new,`] nix_eval_state_builder_free"]
    #[link_name = "\u{1}_nix_eval_state_build"]
    pub fn eval_state_build(
        context: *mut c_context,
        builder: *mut eval_state_builder,
    ) -> *mut EvalState;
}
unsafe extern "C" {
    #[doc = "Free a nix_eval_state_builder\n@ingroup libexpr_init\nDoes not fail.\n\n# Arguments\n\n* `builder` [in]  - The builder to free."]
    #[link_name = "\u{1}_nix_eval_state_builder_free"]
    pub fn eval_state_builder_free(builder: *mut eval_state_builder);
}
unsafe extern "C" {
    #[doc = "Create a new Nix language evaluator state\n@ingroup libexpr_init\nFor more control, use nix_eval_state_builder\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `lookupPath` [in]  - Null-terminated array of strings corresponding to entries in NIX_PATH.\n* `store` [in]  - The Nix store to use.\n\n# Returns\n\nA new Nix state or NULL on failure. Call nix_state_free() when you're done.\n\n# See also\n\n> [`nix_state_builder_new`]"]
    #[link_name = "\u{1}_nix_state_create"]
    pub fn state_create(
        context: *mut c_context,
        lookupPath: *mut *const ::core::ffi::c_char,
        store: *mut Store,
    ) -> *mut EvalState;
}
unsafe extern "C" {
    #[doc = "Frees a Nix state.\n@ingroup libexpr_init\nDoes not fail.\n\n# Arguments\n\n* `state` [in]  - The state to free."]
    #[link_name = "\u{1}_nix_state_free"]
    pub fn state_free(state: *mut EvalState);
}
unsafe extern "C" {
    #[doc = "Increment the garbage collector reference counter for the given object.\nThe Nix language evaluator C API keeps track of alive objects by reference counting.\nWhen you're done with a refcounted pointer, call nix_gc_decref().\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `object` [in]  - The object to keep alive"]
    #[link_name = "\u{1}_nix_gc_incref"]
    pub fn gc_incref(context: *mut c_context, object: *const ::core::ffi::c_void) -> err;
}
unsafe extern "C" {
    #[doc = "Decrement the garbage collector reference counter for the given object\n> **Deprecated** We are phasing out the general nix_gc_decref() in favor of type-specified free functions, such as\nnix_value_decref().\nWe also provide typed `nix_*_decref` functions, which are\n- safer to use\n- easier to integrate when deriving bindings\n- allow more flexibility\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `object` [in]  - The object to stop referencing"]
    #[link_name = "\u{1}_nix_gc_decref"]
    pub fn gc_decref(context: *mut c_context, object: *const ::core::ffi::c_void) -> err;
}
unsafe extern "C" {
    #[doc = "Trigger the garbage collector manually\nYou should not need to do this, but it can be useful for debugging."]
    #[link_name = "\u{1}_nix_gc_now"]
    pub fn gc_now();
}
unsafe extern "C" {
    #[doc = "Register a callback that gets called when the object is garbage\ncollected.\n> **Note** Objects can only have a single finalizer. This function overwrites existing values\nsilently.\n\n# Arguments\n\n* `obj` [in]  - the object to watch\n* `cd` [in]  - the data to pass to the finalizer\n* `finalizer` [in]  - the callback function, called with obj and cd"]
    #[link_name = "\u{1}_nix_gc_register_finalizer"]
    pub fn gc_register_finalizer(
        obj: *mut ::core::ffi::c_void,
        cd: *mut ::core::ffi::c_void,
        finalizer: ::core::option::Option<
            unsafe extern "C" fn(obj: *mut ::core::ffi::c_void, cd: *mut ::core::ffi::c_void),
        >,
    );
}
#[doc = "Unevaluated expression\nThunks often contain an expression and closure, but may contain other\nrepresentations too.\nTheir state is mutable, unlike that of the other types."]
pub const ValueType_NIX_TYPE_THUNK: ValueType = 0;
#[doc = "A 64 bit signed integer."]
pub const ValueType_NIX_TYPE_INT: ValueType = 1;
#[doc = "IEEE 754 double precision floating point number\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-float](https://nix.dev/manual/nix/latest/language/types.html#type-float)"]
pub const ValueType_NIX_TYPE_FLOAT: ValueType = 2;
#[doc = "Boolean true or false value\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-bool](https://nix.dev/manual/nix/latest/language/types.html#type-bool)"]
pub const ValueType_NIX_TYPE_BOOL: ValueType = 3;
#[doc = "String value with context\nString content may contain arbitrary bytes, not necessarily UTF-8.\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-string](https://nix.dev/manual/nix/latest/language/types.html#type-string)"]
pub const ValueType_NIX_TYPE_STRING: ValueType = 4;
#[doc = "Filesystem path\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-path](https://nix.dev/manual/nix/latest/language/types.html#type-path)"]
pub const ValueType_NIX_TYPE_PATH: ValueType = 5;
#[doc = "Null value\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-null](https://nix.dev/manual/nix/latest/language/types.html#type-null)"]
pub const ValueType_NIX_TYPE_NULL: ValueType = 6;
#[doc = "Attribute set (key-value mapping)\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-attrs](https://nix.dev/manual/nix/latest/language/types.html#type-attrs)"]
pub const ValueType_NIX_TYPE_ATTRS: ValueType = 7;
#[doc = "Ordered list of values\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-list](https://nix.dev/manual/nix/latest/language/types.html#type-list)"]
pub const ValueType_NIX_TYPE_LIST: ValueType = 8;
#[doc = "Function (lambda or builtin)\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/types.html#type-function](https://nix.dev/manual/nix/latest/language/types.html#type-function)"]
pub const ValueType_NIX_TYPE_FUNCTION: ValueType = 9;
#[doc = "External value from C++ plugins or C API\n\n# See also\n\n> [`Externals`]"]
pub const ValueType_NIX_TYPE_EXTERNAL: ValueType = 10;
#[doc = "Represents the state of a Nix value\nThunk values (NIX_TYPE_THUNK) change to their final, unchanging type when forced.\n\n# See also\n\n> [https://nix.dev/manual/nix/latest/language/evaluation.html](https://nix.dev/manual/nix/latest/language/evaluation.html)\n@enum ValueType\n@ingroup value"]
pub type ValueType = ::core::ffi::c_uint;
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct BindingsBuilder {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ListBuilder {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PrimOp {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ExternalValue {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct realised_string {
    _unused: [u8; 0],
}
#[doc = "Function pointer for primops\n@ingroup primops\nWhen you want to return an error, call nix_set_err_msg(context, NIX_ERR_UNKNOWN, \"your error message here\").\n\n# Arguments\n\n* `user_data` [in]  - Arbitrary data that was initially supplied to nix_alloc_primop\n* `context` [out]  - Stores error information.\n* `state` [in]  - Evaluator state\n* `args` [in]  - list of arguments. Note that these can be thunks and should be forced using nix_value_force before\nuse.\n* `ret` [out]  - return value\n\n# See also\n\n> [`nix_alloc_primop,`] nix_init_primop"]
pub type PrimOpFun = ::core::option::Option<
    unsafe extern "C" fn(
        user_data: *mut ::core::ffi::c_void,
        context: *mut c_context,
        state: *mut EvalState,
        args: *mut *mut value,
        ret: *mut value,
    ),
>;
unsafe extern "C" {
    #[doc = "Allocate a PrimOp\n@ingroup primops\nCall nix_gc_decref() when you're done with the returned PrimOp.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `fun` [in]  - callback\n* `arity` [in]  - expected number of function arguments\n* `name` [in]  - function name\n* `args` [in]  - array of argument names, NULL-terminated\n* `doc` [in]  - optional, documentation for this primop\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called\n\n# Returns\n\nprimop, or null in case of errors\n\n# See also\n\n> [`nix_init_primop`]"]
    #[link_name = "\u{1}_nix_alloc_primop"]
    pub fn alloc_primop(
        context: *mut c_context,
        fun: PrimOpFun,
        arity: ::core::ffi::c_int,
        name: *const ::core::ffi::c_char,
        args: *mut *const ::core::ffi::c_char,
        doc: *const ::core::ffi::c_char,
        user_data: *mut ::core::ffi::c_void,
    ) -> *mut PrimOp;
}
unsafe extern "C" {
    #[doc = "add a primop to the `builtins` attribute set\n@ingroup primops\nOnly applies to States created after this call.\nMoves your PrimOp content into the global evaluator registry, meaning\nyour input PrimOp pointer becomes invalid. The PrimOp must not be used\nwith nix_init_primop() before or after this call, as this would cause\nundefined behavior.\nYou must call nix_gc_decref() on the original PrimOp pointer\nafter this call to release your reference.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `primOp` [in]  - PrimOp to register\n\n# Returns\n\nerror code, NIX_OK on success"]
    #[link_name = "\u{1}_nix_register_primop"]
    pub fn register_primop(context: *mut c_context, primOp: *mut PrimOp) -> err;
}
unsafe extern "C" {
    #[doc = "Allocate a Nix value\n@ingroup value_create\nCall nix_value_decref() when you're done with the pointer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - nix evaluator state\n\n# Returns\n\nvalue, or null in case of errors"]
    #[link_name = "\u{1}_nix_alloc_value"]
    pub fn alloc_value(context: *mut c_context, state: *mut EvalState) -> *mut value;
}
unsafe extern "C" {
    #[doc = "Increment the garbage collector reference counter for the given `nix_value`.\n@ingroup value\nThe Nix language evaluator C API keeps track of alive objects by reference counting.\nWhen you're done with a refcounted pointer, call nix_value_decref().\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - The object to keep alive"]
    #[link_name = "\u{1}_nix_value_incref"]
    pub fn value_incref(context: *mut c_context, value: *mut value) -> err;
}
unsafe extern "C" {
    #[doc = "Decrement the garbage collector reference counter for the given object\n@ingroup value\nWhen the counter reaches zero, the `nix_value` object becomes invalid.\nThe data referenced by `nix_value` may not be deallocated until the memory\ngarbage collector has run, but deallocation is not guaranteed.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - The object to stop referencing"]
    #[link_name = "\u{1}_nix_value_decref"]
    pub fn value_decref(context: *mut c_context, value: *mut value) -> err;
}
unsafe extern "C" {
    #[doc = "Get value type\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\ntype of nix value"]
    #[link_name = "\u{1}_nix_get_type"]
    pub fn get_type(context: *mut c_context, value: *const value) -> ValueType;
}
unsafe extern "C" {
    #[doc = "Get type name of value as defined in the evaluator\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\ntype name string, free with free()"]
    #[link_name = "\u{1}_nix_get_typename"]
    pub fn get_typename(context: *mut c_context, value: *const value)
        -> *const ::core::ffi::c_char;
}
unsafe extern "C" {
    #[doc = "Get boolean value\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\ntrue or false, error info via context"]
    #[link_name = "\u{1}_nix_get_bool"]
    pub fn get_bool(context: *mut c_context, value: *const value) -> bool;
}
unsafe extern "C" {
    #[doc = "Get the raw string\n@ingroup value_extract\nThis may contain placeholders.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n* `callback` [in]  - Called with the string value.\n* `user_data` [in]  - optional, arbitrary data, passed to the callback when it's called.\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_get_string"]
    pub fn get_string(
        context: *mut c_context,
        value: *const value,
        callback: get_string_callback,
        user_data: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Get path as string\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\nstring valid while value is valid, NULL in case of error"]
    #[link_name = "\u{1}_nix_get_path_string"]
    pub fn get_path_string(
        context: *mut c_context,
        value: *const value,
    ) -> *const ::core::ffi::c_char;
}
unsafe extern "C" {
    #[doc = "Get the length of a list\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\nlength of list, error info via context"]
    #[link_name = "\u{1}_nix_get_list_size"]
    pub fn get_list_size(context: *mut c_context, value: *const value) -> ::core::ffi::c_uint;
}
unsafe extern "C" {
    #[doc = "Get the element count of an attrset\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\nattrset element count, error info via context"]
    #[link_name = "\u{1}_nix_get_attrs_size"]
    pub fn get_attrs_size(context: *mut c_context, value: *const value) -> ::core::ffi::c_uint;
}
unsafe extern "C" {
    #[doc = "Get float value in 64 bits\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\nfloat contents, error info via context"]
    #[link_name = "\u{1}_nix_get_float"]
    pub fn get_float(context: *mut c_context, value: *const value) -> f64;
}
unsafe extern "C" {
    #[doc = "Get int value\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\nint contents, error info via context"]
    #[link_name = "\u{1}_nix_get_int"]
    pub fn get_int(context: *mut c_context, value: *const value) -> i64;
}
unsafe extern "C" {
    #[doc = "Get external reference\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n\n# Returns\n\nreference valid while value is valid. Call nix_gc_incref() if you need it to live longer, then only in that\ncase call nix_gc_decref() when done. NULL in case of error"]
    #[link_name = "\u{1}_nix_get_external"]
    pub fn get_external(context: *mut c_context, value: *mut value) -> *mut ExternalValue;
}
unsafe extern "C" {
    #[doc = "Get the ix'th element of a list\n@ingroup value_extract\nCall nix_value_decref() when you're done with the pointer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n* `state` [in]  - nix evaluator state\n* `ix` [in]  - list element to get\n\n# Returns\n\nvalue, NULL in case of errors"]
    #[link_name = "\u{1}_nix_get_list_byidx"]
    pub fn get_list_byidx(
        context: *mut c_context,
        value: *const value,
        state: *mut EvalState,
        ix: ::core::ffi::c_uint,
    ) -> *mut value;
}
unsafe extern "C" {
    #[doc = "Get the ix'th element of a list without forcing evaluation of the element\n@ingroup value_extract\nReturns the list element without forcing its evaluation, allowing access to lazy values.\nThe list value itself must already be evaluated.\nCall nix_value_decref() when you're done with the pointer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect (must be an evaluated list)\n* `state` [in]  - nix evaluator state\n* `ix` [in]  - list element to get\n\n# Returns\n\nvalue, NULL in case of errors"]
    #[link_name = "\u{1}_nix_get_list_byidx_lazy"]
    pub fn get_list_byidx_lazy(
        context: *mut c_context,
        value: *const value,
        state: *mut EvalState,
        ix: ::core::ffi::c_uint,
    ) -> *mut value;
}
unsafe extern "C" {
    #[doc = "Get an attr by name\n@ingroup value_extract\nCall nix_value_decref() when you're done with the pointer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n* `state` [in]  - nix evaluator state\n* `name` [in]  - attribute name\n\n# Returns\n\nvalue, NULL in case of errors"]
    #[link_name = "\u{1}_nix_get_attr_byname"]
    pub fn get_attr_byname(
        context: *mut c_context,
        value: *const value,
        state: *mut EvalState,
        name: *const ::core::ffi::c_char,
    ) -> *mut value;
}
unsafe extern "C" {
    #[doc = "Get an attribute value by attribute name, without forcing evaluation of the attribute's value\n@ingroup value_extract\nReturns the attribute value without forcing its evaluation, allowing access to lazy values.\nThe attribute set value itself must already be evaluated.\nCall nix_value_decref() when you're done with the pointer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect (must be an evaluated attribute set)\n* `state` [in]  - nix evaluator state\n* `name` [in]  - attribute name\n\n# Returns\n\nvalue, NULL in case of errors"]
    #[link_name = "\u{1}_nix_get_attr_byname_lazy"]
    pub fn get_attr_byname_lazy(
        context: *mut c_context,
        value: *const value,
        state: *mut EvalState,
        name: *const ::core::ffi::c_char,
    ) -> *mut value;
}
unsafe extern "C" {
    #[doc = "Check if an attribute name exists on a value\n@ingroup value_extract\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n* `state` [in]  - nix evaluator state\n* `name` [in]  - attribute name\n\n# Returns\n\nvalue, error info via context"]
    #[link_name = "\u{1}_nix_has_attr_byname"]
    pub fn has_attr_byname(
        context: *mut c_context,
        value: *const value,
        state: *mut EvalState,
        name: *const ::core::ffi::c_char,
    ) -> bool;
}
unsafe extern "C" {
    #[doc = "Get an attribute by index\n@ingroup value_extract\nAlso gives you the name.\nAttributes are returned in an unspecified order which is NOT suitable for\nreproducible operations. In Nix's domain, reproducibility is paramount. The caller\nis responsible for sorting the attributes or storing them in an ordered map to\nensure deterministic behavior in your application.\n> **Note** When Nix does sort attributes, which it does for virtually all intermediate\noperations and outputs, it uses byte-wise lexicographic order (equivalent to\nlexicographic order by Unicode scalar value for valid UTF-8). We recommend\napplying this same ordering for consistency.\nCall nix_value_decref() when you're done with the pointer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n* `state` [in]  - nix evaluator state\n* `i` [in]  - attribute index\n* `name` [out]  - will store a pointer to the attribute name, valid until state is freed\n\n# Returns\n\nvalue, NULL in case of errors"]
    #[link_name = "\u{1}_nix_get_attr_byidx"]
    pub fn get_attr_byidx(
        context: *mut c_context,
        value: *mut value,
        state: *mut EvalState,
        i: ::core::ffi::c_uint,
        name: *mut *const ::core::ffi::c_char,
    ) -> *mut value;
}
unsafe extern "C" {
    #[doc = "Get an attribute by index, without forcing evaluation of the attribute's value\n@ingroup value_extract\nAlso gives you the name.\nReturns the attribute value without forcing its evaluation, allowing access to lazy values.\nThe attribute set value itself must already have been evaluated.\nAttributes are returned in an unspecified order which is NOT suitable for\nreproducible operations. In Nix's domain, reproducibility is paramount. The caller\nis responsible for sorting the attributes or storing them in an ordered map to\nensure deterministic behavior in your application.\n> **Note** When Nix does sort attributes, which it does for virtually all intermediate\noperations and outputs, it uses byte-wise lexicographic order (equivalent to\nlexicographic order by Unicode scalar value for valid UTF-8). We recommend\napplying this same ordering for consistency.\nCall nix_value_decref() when you're done with the pointer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect (must be an evaluated attribute set)\n* `state` [in]  - nix evaluator state\n* `i` [in]  - attribute index\n* `name` [out]  - will store a pointer to the attribute name, valid until state is freed\n\n# Returns\n\nvalue, NULL in case of errors"]
    #[link_name = "\u{1}_nix_get_attr_byidx_lazy"]
    pub fn get_attr_byidx_lazy(
        context: *mut c_context,
        value: *mut value,
        state: *mut EvalState,
        i: ::core::ffi::c_uint,
        name: *mut *const ::core::ffi::c_char,
    ) -> *mut value;
}
unsafe extern "C" {
    #[doc = "Get an attribute name by index\n@ingroup value_extract\nReturns the attribute name without forcing evaluation of the attribute's value.\nAttributes are returned in an unspecified order which is NOT suitable for\nreproducible operations. In Nix's domain, reproducibility is paramount. The caller\nis responsible for sorting the attributes or storing them in an ordered map to\nensure deterministic behavior in your application.\n> **Note** When Nix does sort attributes, which it does for virtually all intermediate\noperations and outputs, it uses byte-wise lexicographic order (equivalent to\nlexicographic order by Unicode scalar value for valid UTF-8). We recommend\napplying this same ordering for consistency.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value to inspect\n* `state` [in]  - nix evaluator state\n* `i` [in]  - attribute index\n\n# Returns\n\nname string valid until state is freed, NULL in case of errors"]
    #[link_name = "\u{1}_nix_get_attr_name_byidx"]
    pub fn get_attr_name_byidx(
        context: *mut c_context,
        value: *mut value,
        state: *mut EvalState,
        i: ::core::ffi::c_uint,
    ) -> *const ::core::ffi::c_char;
}
unsafe extern "C" {
    #[doc = "@name Initializers\nValues are typically \"returned\" by initializing already allocated memory that serves as the return value.\nFor this reason, the construction of values is not tied their allocation.\nNix is a language with immutable values. Respect this property by only initializing Values once; and only initialize\nValues that are meant to be initialized by you. Failing to adhere to these rules may lead to undefined behavior.\n/\n/**@{*/ /** Set boolean value\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `b` [in]  - the boolean value\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_bool"]
    pub fn init_bool(context: *mut c_context, value: *mut value, b: bool) -> err;
}
unsafe extern "C" {
    #[doc = "Set a string\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `str` [in]  - the string, copied\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_string"]
    pub fn init_string(
        context: *mut c_context,
        value: *mut value,
        str_: *const ::core::ffi::c_char,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Set a path\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `str` [in]  - the path string, copied\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_path_string"]
    pub fn init_path_string(
        context: *mut c_context,
        s: *mut EvalState,
        value: *mut value,
        str_: *const ::core::ffi::c_char,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Set a float\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `d` [in]  - the float, 64-bits\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_float"]
    pub fn init_float(context: *mut c_context, value: *mut value, d: f64) -> err;
}
unsafe extern "C" {
    #[doc = "Set an int\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `i` [in]  - the int\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_int"]
    pub fn init_int(context: *mut c_context, value: *mut value, i: i64) -> err;
}
unsafe extern "C" {
    #[doc = "Set null\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_null"]
    pub fn init_null(context: *mut c_context, value: *mut value) -> err;
}
unsafe extern "C" {
    #[doc = "Set the value to a thunk that will perform a function application when needed.\n@ingroup value_create\nThunks may be put into attribute sets and lists to perform some computation lazily; on demand.\nHowever, note that in some places, a thunk must not be returned, such as in the return value of a PrimOp.\nIn such cases, you may use nix_value_call() instead (but note the different argument order).\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `fn` [in]  - function to call\n* `arg` [in]  - argument to pass\n\n# Returns\n\nerror code, NIX_OK on successful initialization.\n\n# See also\n\n> [`nix_value_call()`] for a similar function that performs the call immediately and only stores the return value.\nNote the different argument order."]
    #[link_name = "\u{1}_nix_init_apply"]
    pub fn init_apply(
        context: *mut c_context,
        value: *mut value,
        fn_: *mut value,
        arg: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Set an external value\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `val` [in]  - the external value to set. Will be GC-referenced by the value.\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_external"]
    pub fn init_external(
        context: *mut c_context,
        value: *mut value,
        val: *mut ExternalValue,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Create a list from a list builder\n@ingroup value_create\nAfter this call, the list builder becomes invalid and cannot be used again.\nThe only necessary next step is to free it with nix_list_builder_free().\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `list_builder` [in]  - list builder to use\n* `value` [out]  - Nix value to modify\n\n# Returns\n\nerror code, NIX_OK on success.\n\n# See also\n\n> [`nix_list_builder_free`]"]
    #[link_name = "\u{1}_nix_make_list"]
    pub fn make_list(
        context: *mut c_context,
        list_builder: *mut ListBuilder,
        value: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Create a list builder\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - nix evaluator state\n* `capacity` [in]  - how many bindings you'll add. Don't exceed.\n\n# Returns\n\nlist builder. Call nix_list_builder_free() when you're done."]
    #[link_name = "\u{1}_nix_make_list_builder"]
    pub fn make_list_builder(
        context: *mut c_context,
        state: *mut EvalState,
        capacity: usize,
    ) -> *mut ListBuilder;
}
unsafe extern "C" {
    #[doc = "Insert bindings into a builder\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `list_builder` [in]  - ListBuilder to insert into\n* `index` [in]  - index to manipulate\n* `value` [in]  - value to insert\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_list_builder_insert"]
    pub fn list_builder_insert(
        context: *mut c_context,
        list_builder: *mut ListBuilder,
        index: ::core::ffi::c_uint,
        value: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Free a list builder\nDoes not fail.\n\n# Arguments\n\n* `list_builder` [in]  - The builder to free."]
    #[link_name = "\u{1}_nix_list_builder_free"]
    pub fn list_builder_free(list_builder: *mut ListBuilder);
}
unsafe extern "C" {
    #[doc = "Create an attribute set from a bindings builder\n@ingroup value_create\nAfter this call, the bindings builder becomes invalid and cannot be used again.\nThe only necessary next step is to free it with nix_bindings_builder_free().\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `b` [in]  - bindings builder to use\n\n# Returns\n\nerror code, NIX_OK on success.\n\n# See also\n\n> [`nix_bindings_builder_free`]"]
    #[link_name = "\u{1}_nix_make_attrs"]
    pub fn make_attrs(context: *mut c_context, value: *mut value, b: *mut BindingsBuilder) -> err;
}
unsafe extern "C" {
    #[doc = "Set primop\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `op` [in]  - primop, will be gc-referenced by the value\n\n# See also\n\n> [`nix_alloc_primop`]\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_init_primop"]
    pub fn init_primop(context: *mut c_context, value: *mut value, op: *mut PrimOp) -> err;
}
unsafe extern "C" {
    #[doc = "Copy from another value\n@ingroup value_create\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [out]  - Nix value to modify\n* `source` [in]  - value to copy from\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_copy_value"]
    pub fn copy_value(context: *mut c_context, value: *mut value, source: *const value) -> err;
}
unsafe extern "C" {
    #[doc = "Create a bindings builder\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `state` [in]  - nix evaluator state\n* `capacity` [in]  - how many bindings you'll add. Don't exceed.\n\n# Returns\n\nbindings builder. Call nix_bindings_builder_free() when you're done."]
    #[link_name = "\u{1}_nix_make_bindings_builder"]
    pub fn make_bindings_builder(
        context: *mut c_context,
        state: *mut EvalState,
        capacity: usize,
    ) -> *mut BindingsBuilder;
}
unsafe extern "C" {
    #[doc = "Insert bindings into a builder\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `builder` [in]  - BindingsBuilder to insert into\n* `name` [in]  - attribute name, only used for the duration of the call.\n* `value` [in]  - value to give the binding\n\n# Returns\n\nerror code, NIX_OK on success."]
    #[link_name = "\u{1}_nix_bindings_builder_insert"]
    pub fn bindings_builder_insert(
        context: *mut c_context,
        builder: *mut BindingsBuilder,
        name: *const ::core::ffi::c_char,
        value: *mut value,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Free a bindings builder\nDoes not fail.\n\n# Arguments\n\n* `builder` [in]  - the builder to free"]
    #[link_name = "\u{1}_nix_bindings_builder_free"]
    pub fn bindings_builder_free(builder: *mut BindingsBuilder);
}
unsafe extern "C" {
    #[doc = "Realise a string context.\nThis will\n- realise the store paths referenced by the string's context, and\n- perform the replacement of placeholders.\n- create temporary garbage collection roots for the store paths, for\nthe lifetime of the current process.\n- log to stderr\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `value` [in]  - Nix value, which must be a string\n* `state` [in]  - Nix evaluator state\n* `isIFD` [in]  - If true, disallow derivation outputs if setting `allow-import-from-derivation` is false.\nYou should set this to true when this call is part of a primop.\nYou should set this to false when building for your application's purpose.\n\n# Returns\n\nNULL if failed, or a new nix_realised_string, which must be freed with nix_realised_string_free"]
    #[link_name = "\u{1}_nix_string_realise"]
    pub fn string_realise(
        context: *mut c_context,
        state: *mut EvalState,
        value: *mut value,
        isIFD: bool,
    ) -> *mut realised_string;
}
unsafe extern "C" {
    #[doc = "Start of the string\n\n# Arguments\n\n* `realised_string` [in]  -\n\n# Returns\n\npointer to the start of the string, valid until realised_string is freed. It may not be null-terminated."]
    #[link_name = "\u{1}_nix_realised_string_get_buffer_start"]
    pub fn realised_string_get_buffer_start(
        realised_string: *mut realised_string,
    ) -> *const ::core::ffi::c_char;
}
unsafe extern "C" {
    #[doc = "Length of the string\n\n# Arguments\n\n* `realised_string` [in]  -\n\n# Returns\n\nlength of the string in bytes"]
    #[link_name = "\u{1}_nix_realised_string_get_buffer_size"]
    pub fn realised_string_get_buffer_size(realised_string: *mut realised_string) -> usize;
}
unsafe extern "C" {
    #[doc = "Number of realised store paths\n\n# Arguments\n\n* `realised_string` [in]  -\n\n# Returns\n\nnumber of realised store paths that were referenced by the string via its context"]
    #[link_name = "\u{1}_nix_realised_string_get_store_path_count"]
    pub fn realised_string_get_store_path_count(realised_string: *mut realised_string) -> usize;
}
unsafe extern "C" {
    #[doc = "Get a store path. The store paths are stored in an arbitrary order.\n\n# Arguments\n\n* `realised_string` [in]  -\n* `index` [in]  - index of the store path, must be less than the count\n\n# Returns\n\nstore path valid until realised_string is freed"]
    #[link_name = "\u{1}_nix_realised_string_get_store_path"]
    pub fn realised_string_get_store_path(
        realised_string: *mut realised_string,
        index: usize,
    ) -> *const StorePath;
}
unsafe extern "C" {
    #[doc = "Free a realised string\n\n# Arguments\n\n* `realised_string` [in]  -"]
    #[link_name = "\u{1}_nix_realised_string_free"]
    pub fn realised_string_free(realised_string: *mut realised_string);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct string_return {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct printer {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct string_context {
    _unused: [u8; 0],
}
unsafe extern "C" {
    #[doc = "Sets the contents of a nix_string_return\nCopies the passed string.\n\n# Arguments\n\n* `str` [out]  - the nix_string_return to write to\n* `c` [in]  -   The string to copy"]
    #[link_name = "\u{1}_nix_set_string_return"]
    pub fn set_string_return(str_: *mut string_return, c: *const ::core::ffi::c_char);
}
unsafe extern "C" {
    #[doc = "Print to the nix_printer\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `printer` [out]  - The nix_printer to print to\n* `str` [in]  - The string to print\n\n# Returns\n\nNIX_OK if everything worked"]
    #[link_name = "\u{1}_nix_external_print"]
    pub fn external_print(
        context: *mut c_context,
        printer: *mut printer,
        str_: *const ::core::ffi::c_char,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Add string context to the nix_string_context object\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `string_context` [out]  - The nix_string_context to add to\n* `c` [in]  - The context string to add\n\n# Returns\n\nNIX_OK if everything worked"]
    #[link_name = "\u{1}_nix_external_add_string_context"]
    pub fn external_add_string_context(
        context: *mut c_context,
        string_context: *mut string_context,
        c: *const ::core::ffi::c_char,
    ) -> err;
}
#[doc = "Definition for a class of external values\nCreate and implement one of these, then pass it to nix_create_external_value\nMake sure to keep it alive while the external value lives.\nOptional functions can be set to NULL\n\n# See also\n\n> [`nix_create_external_value`]"]
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct NixCExternalValueDesc {
    #[doc = "Called when printing the external value\n\n# Arguments\n\n* `self` [in]  - the void* passed to nix_create_external_value\n* `printer` [out]  - The printer to print to, pass to nix_external_print"]
    pub print: ::core::option::Option<
        unsafe extern "C" fn(self_: *mut ::core::ffi::c_void, printer: *mut printer),
    >,
    #[doc = "Called on :t\n\n# Arguments\n\n* `self` [in]  - the void* passed to nix_create_external_value\n* `res` [out]  - the return value"]
    pub showType: ::core::option::Option<
        unsafe extern "C" fn(self_: *mut ::core::ffi::c_void, res: *mut string_return),
    >,
    #[doc = "Called on `builtins.typeOf`\n\n# Arguments\n\n* `self` - the void* passed to nix_create_external_value\n* `res` [out]  - the return value"]
    pub typeOf: ::core::option::Option<
        unsafe extern "C" fn(self_: *mut ::core::ffi::c_void, res: *mut string_return),
    >,
    #[doc = "Called on \"${str}\" and builtins.toString.\nThe latter with coerceMore=true\nOptional, the default is to throw an error.\n\n# Arguments\n\n* `self` [in]  - the void* passed to nix_create_external_value\n* `c` [out]  - writable string context for the resulting string\n* `coerceMore` [in]  - boolean, try to coerce to strings in more cases\ninstead of throwing an error\n* `copyToStore` [in]  - boolean, whether to copy referenced paths to store\nor keep them as-is\n* `res` [out]  - the return value. Not touching this, or setting it to the\nempty string, will make the conversion throw an error."]
    pub coerceToString: ::core::option::Option<
        unsafe extern "C" fn(
            self_: *mut ::core::ffi::c_void,
            c: *mut string_context,
            coerceMore: ::core::ffi::c_int,
            copyToStore: ::core::ffi::c_int,
            res: *mut string_return,
        ),
    >,
    #[doc = "Try to compare two external values\nOptional, the default is always false.\nIf the other object was not a Nix C API external value, this comparison will\nalso return false\n\n# Arguments\n\n* `self` [in]  - the void* passed to nix_create_external_value\n* `other` [in]  - the void* passed to the other object's\nnix_create_external_value\n\n# Returns\n\ntrue if the objects are deemed to be equal"]
    pub equal: ::core::option::Option<
        unsafe extern "C" fn(
            self_: *mut ::core::ffi::c_void,
            other: *mut ::core::ffi::c_void,
        ) -> ::core::ffi::c_int,
    >,
    #[doc = "Convert the external value to json\nOptional, the default is to throw an error\n\n# Arguments\n\n* `self` [in]  - the void* passed to nix_create_external_value\n* `state` [in]  - The evaluator state\n* `strict` [in]  - boolean Whether to force the value before printing\n* `c` [out]  - writable string context for the resulting string\n* `copyToStore` [in]  - whether to copy referenced paths to store or keep\nthem as-is\n* `res` [out]  - the return value. Gets parsed as JSON. Not touching this,\nor setting it to the empty string, will make the conversion throw an error."]
    pub printValueAsJSON: ::core::option::Option<
        unsafe extern "C" fn(
            self_: *mut ::core::ffi::c_void,
            state: *mut EvalState,
            strict: bool,
            c: *mut string_context,
            copyToStore: bool,
            res: *mut string_return,
        ),
    >,
    #[doc = "Convert the external value to XML\nOptional, the default is to throw an error\n@todo The mechanisms for this call are incomplete. There are no C\nbindings to work with XML, pathsets and positions.\n\n# Arguments\n\n* `self` [in]  - the void* passed to nix_create_external_value\n* `state` [in]  - The evaluator state\n* `strict` [in]  - boolean Whether to force the value before printing\n* `location` [in]  - boolean Whether to include position information in the\nxml\n* `doc` [out]  - XML document to output to\n* `c` [out]  - writable string context for the resulting string\n* `drvsSeen` [in,out]  - a path set to avoid duplicating derivations\n* `pos` [in]  - The position of the call."]
    pub printValueAsXML: ::core::option::Option<
        unsafe extern "C" fn(
            self_: *mut ::core::ffi::c_void,
            state: *mut EvalState,
            strict: ::core::ffi::c_int,
            location: ::core::ffi::c_int,
            doc: *mut ::core::ffi::c_void,
            c: *mut string_context,
            drvsSeen: *mut ::core::ffi::c_void,
            pos: ::core::ffi::c_int,
        ),
    >,
}
#[allow(clippy::unnecessary_operation, clippy::identity_op)]
const _: () = {
    ["Size of NixCExternalValueDesc"][::core::mem::size_of::<NixCExternalValueDesc>() - 56usize];
    ["Alignment of NixCExternalValueDesc"]
        [::core::mem::align_of::<NixCExternalValueDesc>() - 8usize];
    ["Offset of field: NixCExternalValueDesc::print"]
        [::core::mem::offset_of!(NixCExternalValueDesc, print) - 0usize];
    ["Offset of field: NixCExternalValueDesc::showType"]
        [::core::mem::offset_of!(NixCExternalValueDesc, showType) - 8usize];
    ["Offset of field: NixCExternalValueDesc::typeOf"]
        [::core::mem::offset_of!(NixCExternalValueDesc, typeOf) - 16usize];
    ["Offset of field: NixCExternalValueDesc::coerceToString"]
        [::core::mem::offset_of!(NixCExternalValueDesc, coerceToString) - 24usize];
    ["Offset of field: NixCExternalValueDesc::equal"]
        [::core::mem::offset_of!(NixCExternalValueDesc, equal) - 32usize];
    ["Offset of field: NixCExternalValueDesc::printValueAsJSON"]
        [::core::mem::offset_of!(NixCExternalValueDesc, printValueAsJSON) - 40usize];
    ["Offset of field: NixCExternalValueDesc::printValueAsXML"]
        [::core::mem::offset_of!(NixCExternalValueDesc, printValueAsXML) - 48usize];
};
unsafe extern "C" {
    #[doc = "Create an external value, that can be given to nix_init_external\nCall nix_gc_decref() when you're done with the pointer.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `desc` [in]  - a NixCExternalValueDesc, you should keep this alive as long\nas the ExternalValue lives\n* `v` [in]  - the value to store\n\n# Returns\n\nexternal value, owned by the garbage collector\n\n# See also\n\n> [`nix_init_external`]"]
    #[link_name = "\u{1}_nix_create_external_value"]
    pub fn create_external_value(
        context: *mut c_context,
        desc: *mut NixCExternalValueDesc,
        v: *mut ::core::ffi::c_void,
    ) -> *mut ExternalValue;
}
unsafe extern "C" {
    #[doc = "Extract the pointer from a Nix C API external value.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `b` [in]  - The external value\n\n# Returns\n\nThe pointer, valid while the external value is valid, or null if the external value was not from the Nix C\nAPI.\n\n# See also\n\n> [`nix_get_external`]"]
    #[link_name = "\u{1}_nix_get_external_value_content"]
    pub fn get_external_value_content(
        context: *mut c_context,
        b: *mut ExternalValue,
    ) -> *mut ::core::ffi::c_void;
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct fetchers_settings {
    _unused: [u8; 0],
}
unsafe extern "C" {
    #[link_name = "\u{1}_nix_fetchers_settings_new"]
    pub fn fetchers_settings_new(context: *mut c_context) -> *mut fetchers_settings;
}
unsafe extern "C" {
    #[link_name = "\u{1}_nix_fetchers_settings_free"]
    pub fn fetchers_settings_free(settings: *mut fetchers_settings);
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flake_settings {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flake_reference_parse_flags {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flake_reference {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct flake_lock_flags {
    _unused: [u8; 0],
}
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct locked_flake {
    _unused: [u8; 0],
}
unsafe extern "C" {
    #[doc = "Create a nix_flake_settings initialized with default values.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n\n# Returns\n\nA new nix_flake_settings or NULL on failure.\n\n# See also\n\n> [`nix_flake_settings_free`]"]
    #[link_name = "\u{1}_nix_flake_settings_new"]
    pub fn flake_settings_new(context: *mut c_context) -> *mut flake_settings;
}
unsafe extern "C" {
    #[doc = "Release the resources associated with a nix_flake_settings."]
    #[link_name = "\u{1}_nix_flake_settings_free"]
    pub fn flake_settings_free(settings: *mut flake_settings);
}
unsafe extern "C" {
    #[doc = "Initialize a `nix_flake_settings` to contain `builtins.getFlake` and\npotentially more.\n@warning This does not put the eval state in pure mode!\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `settings` [in]  - The settings to use for e.g. `builtins.getFlake`\n* `builder` [in]  - The builder to modify"]
    #[link_name = "\u{1}_nix_flake_settings_add_to_eval_state_builder"]
    pub fn flake_settings_add_to_eval_state_builder(
        context: *mut c_context,
        settings: *mut flake_settings,
        builder: *mut eval_state_builder,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "A new `nix_flake_reference_parse_flags` with defaults"]
    #[link_name = "\u{1}_nix_flake_reference_parse_flags_new"]
    pub fn flake_reference_parse_flags_new(
        context: *mut c_context,
        settings: *mut flake_settings,
    ) -> *mut flake_reference_parse_flags;
}
unsafe extern "C" {
    #[doc = "Deallocate and release the resources associated with a `nix_flake_reference_parse_flags`.\nDoes not fail.\n\n# Arguments\n\n* `flags` [in]  - the `nix_flake_reference_parse_flags *` to free"]
    #[link_name = "\u{1}_nix_flake_reference_parse_flags_free"]
    pub fn flake_reference_parse_flags_free(flags: *mut flake_reference_parse_flags);
}
unsafe extern "C" {
    #[doc = "Provide a base directory for parsing relative flake references\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `flags` [in]  - The flags to modify\n* `baseDirectory` [in]  - The base directory to add\n* `baseDirectoryLen` [in]  - The length of baseDirectory\n\n# Returns\n\nNIX_OK on success, NIX_ERR on failure"]
    #[link_name = "\u{1}_nix_flake_reference_parse_flags_set_base_directory"]
    pub fn flake_reference_parse_flags_set_base_directory(
        context: *mut c_context,
        flags: *mut flake_reference_parse_flags,
        baseDirectory: *const ::core::ffi::c_char,
        baseDirectoryLen: usize,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "A new `nix_flake_lock_flags` with defaults\n\n# Arguments\n\n* `settings` [in]  - Flake settings that may affect the defaults"]
    #[link_name = "\u{1}_nix_flake_lock_flags_new"]
    pub fn flake_lock_flags_new(
        context: *mut c_context,
        settings: *mut flake_settings,
    ) -> *mut flake_lock_flags;
}
unsafe extern "C" {
    #[doc = "Deallocate and release the resources associated with a `nix_flake_lock_flags`.\nDoes not fail.\n\n# Arguments\n\n* `settings` [in]  - the `nix_flake_lock_flags *` to free"]
    #[link_name = "\u{1}_nix_flake_lock_flags_free"]
    pub fn flake_lock_flags_free(settings: *mut flake_lock_flags);
}
unsafe extern "C" {
    #[doc = "Put the lock flags in a mode that checks whether the lock is up to date.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `flags` [in]  - The flags to modify\n\n# Returns\n\nNIX_OK on success, NIX_ERR on failure\nThis causes `nix_flake_lock` to fail if the lock needs to be updated."]
    #[link_name = "\u{1}_nix_flake_lock_flags_set_mode_check"]
    pub fn flake_lock_flags_set_mode_check(
        context: *mut c_context,
        flags: *mut flake_lock_flags,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Put the lock flags in a mode that updates the lock file in memory, if needed.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `flags` [in]  - The flags to modify\n* `update` [in]  - Whether to allow updates\nThis will cause `nix_flake_lock` to update the lock file in memory, if needed."]
    #[link_name = "\u{1}_nix_flake_lock_flags_set_mode_virtual"]
    pub fn flake_lock_flags_set_mode_virtual(
        context: *mut c_context,
        flags: *mut flake_lock_flags,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Put the lock flags in a mode that updates the lock file on disk, if needed.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `flags` [in]  - The flags to modify\n* `update` [in]  - Whether to allow updates\nThis will cause `nix_flake_lock` to update the lock file on disk, if needed."]
    #[link_name = "\u{1}_nix_flake_lock_flags_set_mode_write_as_needed"]
    pub fn flake_lock_flags_set_mode_write_as_needed(
        context: *mut c_context,
        flags: *mut flake_lock_flags,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Add input overrides to the lock flags\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `flags` [in]  - The flags to modify\n* `inputPath` [in]  - The input path to override\n* `flakeRef` [in]  - The flake reference to use as the override\nThis switches the `flags` to `nix_flake_lock_flags_set_mode_virtual` if not in mode\n`nix_flake_lock_flags_set_mode_check`."]
    #[link_name = "\u{1}_nix_flake_lock_flags_add_input_override"]
    pub fn flake_lock_flags_add_input_override(
        context: *mut c_context,
        flags: *mut flake_lock_flags,
        inputPath: *const ::core::ffi::c_char,
        flakeRef: *mut flake_reference,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Lock a flake, if not already locked.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `settings` [in]  - The flake (and fetch) settings to use\n* `flags` [in]  - The locking flags to use\n* `flake` [in]  - The flake to lock"]
    #[link_name = "\u{1}_nix_flake_lock"]
    pub fn flake_lock(
        context: *mut c_context,
        fetchSettings: *mut fetchers_settings,
        settings: *mut flake_settings,
        eval_state: *mut EvalState,
        flags: *mut flake_lock_flags,
        flake: *mut flake_reference,
    ) -> *mut locked_flake;
}
unsafe extern "C" {
    #[doc = "Deallocate and release the resources associated with a `nix_locked_flake`.\nDoes not fail.\n\n# Arguments\n\n* `locked_flake` [in]  - the `nix_locked_flake *` to free"]
    #[link_name = "\u{1}_nix_locked_flake_free"]
    pub fn locked_flake_free(locked_flake: *mut locked_flake);
}
unsafe extern "C" {
    #[doc = "Parse a URL-like string into a `nix_flake_reference`.\n\n# Arguments\n\n* `context` [out]  - **context** – Optional, stores error information\n* `fetchSettings` [in]  - **context** – The fetch settings to use\n* `flakeSettings` [in]  - **context** – The flake settings to use\n* `parseFlags` [in]  - **context** – Specific context and parameters such as base directory\n* `str` [in]  - **input** – The URI-like string to parse\n* `strLen` [in]  - **input** – The length of `str`\n* `flakeReferenceOut` [out]  - **result** – The resulting flake reference\n* `fragmentCallback` [in]  - **result** – A callback to call with the fragment part of the URL\n* `fragmentCallbackUserData` [in]  - **result** – User data to pass to the fragment callback\n\n# Returns\n\nNIX_OK on success, NIX_ERR on failure"]
    #[link_name = "\u{1}_nix_flake_reference_and_fragment_from_string"]
    pub fn flake_reference_and_fragment_from_string(
        context: *mut c_context,
        fetchSettings: *mut fetchers_settings,
        flakeSettings: *mut flake_settings,
        parseFlags: *mut flake_reference_parse_flags,
        str_: *const ::core::ffi::c_char,
        strLen: usize,
        flakeReferenceOut: *mut *mut flake_reference,
        fragmentCallback: get_string_callback,
        fragmentCallbackUserData: *mut ::core::ffi::c_void,
    ) -> err;
}
unsafe extern "C" {
    #[doc = "Deallocate and release the resources associated with a `nix_flake_reference`.\nDoes not fail.\n\n# Arguments\n\n* `store` [in]  - the `nix_flake_reference *` to free"]
    #[link_name = "\u{1}_nix_flake_reference_free"]
    pub fn flake_reference_free(store: *mut flake_reference);
}
unsafe extern "C" {
    #[doc = "Get the output attributes of a flake.\n\n# Arguments\n\n* `context` [out]  - Optional, stores error information\n* `settings` [in]  - The settings to use\n* `locked_flake` [in]  - the flake to get the output attributes from\n\n# Returns\n\nA new nix_value or NULL on failure. Release the `nix_value` with `nix_value_decref`."]
    #[link_name = "\u{1}_nix_locked_flake_get_output_attrs"]
    pub fn locked_flake_get_output_attrs(
        context: *mut c_context,
        settings: *mut flake_settings,
        evalState: *mut EvalState,
        lockedFlake: *mut locked_flake,
    ) -> *mut value;
}
