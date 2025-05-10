//! System dependent structs and traits
//!
//! This module hosts system defined trait definitions
//! and related structs.
//!
//! The [`SystemClock`] and [`SystemTimeZone`] traits may be
//! implemented and provided to [`Now`] to provide it system
//! access.
//!
//! The struct implementations are feature gated by the `sys`
//! feature flag and include default implementations of the
//! above traits, [`DefaultSystemClock`] and [`DefaultSystemTimeZone`]
//! along with the [`Temporal`] namespace struct that provides
//! a const constructor for [`Now`] with the default trait
//! implementations.
//!
//! The traits in this module define the system methods
//! that must be implemented for system defined topics,

use crate::time::EpochNanoseconds;
use crate::TimeZone;
use core::fmt::Display;

#[cfg(feature = "sys")]
use crate::Now;
#[cfg(feature = "sys")]
use crate::TemporalError;
#[cfg(feature = "sys")]
use alloc::string::ToString;
#[cfg(feature = "sys")]
use web_time::{SystemTime as DefaultSysTime, UNIX_EPOCH};

pub trait SystemClock: Default {
    type Error: Display;
    fn get_system_epoch_nanoseconds(&self) -> Result<EpochNanoseconds, Self::Error>;
}

pub trait SystemTimeZone: Default {
    type Error: Display;
    fn get_system_time_zone(&self) -> Result<TimeZone, Self::Error>;
}

#[doc(inline)]
pub use crate::builtins::{EmptySystemClock, EmptySystemZone};

/// The Rust equivalent to the global `Temporal` object.
///
/// [`Temporal`] provides access to a default [`Now`] that is
/// implemented with [`web_time::SystemTime`] as the default
/// clock.
///
/// ```
/// use temporal_rs::{sys::Temporal, TimeZone};
///
/// let uschi = TimeZone::try_from_str("America/Chicago").unwrap();
/// let instant = Temporal::now().instant().unwrap();
/// let zoned_date_time = Temporal::now().zoned_date_time_iso(Some(uschi.clone())).unwrap();
/// let zoned_from_instant = instant.to_zoned_date_time_iso(uschi.clone());
/// assert_eq!(zoned_date_time.epoch_milliseconds(), zoned_from_instant.epoch_milliseconds());
/// ```
///
#[cfg(feature = "sys")]
pub struct Temporal;

#[cfg(feature = "sys")]
impl Temporal {
    /// Returns a [`Now`] using [`web_time::SystemTime`] as the clock and
    /// [`iana_time_zone`] as the system time zone.
    pub const fn now() -> Now<DefaultSystemClock, DefaultSystemTimeZone> {
        Now::new(DefaultSystemClock, DefaultSystemTimeZone)
    }
}

// ==== Utility functions ====

#[derive(Debug, Default)]
#[cfg(feature = "sys")]
pub struct DefaultSystemTimeZone;

#[cfg(feature = "sys")]
impl SystemTimeZone for DefaultSystemTimeZone {
    type Error = TemporalError;
    fn get_system_time_zone(&self) -> Result<TimeZone, Self::Error> {
        let id =
            iana_time_zone::get_timezone().map_err(|e| TemporalError::general(e.to_string()))?;
        TimeZone::try_from_str(&id)
    }
}

#[derive(Debug, Default)]
#[cfg(feature = "sys")]
pub struct DefaultSystemClock;

#[cfg(feature = "sys")]
impl SystemClock for DefaultSystemClock {
    type Error = TemporalError;
    fn get_system_epoch_nanoseconds(&self) -> Result<EpochNanoseconds, Self::Error> {
        DefaultSysTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .map_err(|e| TemporalError::general(e.to_string()))
            .map(EpochNanoseconds::try_from)?
    }
}
