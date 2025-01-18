pub mod core;

#[cfg(feature = "full")]
pub(crate) mod native;

#[cfg(not(feature = "full"))]
pub use core::*;

#[cfg(feature = "full")]
pub use native::*;
