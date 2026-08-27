#include "nix/expr/eval.hh"
#include "nix/expr/primops.hh"
#include "nix_api_expr.h"
#include "nix_api_expr_internal.h"
#include "nix_api_store_internal.h"
#include "nix_api_util_internal.h"
#include "nix_api_value.h"

static const nix::Value &check_value_not_null(const nix_value *value) {
  if (!value) {
    throw std::runtime_error("nix_value is null");
  }
  return *value->value;
}

static nix_value *borrow_nix_value(nix::Value *value, nix::EvalMemory &mem) {
  return new (mem.allocBytes(sizeof(nix_value))) nix_value{
      .value = value,
      .mem = &mem,
  };
}

// Attrsets.

extern "C"
    nix_value *get_attr_byname_lazy_no_incref(const nix_value *value,
                                              EvalState *state,
                                              const char *name) {
  nix::Symbol sym = state->state.symbols.create(name);
  const nix::Attr *attr = check_value_not_null(value).attrs()->get(sym);
  if (!attr) {
    return nullptr;
  }
  return borrow_nix_value(attr->value, state->state.mem);
}

// Attrset iterator.

struct AttrIterator {
  nix::Bindings::const_iterator current;
  const nix::SymbolTable *symbols;
  nix::EvalMemory *mem;
};

extern "C" AttrIterator *attr_iter_create(
    const nix_value *value, EvalState *state
) {
  const nix::Bindings *bindings = check_value_not_null(value).attrs();
  return new AttrIterator{bindings->begin(),
                          &state->state.symbols, &state->state.mem
  };
}

extern "C" const char *attr_iter_key(const AttrIterator *iter) {
  return (*iter->symbols)[iter->current->name].c_str();
}

extern "C"
    nix_value *
    attr_iter_value(const AttrIterator *iter) {
  return borrow_nix_value(iter->current->value, *iter->mem);
}

extern "C" void attr_iter_advance(AttrIterator *iter) { ++iter->current; }

extern "C" void attr_iter_destroy(AttrIterator *iter) { delete iter; }

// Builtins.

extern "C"
    nix_value *get_builtins(EvalState *state) {
  return borrow_nix_value(state->state.baseEnv.values[0], state->state.mem);
}

// Expression evaluation.

// Lists.

extern "C"
    nix_value *get_list_byidx_lazy_no_incref(const nix_value *value,
                                             unsigned int ix) {
  return borrow_nix_value(check_value_not_null(value).listView()[ix],
                          *value->mem);
}

// String realization (IFD).

// Values.

// Thunk lifecycle and cleanup guarantees:
//
// on_drop is called exactly once IF AND ONLY IF the thunk is never forced:
//
// 1. Thunk forced (success or error):
//    on_force_once() consumes userdata -> userdata=null
//    (on_drop is NOT called - on_force_once is responsible for cleanup)
//
// 2. Thunk never forced:
//    (GC runs) -> ~ExprRustCallback() -> on_drop()
//
// The userdata=null assignment after on_force_once prevents on_drop from
// being called in the destructor for forced thunks.
extern "C" nix_err init_thunk(nix_c_context *context,
                              EvalState *state, nix_value *value,
                              void *userdata,
                              void (*on_force_once)(nix_c_context *,
                                                    EvalState *, nix_value *,
                                                    void *),
                              void (*on_drop)(void *)) {
  // Custom Expr subclass that invokes a Rust callback when evaluated.
  // Note: This is defined outside the try block since struct definitions cannot
  // throw.
  struct ExprRustCallback : nix::Expr {
    void *userdata;
    void (*on_force_once)(nix_c_context *, EvalState *, nix_value *, void *);
    void (*on_drop)(void *);
    bool is_evaluating = false;

    ExprRustCallback(void *data,
                     void (*callback)(nix_c_context *, EvalState *, nix_value *,
                                      void *),
                     void (*drop)(void *))
        : userdata(data), on_force_once(callback), on_drop(drop) {
    }

    // Destructor: called by Boehm GC when this Expr is collected.
    // Note: GC destructors are not guaranteed to run, but when they do,
    // this gives us a chance to clean up the Rust userdata.
    ~ExprRustCallback() override {
      if (on_drop && userdata) {
        on_drop(userdata);
      }
    }

    // Called by Nix's forceValue() exactly once.
    // The callback must overwrite `v` with the computed result.
    void eval(nix::EvalState &state, nix::Env &, nix::Value &v) override {
      // RAII guard to reset is_evaluating on scope exit (even during
      // exceptions)
      struct EvaluatingGuard {
        bool &flag;
        EvaluatingGuard(bool &f) : flag(f) { flag = true; }
        ~EvaluatingGuard() { flag = false; }
      };

      // Detect infinite recursion (same thunk forced while already evaluating)
      if (is_evaluating) {
        nix::ExprBlackHole::throwInfiniteRecursionError(state, v);
      }
      EvaluatingGuard guard(is_evaluating);

      nix_c_context ctx;
      ctx.last_err_code = NIX_OK;

      // Invoke the Rust callback.
      // At this point, `v` contains a blackhole value (set by forceValue).
      // We need to mark it as uninitialized so Rust can use the nix_init_*
      // functions (which error on already-initialized values).
      // The C++ mk* methods don't care, but the C API does.
      // Use placement new to reconstruct the value as uninitialized.
      new (&v) nix::Value();

      // The callback is expected to initialize v with the actual result.
      EvalState wrapper{
          .state = state,
          .ownedFetchSettings = nullptr,
          .ownedSettings = nullptr,
          .ownedState = nullptr,
      };
      nix_value wrapped_value{.value = &v, .mem = &state.mem};
      on_force_once(&ctx, &wrapper, &wrapped_value, userdata);

      // on_force_once has consumed the userdata - set to null to prevent
      // the destructor from calling on_drop.
      userdata = nullptr;

      // Check for errors reported by Rust
      if (ctx.last_err_code != NIX_OK) {
        // Extract the error message from the callback's context
        const char *err_msg = "unknown error in lazy evaluation callback";
        if (ctx.last_err.has_value()) {
          err_msg = ctx.last_err->c_str();
        }
        // Throw a Nix error with the message from Rust.
        // This will be caught by the outer evaluation machinery and
        // reported to the user in the same way as any Nix exception.
        throw nix::Error("%s", err_msg);
      }
    }

    // Required virtual method - no variable binding for external callbacks
    void bindVars(nix::EvalState &,
                  const std::shared_ptr<const nix::StaticEnv> &) override {}
  };

  if (context)
    context->last_err_code = NIX_OK;

  // Only allocating ExprRustCallback can throw; mkThunk is noexcept.
  try {
    auto *expr = new ExprRustCallback(userdata, on_force_once, on_drop);
    value->value->mkThunk(&state->state.baseEnv, expr);
  }
  NIXC_CATCH_ERRS
}
