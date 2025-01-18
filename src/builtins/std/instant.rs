
use crate::{builtins::core, options::{DifferenceSettings, RoundingOptions}, time::EpochNanoseconds, TemporalError, TemporalResult};

use super::{duration::Duration, TimeDuration};

pub struct Instant(core::Instant);

impl From<core::Instant> for Instant {
    fn from(value: core::Instant) -> Self {
        Self(value)
    }
}

impl Instant {
    /// Create a new validated `Instant`.
    #[inline]
    pub fn try_new(nanoseconds: i128) -> TemporalResult<Self> {
        Ok(core::Instant::from(EpochNanoseconds::try_from(nanoseconds)?).into())
    }

    pub fn from_epoch_milliseconds(epoch_milliseconds: i128) -> TemporalResult<Self> {
        core::Instant::from_epoch_milliseconds(epoch_milliseconds).map(Into::into)
    }

    /// Adds a `Duration` to the current `Instant`, returning an error if the `Duration`
    /// contains a `DateDuration`.
    #[inline]
    pub fn add(&self, duration: Duration) -> TemporalResult<Self> {
        self.0.add(duration.0).map(Into::into)
    }

    /// Adds a `TimeDuration` to `Instant`.
    #[inline]
    pub fn add_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.0.add_time_duration(duration).map(Into::into)
    }

    /// Subtract a `Duration` to the current `Instant`, returning an error if the `Duration`
    /// contains a `DateDuration`.
    #[inline]
    pub fn subtract(&self, duration: Duration) -> TemporalResult<Self> {
        self.0.subtract(duration.0).map(Into::into)
    }

    /// Subtracts a `TimeDuration` to `Instant`.
    #[inline]
    pub fn subtract_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.0.subtract_time_duration(duration).map(Into::into)
    }

    /// Returns a `TimeDuration` representing the duration since provided `Instant`
    #[inline]
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.since(&other.0, settings).map(Into::into)
    }

    /// Returns a `TimeDuration` representing the duration until provided `Instant`
    #[inline]
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.until(&other.0, settings).map(Into::into)
    }

    /// Returns an `Instant` by rounding the current `Instant` according to the provided settings.
    pub fn round(&self, options: RoundingOptions) -> TemporalResult<Self> {
        self.0.round(options).map(Into::into)
    }

    /// Returns the `epochMilliseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_milliseconds(&self) -> i64 {
        self.0.epoch_milliseconds()
    }

    /// Returns the `epochNanoseconds` value for this `Instant`.
    #[must_use]
    pub fn epoch_nanoseconds(&self) -> i128 {
        self.0.epoch_nanoseconds()
    }
}
