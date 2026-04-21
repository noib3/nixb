use nixb_sys::Store;

unsafe extern "C" {
    /// Clone a store handle. Returns null on allocation failure.
    pub fn store_clone(store: *const Store) -> *mut Store;
}
