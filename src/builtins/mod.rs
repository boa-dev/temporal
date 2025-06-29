//! The builtins module contains the main implementation of the Temporal builtins

#[cfg(feature = "compiled_data")]
pub mod compiled;
pub mod core;

pub use core::*;

#[cfg(feature = "compiled_data")]
use crate::tzdb::CompiledTzdbProvider;
#[cfg(all(test, feature = "compiled_data"))]
use crate::tzdb::FsTzdbProvider;
#[cfg(feature = "compiled_data")]
use std::sync::LazyLock;

#[cfg(feature = "compiled_data")]
pub static TZ_PROVIDER: LazyLock<CompiledTzdbProvider> =
    LazyLock::new(CompiledTzdbProvider::default);

#[cfg(all(test, feature = "compiled_data"))]
pub(crate) static FS_TZ_PROVIDER: LazyLock<FsTzdbProvider> = LazyLock::new(FsTzdbProvider::default);
