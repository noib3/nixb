#include "nix_api_store_internal.h"

extern "C" Store *store_clone(const Store *store) {
  if (store == nullptr) {
    return nullptr;
  }

  try {
    return new Store{store->ptr};
  } catch (...) {
    return nullptr;
  }
}
