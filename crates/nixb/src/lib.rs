//! TODO: docs.

#[doc(inline)]
pub use nixb_c_context::CContext;
#[doc(inline)]
pub use nixb_error::{Error, ErrorKind, Result};
#[cfg(feature = "expr")]
#[doc(inline)]
pub use nixb_expr as expr;
#[cfg(feature = "plugin")]
#[doc(inline)]
pub use nixb_plugin as plugin;
#[cfg(feature = "store")]
#[doc(inline)]
pub use nixb_store as store;

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
        pub use nixb_expr::thunk::*;
        pub use nixb_expr::value::*;
    }

    #[cfg(feature = "expr")]
    pub use expr::*;
}
