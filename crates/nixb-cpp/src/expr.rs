use core::ffi::{c_char, c_uint, c_void};

use nixb_sys::{EvalState, Value, c_context, err};

/// Opaque type representing an attribute set iterator.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct AttrIterator {
    _unused: [u8; 0],
}

// Attrsets.
unsafe extern "C" {
    /// Get an attribute by name from an attribute set without forcing it.
    ///
    /// This is very similar to `nix_get_attr_byname_lazy`, except that it
    /// doesn't increment the reference count of the returned value and it
    /// doesn't do any validation on the `value` parameter.
    ///
    /// Returns a pointer to the attribute's value if found, or null if the
    /// attribute doesn't exist.
    pub fn get_attr_byname_lazy_no_incref(
        value: *const Value,
        state: *mut EvalState,
        name: *const c_char,
    ) -> *mut Value;
}

// Attribute set iterator.
unsafe extern "C" {
    /// Creates an iterator over an attribute set.
    ///
    /// Call [`attr_iter_destroy`] to free the iterator when done.
    pub fn attr_iter_create(
        value: *const Value,
        state: *mut EvalState,
    ) -> *mut AttrIterator;

    /// Gets the key name of the current attribute.
    ///
    /// # Safety
    ///
    /// The iterator must not have been advanced past the end.
    pub fn attr_iter_key(iter: *const AttrIterator) -> *const c_char;

    /// Gets the value of the current attribute.
    ///
    /// # Safety
    ///
    /// The iterator must not have been advanced past the end.
    pub fn attr_iter_value(iter: *const AttrIterator) -> *mut Value;

    /// Advances the iterator to the next attribute.
    ///
    /// # Safety
    ///
    /// The caller is responsible for ensuring the iterator is not advanced
    /// past the end by checking the length of the attribute set.
    pub fn attr_iter_advance(iter: *mut AttrIterator);

    /// Destroys the iterator and free its memory.
    pub fn attr_iter_destroy(iter: *mut AttrIterator);
}

// Builtins.
unsafe extern "C" {
    /// Get the global `builtins` attribute set.
    ///
    /// Returns a pointer to the `builtins` attrset that contains all built-in
    /// functions like `fetchGit`, `fetchurl`, `toString`, etc.
    ///
    /// The returned pointer is valid as long as the `EvalState` is alive.
    /// It does not need to be freed (it's managed by the EvalState).
    pub fn get_builtins(state: *mut EvalState) -> *mut Value;
}

// Lists.
unsafe extern "C" {
    /// Get an element by index from a list without forcing it.
    ///
    /// This is very similar to `nix_get_list_byidx_lazy`, except that it
    /// doesn't increment the reference count of the returned value and it
    /// doesn't do any validation on the `value` parameter or any bounds
    /// checking on the index.
    pub fn get_list_byidx_lazy_no_incref(
        value: *const Value,
        idx: c_uint,
    ) -> *mut Value;
}

// Values.
unsafe extern "C" {
    /// Initialize a value as a thunk that executes a callback when forced.
    ///
    /// The thunk uses a custom Expr subclass that invokes the provided callback
    /// when Nix's evaluation engine forces the value. The callback is guaranteed
    /// to be called at most once - Nix automatically caches the result by having
    /// the callback overwrite the thunk value in-place.
    ///
    /// # Parameters
    ///
    /// - `context`: Error context for reporting initialization errors
    /// - `state`: EvalState providing the environment for the thunk
    /// - `value`: Uninitialized Value to write the thunk into
    /// - `userdata`: Opaque pointer passed through to callbacks (typically a boxed closure)
    /// - `on_force_once`: Callback invoked when the thunk is forced (called at most once)
    /// - `on_drop`: Cleanup callback invoked when the Expr is destroyed by GC
    ///
    /// # Callback Contracts
    ///
    /// ## `on_force_once` - Evaluation Callback
    ///
    /// When the thunk is forced, this callback receives:
    /// - `context`: Error context for reporting evaluation errors back to C++
    /// - `state`: The EvalState from Nix's evaluation engine
    /// - `value`: The Value to overwrite with the computed result (initially a blackhole)
    /// - `userdata`: The same opaque pointer passed to `init_thunk`
    ///
    /// The callback MUST:
    /// 1. Compute the result value
    /// 2. Overwrite the `value` parameter with the result (using `nix_init_*` functions)
    /// 3. Set `context->last_err_code` if an error occurs
    ///
    /// The callback will be called exactly once if the thunk is forced, or never
    /// if the thunk is never accessed.
    ///
    /// ## `on_drop` - Cleanup Callback (only if never forced)
    ///
    /// This callback is called **if and only if** the thunk is never forced.
    ///
    /// The callback receives:
    /// - `userdata`: The same opaque pointer passed to `init_thunk`
    ///
    /// The callback should free any resources associated with `userdata` (e.g., `Box::from_raw`).
    ///
    /// ### When is it called?
    ///
    /// **Path 1: Thunk is forced (success or error)**
    /// - `on_force_once` is called and consumes the userdata
    /// - `on_drop` is **NOT** called
    /// - `on_force_once` is responsible for cleaning up the userdata
    ///
    /// **Path 2: Thunk is never forced**
    /// - Eventually, Boehm GC collects the Expr
    /// - The destructor runs and calls `on_drop`
    ///
    /// ### Important Notes
    ///
    /// - **`on_force_once` owns the userdata**: When a thunk is forced, `on_force_once`
    ///   receives ownership of the userdata and must clean it up (e.g., `Box::from_raw`).
    /// - **`on_drop` is for unforced thunks only**: It handles cleanup when the thunk
    ///   is garbage collected without ever being forced.
    /// - **Best-effort cleanup**: GC destructors may not run if the program exits before
    ///   a collection cycle.
    pub fn init_thunk(
        context: *mut c_context,
        state: *mut EvalState,
        value: *mut Value,
        userdata: *mut c_void,
        on_force_once: unsafe extern "C" fn(
            context: *mut c_context,
            state: *mut EvalState,
            value: *mut Value,
            userdata: *mut c_void,
        ),
        on_drop: unsafe extern "C" fn(userdata: *mut c_void),
    ) -> err;
}
