//! Rust FFI bindings for creating Nix functions backed by Rust callbacks.

use core::ffi::{c_char, c_void};

use nixb_sys::{EvalState, Value, c_context, err};

/// TODO: docs.
pub type FunctionCallback = unsafe extern "C" fn(
    userdata: *mut c_void,
    context: *mut c_context,
    state: *mut EvalState,
    args: *mut *mut Value,
    ret: *mut Value,
);

unsafe extern "C" {
    /// Initialize a value as a Nix function backed by a Rust callback.
    ///
    /// Creates a lambda where:
    /// - The function name (for stack traces) is `name`
    /// - The body calls `callback` with the arguments when fully applied
    /// - `on_drop` is called when the function is garbage collected
    ///
    /// # Parameters
    ///
    /// - `context`: Error context for reporting initialization errors
    /// - `state`: EvalState providing symbols and base environment
    /// - `value`: Uninitialized Value to write the function into
    /// - `name`: Function name for stack traces (e.g., "buildPackage")
    /// - `name_len`: Length of `name` in bytes
    /// - `arity`: Number of arguments required to call the function
    /// - `args`: Array of argument names (length `arity`, must be non-null)
    /// - `userdata`: Opaque pointer passed to callbacks (typically `&'static dyn Fn`)
    /// - `callback`: Called each time the function is invoked
    /// - `on_drop`: Called when the function is GC'd (can be null for static userdata)
    ///
    /// # Callback Contract
    ///
    /// The callback receives:
    /// - `context`: Error context for reporting errors back to Nix
    /// - `state`: The EvalState from Nix's evaluation engine
    /// - `args`: Array of arguments passed by the caller (may contain thunks)
    /// - `result`: Uninitialized Value to write the result into
    /// - `userdata`: The same pointer passed to `init_function`
    ///
    /// Unlike thunk callbacks, this callback does NOT consume userdata.
    /// The function can be called multiple times with different arguments.
    ///
    /// # Cleanup
    ///
    /// - `on_drop` is called when the function is garbage collected
    /// - For static userdata (`&'static` references), pass null for `on_drop`
    /// - For allocated userdata, `on_drop` should free it
    pub fn init_function(
        context: *mut c_context,
        state: *mut EvalState,
        value: *mut Value,
        name: *const c_char,
        name_len: usize,
        arity: usize,
        args: *const *const c_char,
        userdata: *mut c_void,
        callback: FunctionCallback,
        on_drop: unsafe extern "C" fn(userdata: *mut c_void),
    ) -> err;
}
