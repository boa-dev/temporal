//! An implementation of `TimeDuration` and it's methods.

use crate::{
    options::{TemporalRoundingMode, TemporalUnit},
    utils, TemporalError, TemporalResult,
};

use super::{is_valid_duration, normalized::NormalizedTimeDuration};

/// `TimeDuration` represents the [Time Duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-time-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
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

    /// Returns the current `TimeDuration` as nanoseconds.
    #[inline]
    pub(crate) fn as_nanos(&self) -> f64 {
        self.hours
            .mul_add(60_f64, self.minutes)
            .mul_add(60_f64, self.seconds)
            .mul_add(1_000_f64, self.milliseconds)
            .mul_add(1_000_f64, self.microseconds)
            .mul_add(1_000_f64, self.nanoseconds)
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

    /// Balances a `TimeDuration` given a day value and the largest unit. `balance` will return
    /// the balanced `day` and `TimeDuration`.
    ///
    /// # Errors:
    ///   - Will error if provided duration is invalid
    pub fn balance(&self, largest_unit: TemporalUnit) -> TemporalResult<(f64, Self)> {
        let norm = NormalizedTimeDuration::from_time_duration(&self);

        // 1. Let days, hours, minutes, seconds, milliseconds, and microseconds be 0.
        let mut days = 0f64;
        let mut hours = 0f64;
        let mut minutes = 0f64;
        let mut seconds = 0f64;
        let mut milliseconds = 0f64;
        let mut microseconds = 0f64;

        // 2. Let sign be NormalizedTimeDurationSign(norm).
        let sign = norm.sign();
        // 3. Let nanoseconds be NormalizedTimeDurationAbs(norm).[[TotalNanoseconds]].
        let mut nanoseconds = norm.0.abs();

        match largest_unit {
            // 4. If largestUnit is "year", "month", "week", or "day", then
            TemporalUnit::Year
            | TemporalUnit::Month
            | TemporalUnit::Week
            | TemporalUnit::Day => {
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

                // k. Set days to floor(hours / 24).
                days = (hours / 24f64).floor();
                // l. Set hours to hours modulo 24.
                hours %= 24f64;
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
        // 11. NOTE: When largestUnit is "millisecond", "microsecond", or "nanosecond", milliseconds, microseconds, or nanoseconds may be an unsafe integer. In this case,
        // care must be taken when implementing the calculation using floating point arithmetic. It can be implemented in C++ using std::fma(). String manipulation will also
        // give an exact result, since the multiplication is by a power of 10.
        // 12. Return ! CreateTimeDurationRecord(days × sign, hours × sign, minutes × sign, seconds × sign, milliseconds × sign, microseconds × sign, nanoseconds × sign).
        let days = days.mul_add(sign, 0.0);
        let result = Self::new_unchecked(hours.mul_add(sign, 0.0), minutes.mul_add(sign, 0.0), seconds.mul_add(sign, 0.0), milliseconds.mul_add(sign, 0.0), microseconds.mul_add(sign, 0.0), nanoseconds.mul_add(sign, 0.0));

        let td = Vec::from(&[days, result.hours, result.minutes, result.seconds, result.milliseconds, result.microseconds, result.nanoseconds]);
        if !is_valid_duration(&td) {
            return Err(TemporalError::range().with_message("Invalid balance TimeDuration."));
        }

        Ok((days, result))
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
}

// ==== TimeDuration method impls ====

impl TimeDuration {
    // TODO: Update round to accomodate `Normalization`.
    /// Rounds the current `TimeDuration` given a rounding increment, unit and rounding mode. `round` will return a tuple of the rounded `TimeDuration` and
    /// the `total` value of the smallest unit prior to rounding.
    #[inline]
    pub fn round(
        &self,
        increment: f64,
        unit: TemporalUnit,
        rounding_mode: TemporalRoundingMode,
    ) -> TemporalResult<(Self, f64)> {
        let fraction_seconds = match unit {
            TemporalUnit::Year
            | TemporalUnit::Month
            | TemporalUnit::Week
            | TemporalUnit::Day
            | TemporalUnit::Auto => {
                return Err(TemporalError::r#type()
                    .with_message("Invalid unit provided to for TimeDuration to round."))
            }
            _ => self.nanoseconds().mul_add(
                1_000_000_000f64,
                self.microseconds().mul_add(
                    1_000_000f64,
                    self.milliseconds().mul_add(1000f64, self.seconds()),
                ),
            ),
        };

        match unit {
            // 12. Else if unit is "hour", then
            TemporalUnit::Hour => {
                // a. Let fractionalHours be (fractionalSeconds / 60 + minutes) / 60 + hours.
                let frac_hours = (fraction_seconds / 60f64 + self.minutes) / 60f64 + self.hours;
                // b. Set hours to RoundNumberToIncrement(fractionalHours, increment, roundingMode).
                let rounded_hours =
                    utils::round_number_to_increment(frac_hours, increment, rounding_mode);
                // c. Set total to fractionalHours.
                // d. Set minutes, seconds, milliseconds, microseconds, and nanoseconds to 0.
                let result = Self::new(rounded_hours, 0f64, 0f64, 0f64, 0f64, 0f64)?;
                Ok((result, frac_hours))
            }
            // 13. Else if unit is "minute", then
            TemporalUnit::Minute => {
                // a. Let fractionalMinutes be fractionalSeconds / 60 + minutes.
                let frac_minutes = fraction_seconds / 60f64 + self.minutes;
                // b. Set minutes to RoundNumberToIncrement(fractionalMinutes, increment, roundingMode).
                let rounded_minutes =
                    utils::round_number_to_increment(frac_minutes, increment, rounding_mode);
                // c. Set total to fractionalMinutes.
                // d. Set seconds, milliseconds, microseconds, and nanoseconds to 0.
                let result = Self::new(self.hours, rounded_minutes, 0f64, 0f64, 0f64, 0f64)?;

                Ok((result, frac_minutes))
            }
            // 14. Else if unit is "second", then
            TemporalUnit::Second => {
                // a. Set seconds to RoundNumberToIncrement(fractionalSeconds, increment, roundingMode).
                let rounded_seconds =
                    utils::round_number_to_increment(fraction_seconds, increment, rounding_mode);
                // b. Set total to fractionalSeconds.
                // c. Set milliseconds, microseconds, and nanoseconds to 0.
                let result =
                    Self::new(self.hours, self.minutes, rounded_seconds, 0f64, 0f64, 0f64)?;

                Ok((result, fraction_seconds))
            }
            // 15. Else if unit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Let fractionalMilliseconds be nanoseconds × 10-6 + microseconds × 10-3 + milliseconds.
                let fraction_millis = self.nanoseconds.mul_add(
                    1_000_000f64,
                    self.microseconds.mul_add(1_000f64, self.milliseconds),
                );

                // b. Set milliseconds to RoundNumberToIncrement(fractionalMilliseconds, increment, roundingMode).
                let rounded_millis =
                    utils::round_number_to_increment(fraction_millis, increment, rounding_mode);

                // c. Set total to fractionalMilliseconds.
                // d. Set microseconds and nanoseconds to 0.
                let result = Self::new(
                    self.hours,
                    self.minutes,
                    self.seconds,
                    rounded_millis,
                    0f64,
                    0f64,
                )?;
                Ok((result, fraction_millis))
            }
            // 16. Else if unit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Let fractionalMicroseconds be nanoseconds × 10-3 + microseconds.
                let frac_micros = self.nanoseconds.mul_add(1_000f64, self.microseconds);

                // b. Set microseconds to RoundNumberToIncrement(fractionalMicroseconds, increment, roundingMode).
                let rounded_micros =
                    utils::round_number_to_increment(frac_micros, increment, rounding_mode);

                // c. Set total to fractionalMicroseconds.
                // d. Set nanoseconds to 0.
                let result = Self::new(
                    self.hours,
                    self.minutes,
                    self.seconds,
                    self.milliseconds,
                    rounded_micros,
                    0f64,
                )?;
                Ok((result, frac_micros))
            }
            // 17. Else,
            TemporalUnit::Nanosecond => {
                // a. Assert: unit is "nanosecond".
                // b. Set total to nanoseconds.
                let total = self.nanoseconds;
                // c. Set nanoseconds to RoundNumberToIncrement(nanoseconds, increment, roundingMode).
                let rounded_nanos =
                    utils::round_number_to_increment(self.nanoseconds, increment, rounding_mode);

                let result = Self::new(
                    self.hours,
                    self.minutes,
                    self.seconds,
                    self.milliseconds,
                    self.microseconds,
                    rounded_nanos,
                )?;

                Ok((result, total))
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
