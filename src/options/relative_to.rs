//! RelativeTo rounding option

#[cfg(not(feature = "full"))]
pub use crate::builtins::core::options::RelativeTo;

#[cfg(feature = "full")]
pub use crate::builtins::native::options::RelativeTo;
