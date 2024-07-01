//! This module implements the normalized `Duration` records.

use std::{num::NonZeroU64, ops::Add};

use num_traits::Euclid;

use crate::{
    options::TemporalRoundingMode,
    rounding::{IncrementRounder, Round},
    TemporalError, TemporalResult, NS_PER_DAY,
};

use super::{DateDuration, TimeDuration};

const MAX_TIME_DURATION: i128 = 9_007_199_254_740_991_999_999_999;

// Nanoseconds constants

const NS_PER_DAY_128BIT: i128 = NS_PER_DAY as i128;
const NANOSECONDS_PER_MINUTE: f64 = 60.0 * 1e9;
const NANOSECONDS_PER_HOUR: f64 = 60.0 * 60.0 * 1e9;

// TODO: This should be moved to i128
/// A Normalized `TimeDuration` that represents the current `TimeDuration` in nanoseconds.
#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub struct NormalizedTimeDuration(pub(crate) i128);

impl NormalizedTimeDuration {
    /// Equivalent: 7.5.20 NormalizeTimeDuration ( hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
    pub(crate) fn from_time_duration(time: &TimeDuration) -> Self {
        // TODO: Determine if there is a loss in precision from casting. If so, times by 1,000 (calculate in picoseconds) than truncate?
        let mut nanoseconds: i128 = (time.hours * NANOSECONDS_PER_HOUR) as i128;
        nanoseconds += (time.minutes * NANOSECONDS_PER_MINUTE) as i128;
        nanoseconds += (time.seconds * 1_000_000_000.0) as i128;
        nanoseconds += (time.milliseconds * 1_000_000.0) as i128;
        nanoseconds += (time.microseconds * 1_000.0) as i128;
        nanoseconds += time.nanoseconds as i128;
        // NOTE(nekevss): Is it worth returning a `RangeError` below.
        debug_assert!(nanoseconds.abs() <= MAX_TIME_DURATION);
        Self(nanoseconds)
    }

    // NOTE: `days: f64` should be an integer -> `i64`.
    /// Equivalent: 7.5.23 Add24HourDaysToNormalizedTimeDuration ( d, days )
    #[allow(unused)]
    pub(crate) fn add_days(&self, days: i64) -> TemporalResult<Self> {
        let result = self.0 + i128::from(days) * i128::from(NS_PER_DAY);
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }

    // TODO: Potentially, update divisor to u64?
    /// `Divide the NormalizedTimeDuraiton` by a divisor.
    pub(super) fn divide(&self, divisor: i64) -> i128 {
        // TODO: Validate.
        self.0 / i128::from(divisor)
    }

    pub(super) fn div_rem(&self, divisor: u64) -> (i128, i128) {
        (self.0 / i128::from(divisor), self.0 % i128::from(divisor))
        // self.0.div_rem_euclid(&i128::from(divisor))
    }

    // TODO: Use in algorithm update or remove.
    #[allow(unused)]
    pub(super) fn as_fractional_days(&self) -> f64 {
        // TODO: Verify Max norm is within a castable f64 range.
        let (days, remainder) = self.0.div_rem_euclid(&NS_PER_DAY_128BIT);
        days as f64 + (remainder as f64 / NS_PER_DAY as f64)
    }

    // TODO: Potentially abstract sign into `Sign`
    /// Equivalent: 7.5.31 NormalizedTimeDurationSign ( d )
    #[inline]
    #[must_use]
    pub(crate) fn sign(&self) -> i32 {
        self.0.cmp(&0) as i32
    }

    /// Return the seconds value of the `NormalizedTimeDuration`.
    pub(crate) fn seconds(&self) -> i64 {
        // SAFETY: See validate_second_cast test.
        (self.0 / 1_000_000_000) as i64
    }

    /// Returns the subsecond components of the `NormalizedTimeDuration`.
    pub(crate) fn subseconds(&self) -> i32 {
        // SAFETY: Remainder is 10e9 which is in range of i32
        (self.0 % 1_000_000_000) as i32
    }

    pub(crate) fn checked_sub(&self, other: &Self) -> TemporalResult<Self> {
        let result = self.0 - other.0;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range().with_message(
                "SubtractNormalizedTimeDuration exceeded a valid TimeDuration range.",
            ));
        }
        Ok(Self(result))
    }

    /// Round the current `NormalizedTimeDuration`.
    pub(super) fn round(
        &self,
        increment: NonZeroU64,
        mode: TemporalRoundingMode,
    ) -> TemporalResult<Self> {
        let rounded = IncrementRounder::<i128>::from_potentially_negative_parts(self.0, increment)?
            .round(mode);
        if rounded.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(rounded))
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
    pub(crate) fn new(date: DateDuration, norm: NormalizedTimeDuration) -> TemporalResult<Self> {
        if date.sign() != 0 && norm.sign() != 0 && date.sign() != norm.sign() {
            return Err(TemporalError::range()
                .with_message("DateDuration and NormalizedTimeDuration must agree."));
        }
        Ok(Self((date, norm)))
    }

    pub(crate) fn from_date_duration(date: DateDuration) -> TemporalResult<Self> {
        Self::new(date, NormalizedTimeDuration::default())
    }

    pub(crate) fn date(&self) -> DateDuration {
        self.0 .0
    }

    pub(crate) fn norm(&self) -> NormalizedTimeDuration {
        self.0 .1
    }

    pub(crate) fn sign(&self) -> TemporalResult<i32> {
        if self.0 .0.sign() != self.0 .1.sign() {
            return Err(TemporalError::range().with_message("Invalid NormalizedDuration signs."));
        }
        Ok(self.0 .0.sign())
    }
}

mod tests {
    #[test]
    fn validate_seconds_cast() {
        let max_seconds = super::MAX_TIME_DURATION.div_euclid(1_000_000_000);
        assert!(max_seconds <= i64::MAX.into())
    }

    // TODO: test f64 cast.
}
