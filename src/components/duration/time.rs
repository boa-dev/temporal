//! An implementation of `TimeDuration` and it's methods.

use crate::{
    options::{TemporalRoundingMode, TemporalUnit},
    TemporalError, TemporalResult,
};

use super::{is_valid_duration, normalized::NormalizedTimeDuration};

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
const NANOSECONDS_PER_MINUTE: u64 = NANOSECONDS_PER_SECOND * 60;
const NANOSECONDS_PER_HOUR: u64 = NANOSECONDS_PER_MINUTE * 60;

/// `TimeDuration` represents the [Time Duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-time-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
pub struct TimeDuration {
    pub(crate) hours: f64,
    pub(crate) minutes: f64,
    pub(crate) seconds: f64,
    pub(crate) milliseconds: f64,
    pub(crate) microseconds: f64,
    pub(crate) nanoseconds: f64,
}
// ==== TimeDuration Private API ====

impl TimeDuration {
    /// Creates a new `TimeDuration`.
    #[must_use]
    pub(crate) const fn new_unchecked(
        hours: f64,
        minutes: f64,
        seconds: f64,
        milliseconds: f64,
        microseconds: f64,
        nanoseconds: f64,
    ) -> Self {
        Self {
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        }
    }
}

// ==== TimeDuration's public API ====

impl TimeDuration {
    /// Creates a new validated `TimeDuration`.
    pub fn new(
        hours: f64,
        minutes: f64,
        seconds: f64,
        milliseconds: f64,
        microseconds: f64,
        nanoseconds: f64,
    ) -> TemporalResult<Self> {
        let result = Self::new_unchecked(
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        );
        if !is_valid_duration(&result.into_iter().collect()) {
            return Err(
                TemporalError::range().with_message("Attempted to create an invalid TimeDuration.")
            );
        }
        Ok(result)
    }

    /// Creates a partial `TimeDuration` with all values set to `NaN`.
    #[must_use]
    pub const fn partial() -> Self {
        Self {
            hours: f64::NAN,
            minutes: f64::NAN,
            seconds: f64::NAN,
            milliseconds: f64::NAN,
            microseconds: f64::NAN,
            nanoseconds: f64::NAN,
        }
    }

    /// Creates a `TimeDuration` from a provided partial `TimeDuration`.
    #[must_use]
    pub fn from_partial(partial: &TimeDuration) -> Self {
        Self {
            hours: if partial.hours.is_nan() {
                0.0
            } else {
                partial.hours
            },
            minutes: if partial.minutes.is_nan() {
                0.0
            } else {
                partial.minutes
            },
            seconds: if partial.seconds.is_nan() {
                0.0
            } else {
                partial.seconds
            },
            milliseconds: if partial.milliseconds.is_nan() {
                0.0
            } else {
                partial.milliseconds
            },
            microseconds: if partial.microseconds.is_nan() {
                0.0
            } else {
                partial.microseconds
            },
            nanoseconds: if partial.nanoseconds.is_nan() {
                0.0
            } else {
                partial.nanoseconds
            },
        }
    }

    /// Balances and creates `TimeDuration` from a `NormalizedTimeDuration`. This method will return
    /// a tuple (f64, TimeDuration) where f64 is the overflow day value from balancing.
    ///
    /// Equivalent: `BalanceTimeDuration`
    ///
    /// # Errors:
    ///   - Will error if provided duration is invalid
    pub(crate) fn from_normalized(
        norm: NormalizedTimeDuration,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<(f64, Self)> {
        // 1. Let days, hours, minutes, seconds, milliseconds, and microseconds be 0.
        let mut days = 0f64;
        let mut hours = 0f64;
        let mut minutes = 0f64;
        let mut seconds = 0f64;
        let mut milliseconds = 0f64;
        let mut microseconds = 0f64;

        // 2. Let sign be NormalizedTimeDurationSign(norm).
        let sign = f64::from(norm.sign());
        // 3. Let nanoseconds be NormalizedTimeDurationAbs(norm).[[TotalNanoseconds]].
        let mut nanoseconds = norm.0.abs();

        match largest_unit {
            // 4. If largestUnit is "year", "month", "week", or "day", then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                microseconds = (nanoseconds / 1000f64).floor();
                // b. Set nanoseconds to nanoseconds modulo 1000.
                nanoseconds = nanoseconds.rem_euclid(1000.0);

                // c. Set milliseconds to floor(microseconds / 1000).
                milliseconds = (microseconds / 1000f64).floor();
                // d. Set microseconds to microseconds modulo 1000.
                microseconds = microseconds.rem_euclid(1000.0);

                // e. Set seconds to floor(milliseconds / 1000).
                seconds = (milliseconds / 1000f64).floor();
                // f. Set milliseconds to milliseconds modulo 1000.
                milliseconds = milliseconds.rem_euclid(1000.0);

                // g. Set minutes to floor(seconds / 60).
                minutes = (seconds / 60f64).floor();
                // h. Set seconds to seconds modulo 60.
                seconds = seconds.rem_euclid(60.0);

                // i. Set hours to floor(minutes / 60).
                hours = (minutes / 60f64).floor();
                // j. Set minutes to minutes modulo 60.
                minutes = minutes.rem_euclid(60.0);

                // k. Set days to floor(hours / 24).
                days = (hours / 24f64).floor();
                // l. Set hours to hours modulo 24.
                hours = hours.rem_euclid(24.0);
            }
            // 5. Else if largestUnit is "hour", then
            TemporalUnit::Hour => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                microseconds = (nanoseconds / 1000f64).floor();
                // b. Set nanoseconds to nanoseconds modulo 1000.
                nanoseconds %= 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                milliseconds = (microseconds / 1000f64).floor();
                // d. Set microseconds to microseconds modulo 1000.
                microseconds %= 1000f64;

                // e. Set seconds to floor(milliseconds / 1000).
                seconds = (milliseconds / 1000f64).floor();
                // f. Set milliseconds to milliseconds modulo 1000.
                milliseconds %= 1000f64;

                // g. Set minutes to floor(seconds / 60).
                minutes = (seconds / 60f64).floor();
                // h. Set seconds to seconds modulo 60.
                seconds %= 60f64;

                // i. Set hours to floor(minutes / 60).
                hours = (minutes / 60f64).floor();
                // j. Set minutes to minutes modulo 60.
                minutes %= 60f64;
            }
            // 6. Else if largestUnit is "minute", then
            TemporalUnit::Minute => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                microseconds = (nanoseconds / 1000f64).floor();
                nanoseconds %= 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                milliseconds = (microseconds / 1000f64).floor();
                microseconds %= 1000f64;

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                seconds = (milliseconds / 1000f64).floor();
                milliseconds %= 1000f64;

                // g. Set minutes to floor(seconds / 60).
                // h. Set seconds to seconds modulo 60.
                minutes = (seconds / 60f64).floor();
                seconds %= 60f64;
            }
            // 7. Else if largestUnit is "second", then
            TemporalUnit::Second => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                microseconds = (nanoseconds / 1000f64).floor();
                nanoseconds %= 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                milliseconds = (microseconds / 1000f64).floor();
                microseconds %= 1000f64;

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                seconds = (milliseconds / 1000f64).floor();
                milliseconds %= 1000f64;
            }
            // 8. Else if largestUnit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                microseconds = (nanoseconds / 1000f64).floor();
                nanoseconds %= 1000f64;

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                milliseconds = (microseconds / 1000f64).floor();
                microseconds %= 1000f64;
            }
            // 9. Else if largestUnit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                microseconds = (nanoseconds / 1000f64).floor();
                nanoseconds %= 1000f64;
            }
            // 10. Else,
            // a. Assert: largestUnit is "nanosecond".
            _ => debug_assert!(largest_unit == TemporalUnit::Nanosecond),
        }

        // NOTE(nekevss): `mul_add` is essentially the Rust's implementation of `std::fma()`, so that's handy, but
        // this should be tested much further.
        // 11. NOTE: When largestUnit is "millisecond", "microsecond", or "nanosecond", milliseconds, microseconds, or
        // nanoseconds may be an unsafe integer. In this case, care must be taken when implementing the calculation
        // using floating point arithmetic. It can be implemented in C++ using std::fma(). String manipulation will also
        // give an exact result, since the multiplication is by a power of 10.

        // 12. Return ! CreateTimeDurationRecord(days × sign, hours × sign, minutes × sign, seconds × sign, milliseconds × sign, microseconds × sign, nanoseconds × sign).
        let days = days.mul_add(sign, 0.0);
        let result = Self::new_unchecked(
            hours.mul_add(sign, 0.0),
            minutes.mul_add(sign, 0.0),
            seconds.mul_add(sign, 0.0),
            milliseconds.mul_add(sign, 0.0),
            microseconds.mul_add(sign, 0.0),
            nanoseconds.mul_add(sign, 0.0),
        );

        let td = Vec::from(&[
            days,
            result.hours,
            result.minutes,
            result.seconds,
            result.milliseconds,
            result.microseconds,
            result.nanoseconds,
        ]);
        if !is_valid_duration(&td) {
            return Err(TemporalError::range().with_message("Invalid balance TimeDuration."));
        }

        Ok((days, result))
    }

    /// Returns a new `TimeDuration` representing the absolute value of the current.
    #[inline]
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            hours: self.hours.abs(),
            minutes: self.minutes.abs(),
            seconds: self.seconds.abs(),
            milliseconds: self.milliseconds.abs(),
            microseconds: self.microseconds.abs(),
            nanoseconds: self.nanoseconds.abs(),
        }
    }

    /// Returns a negated `TimeDuration`.
    #[inline]
    #[must_use]
    pub fn negated(&self) -> Self {
        Self {
            hours: self.hours * -1f64,
            minutes: self.minutes * -1f64,
            seconds: self.seconds * -1f64,
            milliseconds: self.milliseconds * -1f64,
            microseconds: self.microseconds * -1f64,
            nanoseconds: self.nanoseconds * -1f64,
        }
    }

    /// Utility function for returning if values in a valid range.
    #[inline]
    #[must_use]
    pub fn is_within_range(&self) -> bool {
        self.hours.abs() < 24f64
            && self.minutes.abs() < 60f64
            && self.seconds.abs() < 60f64
            && self.milliseconds.abs() < 1000f64
            && self.milliseconds.abs() < 1000f64
            && self.milliseconds.abs() < 1000f64
    }

    /// Returns the `[[hours]]` value.
    #[must_use]
    pub const fn hours(&self) -> f64 {
        self.hours
    }

    /// Returns the `[[minutes]]` value.
    #[must_use]
    pub const fn minutes(&self) -> f64 {
        self.minutes
    }

    /// Returns the `[[seconds]]` value.
    #[must_use]
    pub const fn seconds(&self) -> f64 {
        self.seconds
    }

    /// Returns the `[[milliseconds]]` value.
    #[must_use]
    pub const fn milliseconds(&self) -> f64 {
        self.milliseconds
    }

    /// Returns the `[[microseconds]]` value.
    #[must_use]
    pub const fn microseconds(&self) -> f64 {
        self.microseconds
    }

    /// Returns the `[[nanoseconds]]` value.
    #[must_use]
    pub const fn nanoseconds(&self) -> f64 {
        self.nanoseconds
    }

    /// Returns the `TimeDuration`'s iterator.
    #[must_use]
    pub fn iter(&self) -> TimeIter<'_> {
        <&Self as IntoIterator>::into_iter(self)
    }

    /// Returns this `TimeDuration` as a `NormalizedTimeDuration`.
    pub(crate) fn to_normalized(self) -> NormalizedTimeDuration {
        NormalizedTimeDuration::from_time_duration(&self)
    }
}

// ==== TimeDuration method impls ====

impl TimeDuration {
    // TODO: Update round to accomodate `Normalization`.
    /// Rounds the current `TimeDuration` given a rounding increment, unit and rounding mode. `round` will return a tuple of the rounded `TimeDuration` and
    /// the `total` value of the smallest unit prior to rounding.
    #[inline]
    pub fn round(
        &self,
        increment: u64,
        unit: TemporalUnit,
        mode: TemporalRoundingMode,
    ) -> TemporalResult<(NormalizedTimeDuration, i64)> {
        let norm = match unit {
            TemporalUnit::Year
            | TemporalUnit::Month
            | TemporalUnit::Week
            | TemporalUnit::Day
            | TemporalUnit::Auto => {
                return Err(TemporalError::r#type()
                    .with_message("Invalid unit provided to for TimeDuration to round."))
            }
            _ => self.to_normalized(),
        };

        match unit {
            // 12. Else if unit is "hour", then
            TemporalUnit::Hour => {
                // a. Let divisor be 3.6 × 10**12.
                // b. Set total to DivideNormalizedTimeDuration(norm, divisor).
                let total = norm.divide(NANOSECONDS_PER_HOUR as i64);
                // c. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, divisor × increment, roundingMode).
                let norm = norm.round(NANOSECONDS_PER_HOUR * increment, mode)?;
                Ok((norm, total))
            }
            // 13. Else if unit is "minute", then
            TemporalUnit::Minute => {
                // a. Let divisor be 6 × 10**10.
                // b. Set total to DivideNormalizedTimeDuration(norm, divisor).
                let total = norm.divide(NANOSECONDS_PER_MINUTE as i64);
                // c. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, divisor × increment, roundingMode).
                let norm = norm.round(NANOSECONDS_PER_MINUTE * increment, mode)?;
                Ok((norm, total))
            }
            // 14. Else if unit is "second", then
            TemporalUnit::Second => {
                // a. Let divisor be 10**9.
                // b. Set total to DivideNormalizedTimeDuration(norm, divisor).
                let total = norm.divide(NANOSECONDS_PER_SECOND as i64);
                // c. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, divisor × increment, roundingMode).
                let norm = norm.round(NANOSECONDS_PER_SECOND * increment, mode)?;
                Ok((norm, total))
            }
            // 15. Else if unit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Let divisor be 10**6.
                // b. Set total to DivideNormalizedTimeDuration(norm, divisor).
                let total = norm.divide(1_000_000);
                // c. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, divisor × increment, roundingMode).
                let norm = norm.round(1_000_000 * increment, mode)?;
                Ok((norm, total))
            }
            // 16. Else if unit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Let divisor be 10**3.
                // b. Set total to DivideNormalizedTimeDuration(norm, divisor).
                let total = norm.divide(1_000);
                // c. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, divisor × increment, roundingMode).
                let norm = norm.round(1_000 * increment, mode)?;
                Ok((norm, total))
            }
            // 17. Else,
            TemporalUnit::Nanosecond => {
                // a. Assert: unit is "nanosecond".
                // b. Set total to NormalizedTimeDurationSeconds(norm) × 10**9 + NormalizedTimeDurationSubseconds(norm).
                let total =
                    norm.seconds() * (NANOSECONDS_PER_SECOND as i64) + i64::from(norm.subseconds());
                // c. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, increment, roundingMode).
                let norm = norm.round(increment, mode)?;
                Ok((norm, total))
            }
            _ => unreachable!("All other units early return error."),
        }
    }
}

impl<'a> IntoIterator for &'a TimeDuration {
    type Item = f64;
    type IntoIter = TimeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        TimeIter {
            time: self,
            index: 0,
        }
    }
}

/// An iterator over a `TimeDuration`.
#[derive(Debug, Clone)]
pub struct TimeIter<'a> {
    time: &'a TimeDuration,
    index: usize,
}

impl Iterator for TimeIter<'_> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        let result = match self.index {
            0 => Some(self.time.hours),
            1 => Some(self.time.minutes),
            2 => Some(self.time.seconds),
            3 => Some(self.time.milliseconds),
            4 => Some(self.time.microseconds),
            5 => Some(self.time.nanoseconds),
            _ => None,
        };
        self.index += 1;
        result
    }
}
