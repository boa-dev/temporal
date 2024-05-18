//! This module implements the normalized `Duration` records.

use std::ops::Add;

use crate::{options::TemporalRoundingMode, utils, TemporalError, TemporalResult, NS_PER_DAY};

use super::{DateDuration, TimeDuration};

const MAX_TIME_DURATION: f64 = 2e53 * 10e9 - 1.0;

/// A Normalized `TimeDuration` that represents the current `TimeDuration` in nanoseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct NormalizedTimeDuration(pub(crate) f64);

impl NormalizedTimeDuration {
    /// Equivalent: 7.5.20 NormalizeTimeDuration ( hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
    pub(super) fn from_time_duration(time: &TimeDuration) -> Self {
        let minutes = time.minutes + time.hours * 60.0;
        let seconds = time.seconds + minutes * 60.0;
        let milliseconds = time.milliseconds + seconds * 1000.0;
        let microseconds = time.microseconds + milliseconds * 1000.0;
        let nanoseconds = time.nanoseconds + microseconds * 1000.0;
        // NOTE(nekevss): Is it worth returning a `RangeError` below.
        debug_assert!(nanoseconds.abs() <= MAX_TIME_DURATION);
        Self(nanoseconds)
    }

    /// Equivalent: 7.5.23 Add24HourDaysToNormalizedTimeDuration ( d, days )
    #[allow(unused)]
    pub(super) fn add_days(&self, days: f64) -> TemporalResult<Self> {
        let result = self.0 + days * NS_PER_DAY as f64;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }

    // TODO: Implement as `ops::Div`
    /// `Divide the NormalizedTimeDuraiton` by a divisor.
    pub(super) fn divide(&self, divisor: i64) -> f64 {
        // TODO: Validate.
        self.0 / (divisor as f64)
    }

    /// Equivalent: 7.5.31 NormalizedTimeDurationSign ( d )
    #[inline]
    #[must_use]
    pub(super) fn sign(&self) -> i32 {
        if self.0 < 0.0 {
            return -1;
        } else if self.0 > 0.0 {
            return 1;
        }
        0
    }

    /// Return the seconds value of the `NormalizedTimeDuration`.
    pub(crate) fn seconds(&self) -> i64 {
        (self.0 / 10e9).trunc() as i64
    }

    /// Returns the subsecond components of the `NormalizedTimeDuration`.
    pub(crate) fn subseconds(&self) -> i32 {
        // SAFETY: Remainder is 10e9 which is in range of i32
        (self.0 % 10e9f64) as i32
    }

    /// Round the current `NormalizedTimeDuration`.
    pub(super) fn round(&self, increment: u64, mode: TemporalRoundingMode) -> TemporalResult<Self> {
        let rounded = utils::round_number_to_increment(self.0, increment as f64, mode);
        if rounded.abs() > MAX_TIME_DURATION as i64 {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(rounded as f64))
    }
}

// NOTE(nekevss): As this `Add` impl is fallible. Maybe it would be best implemented as a method.
/// Equivalent: 7.5.22 AddNormalizedTimeDuration ( one, two )
impl Add<Self> for NormalizedTimeDuration {
    type Output = TemporalResult<Self>;

    fn add(self, rhs: Self) -> Self::Output {
        let result = self.0 + rhs.0;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }
}

/// A normalized `DurationRecord` that contains a `DateDuration` and `NormalizedTimeDuration`.
#[derive(Debug, Clone, Copy)]
pub struct NormalizedDurationRecord(pub(crate) (DateDuration, NormalizedTimeDuration));

impl NormalizedDurationRecord {
    /// Creates a new `NormalizedDurationRecord`.
    ///
    /// Equivalent: `CreateNormalizedDurationRecord` & `CombineDateAndNormalizedTimeDuration`.
    pub(super) fn new(date: DateDuration, norm: NormalizedTimeDuration) -> TemporalResult<Self> {
        if date.sign() != 0 && norm.sign() != 0 && date.sign() != norm.sign() {
            return Err(TemporalError::range()
                .with_message("DateDuration and NormalizedTimeDuration must agree."));
        }
        Ok(Self((date, norm)))
    }
}
