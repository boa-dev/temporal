//! An implementation of `TimeDuration` and it's methods.

use std::num::NonZeroU128;

use crate::{
    options::{ResolvedRoundingOptions, TemporalUnit},
    rounding::{IncrementRounder, Round},
    TemporalError, TemporalResult, TemporalUnwrap,
};

use super::{
    is_valid_duration,
    normalized::{NormalizedDurationRecord, NormalizedTimeDuration},
    DateDuration,
};

use num_traits::{Euclid, FromPrimitive, MulAdd};

/// `TimeDuration` represents the [Time Duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-time-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
pub struct TimeDuration {
    /// `TimeDuration`'s internal hour value.
    pub hours: f64,
    /// `TimeDuration`'s internal minute value.
    pub minutes: f64,
    /// `TimeDuration`'s internal second value.
    pub seconds: f64,
    /// `TimeDuration`'s internal millisecond value.
    pub milliseconds: f64,
    /// `TimeDuration`'s internal microsecond value.
    pub microseconds: f64,
    /// `TimeDuration`'s internal nanosecond value.
    pub nanoseconds: f64,
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
        let mut days = 0;
        let mut hours = 0;
        let mut minutes = 0;
        let mut seconds = 0;
        let mut milliseconds = 0;
        let mut microseconds = 0;

        // 2. Let sign be NormalizedTimeDurationSign(norm).
        let sign = i32::from(norm.sign() as i8);
        // 3. Let nanoseconds be NormalizedTimeDurationAbs(norm).[[TotalNanoseconds]].
        let mut nanoseconds = norm.0.abs();

        match largest_unit {
            // 4. If largestUnit is "year", "month", "week", or "day", then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                (microseconds, nanoseconds) = nanoseconds.div_rem_euclid(&1_000);

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                (milliseconds, microseconds) = microseconds.div_rem_euclid(&1_000);

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                (seconds, milliseconds) = milliseconds.div_rem_euclid(&1_000);

                // g. Set minutes to floor(seconds / 60).
                // h. Set seconds to seconds modulo 60.
                (minutes, seconds) = seconds.div_rem_euclid(&60);

                // i. Set hours to floor(minutes / 60).
                // j. Set minutes to minutes modulo 60.
                (hours, minutes) = minutes.div_rem_euclid(&60);

                // k. Set days to floor(hours / 24).
                // l. Set hours to hours modulo 24.
                (days, hours) = hours.div_rem_euclid(&24);
            }
            // 5. Else if largestUnit is "hour", then
            TemporalUnit::Hour => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                (microseconds, nanoseconds) = nanoseconds.div_rem_euclid(&1_000);

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                (milliseconds, microseconds) = microseconds.div_rem_euclid(&1_000);

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                (seconds, milliseconds) = milliseconds.div_rem_euclid(&1_000);

                // g. Set minutes to floor(seconds / 60).
                // h. Set seconds to seconds modulo 60.
                (minutes, seconds) = seconds.div_rem_euclid(&60);

                // i. Set hours to floor(minutes / 60).
                // j. Set minutes to minutes modulo 60.
                (hours, minutes) = minutes.div_rem_euclid(&60);
            }
            // 6. Else if largestUnit is "minute", then
            TemporalUnit::Minute => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                (microseconds, nanoseconds) = nanoseconds.div_rem_euclid(&1_000);

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                (milliseconds, microseconds) = microseconds.div_rem_euclid(&1_000);

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                (seconds, milliseconds) = milliseconds.div_rem_euclid(&1_000);

                // g. Set minutes to floor(seconds / 60).
                // h. Set seconds to seconds modulo 60.
                (minutes, seconds) = seconds.div_rem_euclid(&60);
            }
            // 7. Else if largestUnit is "second", then
            TemporalUnit::Second => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                (microseconds, nanoseconds) = nanoseconds.div_rem_euclid(&1_000);

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                (milliseconds, microseconds) = microseconds.div_rem_euclid(&1_000);

                // e. Set seconds to floor(milliseconds / 1000).
                // f. Set milliseconds to milliseconds modulo 1000.
                (seconds, milliseconds) = milliseconds.div_rem_euclid(&1_000);
            }
            // 8. Else if largestUnit is "millisecond", then
            TemporalUnit::Millisecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                (microseconds, nanoseconds) = nanoseconds.div_rem_euclid(&1_000);

                // c. Set milliseconds to floor(microseconds / 1000).
                // d. Set microseconds to microseconds modulo 1000.
                (milliseconds, microseconds) = microseconds.div_rem_euclid(&1_000);
            }
            // 9. Else if largestUnit is "microsecond", then
            TemporalUnit::Microsecond => {
                // a. Set microseconds to floor(nanoseconds / 1000).
                // b. Set nanoseconds to nanoseconds modulo 1000.
                (microseconds, nanoseconds) = nanoseconds.div_rem_euclid(&1_000);
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

        // NOTE: days may have the potentially to exceed i64
        // 12. Return ! CreateTimeDurationRecord(days × sign, hours × sign, minutes × sign, seconds × sign, milliseconds × sign, microseconds × sign, nanoseconds × sign).
        let days = (days as i64).mul_add(sign.into(), 0);
        let result = Self::new_unchecked(
            (hours as i32).mul_add(sign, 0).into(),
            (minutes as i32).mul_add(sign, 0).into(),
            (seconds as i32).mul_add(sign, 0).into(),
            (milliseconds as i32).mul_add(sign, 0).into(),
            (microseconds as i32).mul_add(sign, 0).into(),
            (nanoseconds as i32).mul_add(sign, 0).into(),
        );

        // TODO: Stabilize casting and the value size.
        let td = Vec::from(&[
            days as f64,
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

        // TODO: Remove cast below.
        Ok((days as f64, result))
    }

    /// Returns this `TimeDuration` as a `NormalizedTimeDuration`.
    #[inline]
    pub(crate) fn to_normalized(self) -> NormalizedTimeDuration {
        NormalizedTimeDuration::from_time_duration(&self)
    }

    /// Returns the value of `TimeDuration`'s fields.
    #[inline]
    #[must_use]
    pub(crate) fn fields(&self) -> Vec<f64> {
        Vec::from(&[
            self.hours,
            self.minutes,
            self.seconds,
            self.milliseconds,
            self.microseconds,
            self.nanoseconds,
        ])
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
        if !is_valid_duration(&result.fields()) {
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
}

// ==== TimeDuration method impls ====

impl TimeDuration {
    // TODO: Maybe move to `NormalizedTimeDuration`
    pub(crate) fn round(
        days: f64,
        norm: &NormalizedTimeDuration,
        options: ResolvedRoundingOptions,
    ) -> TemporalResult<(NormalizedDurationRecord, Option<i128>)> {
        // 1. Assert: IsCalendarUnit(unit) is false.
        let (days, norm, total) = match options.smallest_unit {
            // 2. If unit is "day", then
            TemporalUnit::Day => {
                // a. Let fractionalDays be days + DivideNormalizedTimeDuration(norm, nsPerDay).
                let fractional_days = days + norm.as_fractional_days();
                // b. Set days to RoundNumberToIncrement(fractionalDays, increment, roundingMode).
                let days = IncrementRounder::from_potentially_negative_parts(
                    fractional_days,
                    options.increment.as_extended_increment(),
                )?
                .round(options.rounding_mode);
                // c. Let total be fractionalDays.
                // d. Set norm to ZeroTimeDuration().
                (
                    f64::from_i128(days).ok_or(
                        TemporalError::range().with_message("days exceeded a valid range."),
                    )?,
                    NormalizedTimeDuration::default(),
                    i128::from_f64(fractional_days),
                )
            }
            // 3. Else,
            TemporalUnit::Hour
            | TemporalUnit::Minute
            | TemporalUnit::Second
            | TemporalUnit::Millisecond
            | TemporalUnit::Microsecond
            | TemporalUnit::Nanosecond => {
                // a. Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains unit, is time.
                // b. Let divisor be the value in the "Length in Nanoseconds" column of the row of Table 22 whose "Singular" column contains unit.
                let divisor = options.smallest_unit.as_nanoseconds().temporal_unwrap()?;
                // c. Let total be DivideNormalizedTimeDuration(norm, divisor).
                let total = norm.divide(divisor as i64);
                let non_zero_divisor = unsafe { NonZeroU128::new_unchecked(divisor.into()) };
                // d. Set norm to ? RoundNormalizedTimeDurationToIncrement(norm, divisor × increment, roundingMode).
                let norm = norm.round(
                    non_zero_divisor
                        .checked_mul(options.increment.as_extended_increment())
                        .temporal_unwrap()?,
                    options.rounding_mode,
                )?;
                (days, norm, Some(total))
            }
            _ => return Err(TemporalError::assert()),
        };

        // 4. Return the Record { [[NormalizedDuration]]: ? CreateNormalizedDurationRecord(0, 0, 0, days, norm), [[Total]]: total  }.
        Ok((
            NormalizedDurationRecord::new(DateDuration::new(0.0, 0.0, 0.0, days)?, norm)?,
            total,
        ))
    }
}
