use crate::{
    builtins::core as temporal_core,
    options::{DifferenceSettings, RoundingOptions, ToStringRoundingOptions},
    time::EpochNanoseconds,
    TemporalError, TemporalResult, TimeZone,
};
use alloc::string::String;

use super::{duration::Duration, timezone::TZ_PROVIDER, TimeDuration};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Instant(temporal_core::Instant);

impl From<temporal_core::Instant> for Instant {
    fn from(value: temporal_core::Instant) -> Self {
        Self(value)
    }
}

impl core::fmt::Display for Instant {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(
            &self
                .as_ixdtf_string(None, ToStringRoundingOptions::default())
                .expect("A valid instant string."),
        )
    }
}

impl Instant {
    /// Create a new validated `Instant`.
    #[inline]
    pub fn try_new(nanoseconds: i128) -> TemporalResult<Self> {
        Ok(temporal_core::Instant::from(EpochNanoseconds::try_from(nanoseconds)?).into())
    }

    /// Creates a new `Instant` from the provided Epoch Millisecond value.
    pub fn from_epoch_milliseconds(epoch_milliseconds: i128) -> TemporalResult<Self> {
        temporal_core::Instant::from_epoch_milliseconds(epoch_milliseconds).map(Into::into)
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

    /// Returns the RFC9557 (IXDTF) string for this `Instant` with the
    /// provided options
    pub fn as_ixdtf_string(
        &self,
        timezone: Option<&TimeZone>,
        options: ToStringRoundingOptions,
    ) -> TemporalResult<String> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;

        self.0
            .as_ixdtf_string_with_provider(timezone, options, &*provider)
    }
}

impl core::str::FromStr for Instant {
    type Err = TemporalError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        temporal_core::Instant::from_str(s).map(Into::into)
    }
}
