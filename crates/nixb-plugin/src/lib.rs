//! TODO: docs.

#![allow(clippy::undocumented_unsafe_blocks)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(nightly, feature(const_type_name))]
#![cfg_attr(nightly, feature(generic_const_exprs))]

extern crate alloc;

#[cfg(feature = "dlopen")]
mod dlopen;
mod entry;
pub mod primop;
pub use nixb_macros::{PrimOp, plugin};

#[doc(hidden)]
pub use entry::entry;

pub mod prelude {
    //! TODO: docs.

    #[cfg(feature = "expr")]
    mod expr {
        pub use nixb_expr::Utf8CStr;
        pub use nixb_expr::attrset::*;
        pub use nixb_expr::callable::*;
        pub use nixb_expr::context::*;
        pub use nixb_expr::error::*;
        pub use nixb_expr::function::*;
        pub use nixb_expr::list::*;
        pub use nixb_expr::primop::*;
        pub use nixb_expr::thunk::*;
        pub use nixb_expr::value::*;
    }

    #[cfg(feature = "expr")]
    pub use expr::*;
}
