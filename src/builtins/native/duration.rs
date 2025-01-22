use crate::builtins::core::PartialDuration;
use crate::options::ToStringRoundingOptions;
use crate::{
    builtins::core as temporal_core,
    options::{RelativeTo, RoundingOptions},
    primitive::FiniteF64,
    Sign, TemporalError, TemporalResult,
};
use alloc::string::String;

use super::{timezone::TZ_PROVIDER, DateDuration, TimeDuration};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub struct Duration(pub(crate) temporal_core::Duration);

impl core::fmt::Display for Duration {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(
            &self
                .as_temporal_string(ToStringRoundingOptions::default())
                .expect("Default options on a valid Duration should return a string."),
        )
    }
}

impl From<temporal_core::Duration> for Duration {
    fn from(value: temporal_core::Duration) -> Self {
        Self(value)
    }
}

impl From<DateDuration> for Duration {
    fn from(value: DateDuration) -> Self {
        Self(value.into())
    }
}

impl From<TimeDuration> for Duration {
    fn from(value: TimeDuration) -> Self {
        Self(value.into())
    }
}

impl Duration {
    /// Creates a new validated `Duration`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        years: FiniteF64,
        months: FiniteF64,
        weeks: FiniteF64,
        days: FiniteF64,
        hours: FiniteF64,
        minutes: FiniteF64,
        seconds: FiniteF64,
        milliseconds: FiniteF64,
        microseconds: FiniteF64,
        nanoseconds: FiniteF64,
    ) -> TemporalResult<Self> {
        temporal_core::Duration::new(
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        )
        .map(Into::into)
    }

    /// Creates a `Duration` from a provided `PartialDuration`.
    pub fn from_partial_duration(partial: PartialDuration) -> TemporalResult<Self> {
        temporal_core::Duration::from_partial_duration(partial).map(Into::into)
    }
}

// ==== Public `Duration` Getters/Setters ====

impl Duration {
    /// Returns a reference to the inner `TimeDuration`
    #[inline]
    #[must_use]
    pub fn time(&self) -> &TimeDuration {
        self.0.time()
    }

    /// Returns a reference to the inner `DateDuration`
    #[inline]
    #[must_use]
    pub fn date(&self) -> &DateDuration {
        self.0.date()
    }

    /// Set this `DurationRecord`'s `TimeDuration`.
    #[inline]
    pub fn set_time_duration(&mut self, time: TimeDuration) {
        self.0.set_time_duration(time);
    }

    /// Returns the `years` field of duration.
    #[inline]
    #[must_use]
    pub const fn years(&self) -> FiniteF64 {
        self.0.years()
    }

    /// Returns the `months` field of duration.
    #[inline]
    #[must_use]
    pub const fn months(&self) -> FiniteF64 {
        self.0.months()
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn weeks(&self) -> FiniteF64 {
        self.0.weeks()
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn days(&self) -> FiniteF64 {
        self.0.days()
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn hours(&self) -> FiniteF64 {
        self.0.hours()
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn minutes(&self) -> FiniteF64 {
        self.0.minutes()
    }

    /// Returns the `seconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn seconds(&self) -> FiniteF64 {
        self.0.seconds()
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn milliseconds(&self) -> FiniteF64 {
        self.0.milliseconds()
    }

    /// Returns the `microseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn microseconds(&self) -> FiniteF64 {
        self.0.microseconds()
    }

    /// Returns the `nanoseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn nanoseconds(&self) -> FiniteF64 {
        self.0.nanoseconds()
    }
}

// ==== Public Duration methods ====

impl Duration {
    /// Determines the sign for the current self.
    #[inline]
    #[must_use]
    pub fn sign(&self) -> Sign {
        self.0.sign()
    }

    /// Returns whether the current `Duration` is zero.
    ///
    /// Equivalant to `Temporal.Duration.blank()`.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns a negated `Duration`
    #[inline]
    #[must_use]
    pub fn negated(&self) -> Self {
        self.0.negated().into()
    }

    /// Returns the absolute value of `Duration`.
    #[inline]
    #[must_use]
    pub fn abs(&self) -> Self {
        self.0.abs().into()
    }

    /// Returns the result of adding a `Duration` to the current `Duration`
    #[inline]
    pub fn add(&self, other: &Self) -> TemporalResult<Self> {
        self.0.add(&other.0).map(Into::into)
    }

    /// Returns the result of subtracting a `Duration` from the current `Duration`
    #[inline]
    pub fn subtract(&self, other: &Self) -> TemporalResult<Self> {
        self.0.subtract(&other.0).map(Into::into)
    }

    pub fn round(
        &self,
        options: RoundingOptions,
        relative_to: Option<RelativeTo>,
    ) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.0
            .round_with_provider(options, relative_to.map(Into::into), &*provider)
            .map(Into::into)
    }

    pub fn as_temporal_string(&self, options: ToStringRoundingOptions) -> TemporalResult<String> {
        self.0.as_temporal_string(options)
    }
}
