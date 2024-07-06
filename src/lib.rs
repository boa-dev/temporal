//! The `Temporal` crate is an implementation of ECMAScript's Temporal buitl-in.
//!
//! The crate is being designed with both engine and general use in mind.
//!
//! IMPORTANT NOTE: Please note that this library is actively being developed and is very
//! much experimental and NOT STABLE.
//!
//! [`Temporal`][proposal] is the Stage 3 proposal for ECMAScript that provides new JS objects and functions
//! for working with dates and times that fully supports time zones and non-gregorian calendars.
//!
//! This library's primary source is the Temporal Proposal [specification][spec].
//!
//! [proposal]: https://github.com/tc39/proposal-temporal
//! [spec]: https://tc39.es/proposal-temporal/
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/boa-dev/boa/main/assets/logo.svg"
)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    // Currently throws a false positive regarding dependencies that are only used in benchmarks.
    unused_crate_dependencies,
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::missing_errors_doc,
    clippy::let_unit_value,
    clippy::option_if_let_else,

    // It may be worth to look if we can fix the issues highlighted by these lints.
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_wrap,

    // Add temporarily - Needs addressing
    clippy::missing_panics_doc,
)]

pub mod components;
pub mod error;
pub mod fields;
pub mod iso;
pub mod options;
pub mod parsers;

#[doc(hidden)]
pub(crate) mod rounding;
#[doc(hidden)]
pub(crate) mod utils;

use std::cmp::Ordering;

// TODO: evaluate positives and negatives of using tinystr.
// Re-exporting tinystr as a convenience, as it is currently tied into the API.
pub use tinystr::TinyAsciiStr;

#[doc(inline)]
pub use error::TemporalError;
#[doc(inline)]
pub use fields::TemporalFields;

/// The `Temporal` result type
pub type TemporalResult<T> = Result<T, TemporalError>;

/// A library specific trait for unwrapping assertions.
pub(crate) trait TemporalUnwrap {
    type Output;

    /// `temporal_rs` based assertion for unwrapping. This will panic in debug
    /// builds, but throws error during runtime.
    fn temporal_unwrap(self) -> TemporalResult<Self::Output>;
}

impl<T> TemporalUnwrap for Option<T> {
    type Output = T;

    fn temporal_unwrap(self) -> TemporalResult<Self::Output> {
        debug_assert!(self.is_some());
        self.ok_or(TemporalError::assert())
    }
}

#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Sign {
    Positive = 1,
    Zero = 0,
    Negative = -1,
}

impl From<i8> for Sign {
    fn from(value: i8) -> Self {
        match value.cmp(&0) {
            Ordering::Greater => Self::Positive,
            Ordering::Equal => Self::Zero,
            Ordering::Less => Self::Negative,
        }
    }
}

impl Sign {
    /// Coerces the current `Sign` to be either negative or positive.
    pub(crate) fn as_non_zero(&self) -> Self {
        if matches!(self, Self::Zero) {
            return Self::Positive;
        }
        *self
    }
}

// Relevant numeric constants
/// Nanoseconds per day constant: 8.64e+13
pub const NS_PER_DAY: u64 = MS_PER_DAY as u64 * 1_000_000;
/// Milliseconds per day constant: 8.64e+7
pub const MS_PER_DAY: u32 = 24 * 60 * 60 * 1000;
/// Max Instant nanosecond constant
#[doc(hidden)]
pub(crate) const NS_MAX_INSTANT: i128 = NS_PER_DAY as i128 * 100_000_000i128;
/// Min Instant nanosecond constant
#[doc(hidden)]
pub(crate) const NS_MIN_INSTANT: i128 = -NS_MAX_INSTANT;
