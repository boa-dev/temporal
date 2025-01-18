//! RelativeTo rounding option

#[cfg(not(feature = "experimental"))]
pub use crate::builtins::core::options::RelativeTo;


#[cfg(feature = "experimental")]
pub use crate::builtins::std::options::RelativeTo;
