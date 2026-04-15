//! TODO: docs.

#[cfg(feature = "expr")]
#[doc(inline)]
pub use nixb_expr as expr;
#[cfg(feature = "plugin")]
#[doc(inline)]
pub use nixb_plugin as plugin;
#[doc(inline)]
pub use nixb_result as result;

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
