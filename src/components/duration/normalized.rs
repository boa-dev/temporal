//! This module implements the normalized `Duration` records.

use crate::{TemporalError, TemporalResult, NS_PER_DAY};

use super::TimeDuration;

const MAX_TIME_DURATION: f64 = 2e53 * 10e9 - 1.0;

#[derive(Debug, Clone, Copy, Default, PartialEq, PartialOrd)]
pub(crate) struct NormalizedTimeDuration(pub(crate) f64);

impl NormalizedTimeDuration {
    /// Equivalent: 7.5.20 NormalizeTimeDuration ( hours, minutes, seconds, milliseconds, microseconds, nanoseconds )
    pub(crate) fn from_time_duration(time: &TimeDuration) -> Self {
        let minutes = time.minutes + time.hours * 60.0;
        let seconds = time.seconds + minutes * 60.0;
        let milliseconds = time.milliseconds + seconds * 1000.0;
        let microseconds = time.microseconds + milliseconds * 1000.0;
        let nanoseconds = time.nanoseconds + microseconds * 1000.0;
        // NOTE(nekevss): Is it worth returning a `RangeError` below.
        debug_assert!(nanoseconds.abs() <= MAX_TIME_DURATION);
        Self(nanoseconds)
    }

    /// Equivalent: 7.5.22 AddNormalizedTimeDuration ( one, two )
    #[allow(unused)]
    pub(crate) fn add(&self, other: &Self) -> TemporalResult<Self> {
        let result = self.0 + other.0;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }

    /// Equivalent: 7.5.23 Add24HourDaysToNormalizedTimeDuration ( d, days )
    #[allow(unused)]
    pub(crate) fn add_days(&self, days: f64) -> TemporalResult<Self> {
        let result = self.0 + days * NS_PER_DAY as f64;
        if result.abs() > MAX_TIME_DURATION {
            return Err(TemporalError::range()
                .with_message("normalizedTimeDuration exceeds maxTimeDuration."));
        }
        Ok(Self(result))
    }

    // NOTE: DivideNormalizedTimeDuration probably requires `__float128` support as `NormalizedTimeDuration` is not `safe integer`.
    // Tracking issue: https://github.com/rust-lang/rfcs/pull/3453

    /// Equivalent: 7.5.31 NormalizedTimeDurationSign ( d )
    pub(crate) fn sign(&self) -> f64 {
        if self.0 < 0.0 {
            return -1.0;
        } else if self.0 > 0.0 {
            return 1.0;
        }
        0.0
    }
}
