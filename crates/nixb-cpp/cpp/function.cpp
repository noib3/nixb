#include "nix/expr/eval.hh"
#include "nix/expr/nixexpr.hh"
#include "nix_api_expr.h"
#include "nix_api_expr_internal.h"
#include "nix_api_util.h"
#include "nix_api_util_internal.h"

#ifdef NIX_2_34
using RustCallback = void (*)(void *, nix_c_context *, EvalState *,
                              nix_value **, nix_value *);

static nix_value *new_nix_value(nix::Value *value, nix::EvalMemory &mem) {
  return new (mem.allocBytes(sizeof(nix_value))) nix_value{
      .value = value,
      .mem = &mem,
  };
}
#else
using RustCallback = void (*)(void *, nix_c_context *, nix::EvalState *,
                              nix::Value **, nix::Value *);
#endif

/// Custom ExprLambda that calls a Rust callback when fully applied.
///
/// This struct extends ExprLambda and points `body` to itself. When the
/// function is called, `callFunction` invokes `body->eval()`, which dispatches
/// to our overridden `eval()` that either returns another lambda (when
/// partially applied) or calls the Rust callback (when fully applied).
///
/// Unlike thunks (which are forced once and cached), functions can be called
/// multiple times with different arguments. The callback does NOT consume
/// userdata - it remains valid for subsequent calls.
struct RustFunctionData {
  void *userdata;
  RustCallback callback;
  void (*on_drop)(void *);
  size_t arity;
  nix::Symbol func_name;
  nix::Symbol arg_names[nix::maxPrimOpArity];

  RustFunctionData(void *data, RustCallback cb, void (*drop)(void *),
                   size_t arity_, nix::Symbol func_name_,
                   const nix::Symbol *arg_names_)
      : userdata(data), callback(cb), on_drop(drop), arity(arity_),
        func_name(func_name_) {
    for (size_t i = 0; i < arity; ++i)
      arg_names[i] = arg_names_[i];
  }

  ~RustFunctionData() {
    if (on_drop) {
      on_drop(userdata);
    }
  }
};

struct ExprRustLambda : nix::ExprLambda {
  RustFunctionData *shared;
  size_t argc;
  nix::Value *args[nix::maxPrimOpArity];

  ExprRustLambda(RustFunctionData *shared_, size_t argc_,
                 const nix::Value *const *args_)
      : nix::ExprLambda(
#ifdef NIX_2_32
            nix::noPos, shared_->arg_names[argc_], nullptr, this
#else
            nix::noPos, shared_->arg_names[argc_], this
#endif
            ),
        shared(shared_), argc(argc_) {
    // Function name for stack traces.
    this->name = shared->func_name;
    // No structured formals - Rust handles destructuring.
    for (size_t i = 0; i < argc; ++i)
      args[i] = const_cast<nix::Value *>(args_[i]);
  }

  void eval(nix::EvalState &state, nix::Env &env, nix::Value &v) override {
    // Called by callFunction as body->eval()
    // The argument is bound at index 0 by callFunction
    nix::Value *arg = env.values[0];

    if (argc + 1 < shared->arity) {
      // Partial application: return a new lambda that captures the arg.
      nix::Value *new_args[nix::maxPrimOpArity];
      for (size_t i = 0; i < argc; ++i)
        new_args[i] = args[i];
      new_args[argc] = arg;

      auto *lambda = new
#if NIX_USE_BOEHMGC
          (GC)
#endif
              ExprRustLambda(shared, argc + 1, new_args);

      new (&v) nix::Value();
      v.mkLambda(&state.baseEnv, lambda);
      return;
    }

    nix_c_context ctx;
    ctx.last_err_code = NIX_OK;

    nix::Value *all_args[nix::maxPrimOpArity];
    for (size_t i = 0; i < argc; ++i)
      all_args[i] = args[i];
    all_args[argc] = arg;

#ifdef NIX_2_34
    nix::Value v_tmp;
    nix_value *external_args[nix::maxPrimOpArity];
    for (size_t i = 0; i <= argc; ++i)
      external_args[i] = new_nix_value(all_args[i], state.mem);

    nix_value *v_tmp_ptr = new_nix_value(&v_tmp, state.mem);
    EvalState wrapper{
        .state = state,
        .ownedFetchSettings = nullptr,
        .ownedSettings = nullptr,
        .ownedState = nullptr,
    };

    // Call the Rust callback using C API wrapper types.
    shared->callback(shared->userdata, &ctx, &wrapper, external_args,
                     v_tmp_ptr);
#else
    // Reset v for initialization (it may contain a blackhole)
    new (&v) nix::Value();

    // Call the Rust callback
    // NOTE: userdata is NOT consumed - function can be called multiple times
    shared->callback(shared->userdata, &ctx, &state, all_args, &v);
#endif

    if (ctx.last_err_code != NIX_OK) {
      const char *err_msg = "unknown error in function callback";
      if (ctx.last_err.has_value()) {
        err_msg = ctx.last_err->c_str();
      }
      throw nix::Error("%s", err_msg);
    }

#ifdef NIX_2_34
    if (!v_tmp.isValid()) {
      throw nix::Error("Implementation error in function callback: return "
                       "value was not initialized");
    }

    if (v_tmp.type() == nix::nThunk) {
      throw nix::Error("Implementation error in function callback: return "
                       "value must not be a thunk");
    }

    v = v_tmp;
#else
    if (!v.isValid()) {
      throw nix::Error("Implementation error in function callback: return "
                       "value was not initialized");
    }
#endif
  }

  void bindVars(nix::EvalState &,
                const std::shared_ptr<const nix::StaticEnv> &) override {}
};

extern "C" {

/// Initialize a value as a Nix function backed by a Rust callback.
///
/// Creates a lambda where:
/// - The function name (for stack traces) is `name`
/// - The body calls `callback` with the argument when invoked
/// - `on_drop` is called when the function is garbage collected
///
/// # Parameters
///
/// - `context`: Error context for reporting initialization errors
/// - `state`: EvalState providing symbols and base environment
/// - `value`: Uninitialized Value to write the function into
/// - `name`: Function name for stack traces (e.g., "buildPackage")
/// - `name_len`: Length of `name` in bytes
/// - `userdata`: Opaque pointer passed to callbacks (typically &'static dyn Fn)
/// - `callback`: Called each time the function is invoked (after full
/// application)
/// - `on_drop`: Called when the function is GC'd (can be nullptr for static
/// userdata)
///
/// # Callback Contract
///
/// The callback receives:
/// - `userdata`: The same pointer passed to init_function
/// - `context`: Error context for reporting errors back to Nix
/// - `state`: The EvalState from Nix's evaluation engine
/// - `args`: Array of arguments passed by the caller (may contain thunks)
/// - `result`: Uninitialized Value to write the result into
///
/// Unlike thunk callbacks, this callback does NOT consume userdata.
/// The function can be called multiple times with different arguments.
///
/// # Cleanup
///
/// - `on_drop` is called when the function is garbage collected
/// - For static userdata (&'static references), pass nullptr for on_drop
/// - For allocated userdata, on_drop should free it
nix_err init_function(nix_c_context *context,
#ifdef NIX_2_34
                      EvalState *state, nix_value *value,
#else
                      nix::EvalState *state, nix::Value *value,
#endif
                      const char *name, size_t name_len, size_t arity,
                      const char **args, void *userdata, RustCallback callback,
                      void (*on_drop)(void *)) {
  if (context)
    context->last_err_code = NIX_OK;

  try {
    if (!name)
      throw nix::Error("function name must not be null");
    if (arity == 0 || arity > nix::maxPrimOpArity)
      throw nix::Error("function arity must be between 1 and %d",
                       (int)nix::maxPrimOpArity);
    if (!args)
      throw nix::Error("function argument names array must not be null");

    nix::Symbol arg_syms[nix::maxPrimOpArity];
    for (size_t i = 0; i < arity; ++i) {
      if (!args[i])
        throw nix::Error("function argument name at index %d is null", (int)i);
#ifdef NIX_2_34
      arg_syms[i] = state->state.symbols.create(args[i]);
#else
      arg_syms[i] = state->symbols.create(args[i]);
#endif
    }

    auto *shared = new
#if NIX_USE_BOEHMGC
        (GC)
#endif
            RustFunctionData(
                userdata, callback, on_drop, arity,
#ifdef NIX_2_34
                state->state.symbols.create(std::string_view{name, name_len}),
#else
            state->symbols.create(std::string_view{name, name_len}),
#endif
                arg_syms);

    auto *lambda = new
#if NIX_USE_BOEHMGC
        (GC)
#endif
            ExprRustLambda(shared, 0, nullptr);

#ifdef NIX_2_34
    value->value->mkLambda(&state->state.baseEnv, lambda);
#else
    value->mkLambda(&state->baseEnv, lambda);
#endif
  }
  NIXC_CATCH_ERRS
}
}
