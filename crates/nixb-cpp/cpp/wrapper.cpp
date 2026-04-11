#include "nix/expr/eval.hh"
#include "nix/expr/primops.hh"
#include "nix_api_expr.h"
#include "nix_api_expr_internal.h"
#include "nix_api_store_internal.h"
#include "nix_api_util_internal.h"
#include "nix_api_value.h"

#ifdef NIX_2_34
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
#endif

// Attrsets.

#ifndef NIX_2_34
extern "C" nix::BindingsBuilder *make_bindings_builder(nix::EvalState *state,
                                                       size_t capacity) {
  // buildBindings returns by value, so we allocate on heap.
  auto *builder = new nix::BindingsBuilder(state->buildBindings(capacity));
  return builder;
}

extern "C" void bindings_builder_insert(nix::BindingsBuilder *builder,
                                        const char *name, nix::Value *value) {
  nix::Symbol sym = builder->symbols.get().create(name);
  builder->insert(sym, value);
}

extern "C" void make_attrs(nix::Value *v, nix::BindingsBuilder *builder) {
  v->mkAttrs(*builder);
  delete builder;
}
#endif

extern "C"
#ifdef NIX_2_34
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
#else
    nix::Value *get_attr_byname_lazy_no_incref(const nix::Value *value,
                                               nix::EvalState *state,
                                               const char *name) {
  nix::Symbol sym = state->symbols.create(name);
  const nix::Attr *attr = value->attrs()->get(sym);
  if (!attr) {
    return nullptr;
  }
  return attr->value;
}
#endif

// Attrset iterator.

struct AttrIterator {
  nix::Bindings::const_iterator current;
  const nix::SymbolTable *symbols;
#ifdef NIX_2_34
  nix::EvalMemory *mem;
#endif
};

extern "C" AttrIterator *attr_iter_create(
#ifdef NIX_2_34
    const nix_value *value, EvalState *state
#else
    const nix::Value *value, nix::EvalState *state
#endif
) {
#ifdef NIX_2_34
  const nix::Bindings *bindings = check_value_not_null(value).attrs();
#else
  const nix::Bindings *bindings = value->attrs();
#endif
  return new AttrIterator{bindings->begin(),
#ifdef NIX_2_34
                          &state->state.symbols, &state->state.mem
#else
                          &state->symbols
#endif
  };
}

extern "C" const char *attr_iter_key(const AttrIterator *iter) {
  return (*iter->symbols)[iter->current->name].c_str();
}

extern "C"
#ifdef NIX_2_34
    nix_value *
#else
    nix::Value *
#endif
    attr_iter_value(const AttrIterator *iter) {
#ifdef NIX_2_34
  return borrow_nix_value(iter->current->value, *iter->mem);
#else
  return iter->current->value;
#endif
}

extern "C" void attr_iter_advance(AttrIterator *iter) { ++iter->current; }

extern "C" void attr_iter_destroy(AttrIterator *iter) { delete iter; }

// Builtins.

extern "C"
#ifdef NIX_2_34
    nix_value *get_builtins(EvalState *state) {
  return borrow_nix_value(state->state.baseEnv.values[0], state->state.mem);
}
#else
    nix::Value *get_builtins(nix::EvalState *state) {
  // builtins is the first value in baseEnv
  return state->baseEnv.values[0];
}
#endif

// Expression evaluation.

#ifndef NIX_2_34
extern "C" nix_err expr_eval_from_string(nix_c_context *context,
                                         nix::EvalState *state,
                                         const char *expr, const char *path,
                                         nix::Value *value) {
  if (context)
    context->last_err_code = NIX_OK;
  try {
    nix::Expr *parsedExpr =
        state->parseExprFromString(expr, state->rootPath(nix::CanonPath(path)));
    state->eval(parsedExpr, *value);
    state->forceValue(*value, nix::noPos);
  }
  NIXC_CATCH_ERRS
}
#endif

// Lists.

#ifndef NIX_2_34
extern "C" nix::ListBuilder *make_list_builder(nix::EvalState *state,
                                               size_t size) {
  auto *builder = new nix::ListBuilder(state->buildList(size));
  return builder;
}

extern "C" void list_builder_insert(nix::ListBuilder *builder, size_t index,
                                    nix::Value *value) {
  (*builder)[index] = value;
}

extern "C" void make_list(nix::Value *v, nix::ListBuilder *builder) {
  v->mkList(*builder);
  delete builder;
}
#endif

extern "C"
#ifdef NIX_2_34
    nix_value *get_list_byidx_lazy_no_incref(const nix_value *value,
                                             unsigned int ix) {
  return borrow_nix_value(check_value_not_null(value).listView()[ix],
                          *value->mem);
}
#else
    nix::Value *get_list_byidx_lazy_no_incref(const nix::Value *value,
                                              unsigned int ix) {
  return value->listView()[ix];
}
#endif

// String realization (IFD).

#ifndef NIX_2_34
extern "C" nix_realised_string *string_realise(nix_c_context *context,
                                               nix::EvalState *state,
                                               nix::Value *value, bool isIFD) {
  if (context)
    context->last_err_code = NIX_OK;
  try {
    nix::StorePathSet storePaths;
    auto s = state->realiseString(*value, &storePaths, isIFD);

    std::vector<StorePath> vec;
    for (auto &sp : storePaths) {
      vec.push_back(StorePath{sp});
    }

    return new nix_realised_string{.str = s, .storePaths = vec};
  }
  NIXC_CATCH_ERRS_NULL
}
#endif

// Values.

#ifndef NIX_2_34
extern "C" nix::Value *alloc_value(nix::EvalState *state) {
  nix::Value *res = state->allocValue();
  nix_gc_incref(nullptr, res);
  return res;
}

extern "C" nix_err force_value(nix_c_context *context, nix::EvalState *state,
                               nix::Value *value) {
  if (context)
    context->last_err_code = NIX_OK;
  try {
    state->forceValue(*value, nix::noPos);
  }
  NIXC_CATCH_ERRS
}

extern "C" void init_path_string(nix::EvalState *state, nix::Value *value,
                                 const char *str) {
#ifdef NIX_2_32
  value->mkPath(state->rootPath(nix::CanonPath(str)));
#else
  value->mkPath(state->rootPath(nix::CanonPath(str)), state->mem);
#endif
}

extern "C" nix_err value_call_multi(nix_c_context *context,
                                    nix::EvalState *state, nix::Value *fn,
                                    size_t nargs, nix::Value **args,
                                    nix::Value *result) {
  if (context)
    context->last_err_code = NIX_OK;
  try {
    state->callFunction(*fn, {args, nargs}, *result, nix::noPos);
    state->forceValue(*result, nix::noPos);
  }
  NIXC_CATCH_ERRS
}
#endif

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
#ifdef NIX_2_34
                              EvalState *state, nix_value *value,
#else
                              nix::EvalState *state, nix::Value *value,
#endif
                              void *userdata,
#ifdef NIX_2_34
                              void (*on_force_once)(nix_c_context *,
                                                    EvalState *, nix_value *,
                                                    void *),
#else
                              void (*on_force_once)(nix_c_context *,
                                                    nix::EvalState *,
                                                    nix::Value *, void *),
#endif
                              void (*on_drop)(void *)) {
  // Custom Expr subclass that invokes a Rust callback when evaluated.
  // Note: This is defined outside the try block since struct definitions cannot
  // throw.
  struct ExprRustCallback : nix::Expr {
    void *userdata;
#ifdef NIX_2_34
    void (*on_force_once)(nix_c_context *, EvalState *, nix_value *, void *);
#else
    void (*on_force_once)(nix_c_context *, nix::EvalState *, nix::Value *,
                          void *);
#endif
    void (*on_drop)(void *);
    bool is_evaluating = false;

    ExprRustCallback(void *data,
#ifdef NIX_2_34
                     void (*callback)(nix_c_context *, EvalState *, nix_value *,
                                      void *),
#else
                     void (*callback)(nix_c_context *, nix::EvalState *,
                                      nix::Value *, void *),
#endif
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
#ifdef NIX_2_34
      EvalState wrapper{
          .state = state,
          .ownedFetchSettings = nullptr,
          .ownedSettings = nullptr,
          .ownedState = nullptr,
      };
      nix_value wrapped_value{.value = &v, .mem = &state.mem};
      on_force_once(&ctx, &wrapper, &wrapped_value, userdata);
#else
      on_force_once(&ctx, &state, &v, userdata);
#endif

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

  // Only mkThunk can throw, so we keep the try block minimal
  try {
    auto *expr = new ExprRustCallback(userdata, on_force_once, on_drop);
#ifdef NIX_2_34
    value->value->mkThunk(&state->state.baseEnv, expr);
#else
    value->mkThunk(&state->baseEnv, expr);
#endif
  }
  NIXC_CATCH_ERRS
}
