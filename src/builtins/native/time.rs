use crate::{
    builtins::core,
    options::{
        ArithmeticOverflow, DifferenceSettings, TemporalRoundingMode, TemporalUnit,
        ToStringRoundingOptions,
    },
    TemporalResult,
};
use alloc::string::String;

use super::{Duration, PartialTime, TimeDuration};
pub struct PlainTime(pub(crate) core::PlainTime);

impl From<core::PlainTime> for PlainTime {
    fn from(value: core::PlainTime) -> Self {
        Self(value)
    }
}

impl PlainTime {
    /// Creates a new `PlainTime`, constraining any field into a valid range.
    ///
    /// ```rust
    /// use temporal_rs::PlainTime;
    ///
    /// let time = PlainTime::new(23, 59, 59, 999, 999, 999).unwrap();
    ///
    /// let constrained_time = PlainTime::new(24, 59, 59, 999, 999, 999).unwrap();
    /// assert_eq!(time, constrained_time);
    /// ```
    pub fn new(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
    ) -> TemporalResult<Self> {
        core::PlainTime::new(hour, minute, second, millisecond, microsecond, nanosecond)
            .map(Into::into)
    }

    /// Creates a new `PlainTime`, rejecting any field that is not in a valid range.
    ///
    /// ```rust
    /// use temporal_rs::PlainTime;
    ///
    /// let time = PlainTime::try_new(23, 59, 59, 999, 999, 999).unwrap();
    ///
    /// let invalid_time = PlainTime::try_new(24, 59, 59, 999, 999, 999);
    /// assert!(invalid_time.is_err());
    /// ```
    pub fn try_new(
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
    ) -> TemporalResult<Self> {
        core::PlainTime::try_new(hour, minute, second, millisecond, microsecond, nanosecond)
            .map(Into::into)
    }

    /// Creates a new `PlainTime` from a `PartialTime`.
    ///
    /// ```rust
    /// use temporal_rs::{partial::PartialTime, PlainTime};
    ///
    /// let partial_time = PartialTime {
    ///     hour: Some(22),
    ///     ..Default::default()
    /// };
    ///
    /// let time = PlainTime::from_partial(partial_time, None).unwrap();
    ///
    /// assert_eq!(time.hour(), 22);
    /// assert_eq!(time.minute(), 0);
    /// assert_eq!(time.second(), 0);
    /// assert_eq!(time.millisecond(), 0);
    /// assert_eq!(time.microsecond(), 0);
    /// assert_eq!(time.nanosecond(), 0);
    ///
    /// ```
    pub fn from_partial(
        partial: PartialTime,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        core::PlainTime::from_partial(partial, overflow).map(Into::into)
    }

    /// Creates a new `PlainTime` using the current `PlainTime` fields as a fallback.
    ///
    /// ```rust
    /// use temporal_rs::{partial::PartialTime, PlainTime};
    ///
    /// let partial_time = PartialTime {
    ///     hour: Some(22),
    ///     ..Default::default()
    /// };
    ///
    /// let initial = PlainTime::try_new(15, 30, 12, 123, 456, 789).unwrap();
    ///
    /// let time = initial.with(partial_time, None).unwrap();
    ///
    /// assert_eq!(time.hour(), 22);
    /// assert_eq!(time.minute(), 30);
    /// assert_eq!(time.second(), 12);
    /// assert_eq!(time.millisecond(), 123);
    /// assert_eq!(time.microsecond(), 456);
    /// assert_eq!(time.nanosecond(), 789);
    ///
    /// ```
    pub fn with(
        &self,
        partial: PartialTime,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.0.with(partial, overflow).map(Into::into)
    }

    /// Returns the internal `hour` field.
    #[inline]
    #[must_use]
    pub const fn hour(&self) -> u8 {
        self.0.hour()
    }

    /// Returns the internal `minute` field.
    #[inline]
    #[must_use]
    pub const fn minute(&self) -> u8 {
        self.0.minute()
    }

    /// Returns the internal `second` field.
    #[inline]
    #[must_use]
    pub const fn second(&self) -> u8 {
        self.0.second()
    }

    /// Returns the internal `millisecond` field.
    #[inline]
    #[must_use]
    pub const fn millisecond(&self) -> u16 {
        self.0.millisecond()
    }

    /// Returns the internal `microsecond` field.
    #[inline]
    #[must_use]
    pub const fn microsecond(&self) -> u16 {
        self.0.microsecond()
    }

    /// Returns the internal `nanosecond` field.
    #[inline]
    #[must_use]
    pub const fn nanosecond(&self) -> u16 {
        self.0.nanosecond()
    }

    /// Add a `Duration` to the current `Time`.
    pub fn add(&self, duration: &Duration) -> TemporalResult<Self> {
        self.0.add(&duration.0).map(Into::into)
    }

    /// Adds a `TimeDuration` to the current `Time`.
    #[inline]
    pub fn add_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.0.add_time_duration(duration).map(Into::into)
    }

    /// Subtract a `Duration` to the current `Time`.
    pub fn subtract(&self, duration: &Duration) -> TemporalResult<Self> {
        self.0.subtract(&duration.0).map(Into::into)
    }

    /// Adds a `TimeDuration` to the current `Time`.
    #[inline]
    pub fn subtract_time_duration(&self, duration: &TimeDuration) -> TemporalResult<Self> {
        self.0.subtract_time_duration(duration).map(Into::into)
    }

    #[inline]
    /// Returns the `Duration` until the provided `Time` from the current `Time`.
    ///
    /// NOTE: `until` assumes the provided other time will occur in the future relative to the current.
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.until(&other.0, settings).map(Into::into)
    }

    #[inline]
    /// Returns the `Duration` since the provided `Time` from the current `Time`.
    ///
    /// NOTE: `since` assumes the provided other time is in the past relative to the current.
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.since(&other.0, settings).map(Into::into)
    }

    /// Rounds the current `Time` according to provided options.
    pub fn round(
        &self,
        smallest_unit: TemporalUnit,
        rounding_increment: Option<f64>,
        rounding_mode: Option<TemporalRoundingMode>,
    ) -> TemporalResult<Self> {
        self.0
            .round(smallest_unit, rounding_increment, rounding_mode)
            .map(Into::into)
    }

    pub fn to_ixdtf_string(&self, options: ToStringRoundingOptions) -> TemporalResult<String> {
        self.0.as_ixdtf_string(options)
    }
}
