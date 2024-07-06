//! Implementation of a `DateDuration`

use crate::{
    components::{Date, Duration},
    options::{ArithmeticOverflow, TemporalUnit},
    Sign, TemporalError, TemporalResult, TemporalUnwrap,
};

/// `DateDuration` represents the [date duration record][spec] of the `Duration.`
///
/// These fields are laid out in the [Temporal Proposal][field spec] as 64-bit floating point numbers.
///
/// [spec]: https://tc39.es/proposal-temporal/#sec-temporal-date-duration-records
/// [field spec]: https://tc39.es/proposal-temporal/#sec-properties-of-temporal-duration-instances
#[non_exhaustive]
#[derive(Debug, Default, Clone, Copy)]
pub struct DateDuration {
    /// `DateDuration`'s internal year value.
    pub years: f64,
    /// `DateDuration`'s internal month value.
    pub months: f64,
    /// `DateDuration`'s internal week value.
    pub weeks: f64,
    /// `DateDuration`'s internal day value.
    pub days: f64,
}

impl DateDuration {
    /// Creates a new, non-validated `DateDuration`.
    #[inline]
    #[must_use]
    pub(crate) const fn new_unchecked(years: f64, months: f64, weeks: f64, days: f64) -> Self {
        Self {
            years,
            months,
            weeks,
            days,
        }
    }

    /// 7.5.38 BalanceDateDurationRelative ( years, months, weeks, days, largestUnit, smallestUnit, plainRelativeTo, calendarRec )
    pub fn balance_relative(
        &self,
        largest_unit: TemporalUnit,
        smallest_unit: TemporalUnit,
        plain_relative_to: Option<&Date>,
    ) -> TemporalResult<DateDuration> {
        // TODO: Confirm 1 or 5 based off response to issue.
        // 1. Assert: If plainRelativeTo is not undefined, calendarRec is not undefined.
        let plain_relative = plain_relative_to.temporal_unwrap()?;

        // 2. Let allZero be false.
        // 3. If years = 0, and months = 0, and weeks = 0, and days = 0, set allZero to true.
        let all_zero =
            self.years == 0.0 && self.months == 0.0 && self.weeks == 0.0 && self.days == 0.0;

        // 4. If largestUnit is not one of "year", "month", or "week", or allZero is true, then
        match largest_unit {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week if !all_zero => {}
            _ => {
                // a. Return ! CreateDateDurationRecord(years, months, weeks, days).
                return Ok(*self);
            }
        }

        // NOTE: See Step 1.
        // 5. If plainRelativeTo is undefined, then
        // a. Throw a RangeError exception.
        // 6. Assert: CalendarMethodsRecordHasLookedUp(calendarRec, DATE-ADD) is true.
        // 7. Assert: CalendarMethodsRecordHasLookedUp(calendarRec, DATE-UNTIL) is true.
        // 8. Let untilOptions be OrdinaryObjectCreate(null).
        // 9. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", largestUnit).

        match largest_unit {
            // 10. If largestUnit is "year", then
            TemporalUnit::Year => {
                // a. If smallestUnit is "week", then
                if smallest_unit == TemporalUnit::Week {
                    // i. Assert: days = 0.
                    // ii. Let yearsMonthsDuration be ! CreateTemporalDuration(years, months, 0, 0, 0, 0, 0, 0, 0, 0).
                    let years_months = Duration::from_date_duration(&Self::new_unchecked(
                        self.years,
                        self.months,
                        0.0,
                        0.0,
                    ));

                    // iii. Let later be ? AddDate(calendarRec, plainRelativeTo, yearsMonthsDuration).
                    let later = plain_relative.calendar().date_add(
                        plain_relative,
                        &years_months,
                        ArithmeticOverflow::Constrain,
                    )?;

                    // iv. Let untilResult be ? CalendarDateUntil(calendarRec, plainRelativeTo, later, untilOptions).
                    let until = plain_relative.calendar().date_until(
                        plain_relative,
                        &later,
                        largest_unit,
                    )?;

                    // v. Return ? CreateDateDurationRecord(untilResult.[[Years]], untilResult.[[Months]], weeks, 0).
                    return Self::new(until.years(), until.months(), self.weeks, 0.0);
                }

                // b. Let yearsMonthsWeeksDaysDuration be ! CreateTemporalDuration(years, months, weeks, days, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Duration::from_date_duration(self);

                // c. Let later be ? AddDate(calendarRec, plainRelativeTo, yearsMonthsWeeksDaysDuration).
                let later = plain_relative.calendar().date_add(
                    plain_relative,
                    &years_months_weeks,
                    ArithmeticOverflow::Constrain,
                )?;
                // d. Let untilResult be ? CalendarDateUntil(calendarRec, plainRelativeTo, later, untilOptions).
                let until =
                    plain_relative
                        .calendar()
                        .date_until(plain_relative, &later, largest_unit)?;
                // e. Return ! CreateDateDurationRecord(untilResult.[[Years]], untilResult.[[Months]], untilResult.[[Weeks]], untilResult.[[Days]]).
                Self::new(until.years(), until.months(), until.weeks(), until.days())
            }
            // 11. If largestUnit is "month", then
            TemporalUnit::Month => {
                // a. Assert: years = 0.
                // b. If smallestUnit is "week", then
                if smallest_unit == TemporalUnit::Week {
                    // i. Assert: days = 0.
                    // ii. Return ! CreateDateDurationRecord(0, months, weeks, 0).
                    return Self::new(0.0, self.months, self.weeks, 0.0);
                }

                // c. Let monthsWeeksDaysDuration be ! CreateTemporalDuration(0, months, weeks, days, 0, 0, 0, 0, 0, 0).
                let months_weeks_days = Duration::from_date_duration(&Self::new_unchecked(
                    0.0,
                    self.months,
                    self.weeks,
                    self.days,
                ));

                // d. Let later be ? AddDate(calendarRec, plainRelativeTo, monthsWeeksDaysDuration).
                let later = plain_relative.calendar().date_add(
                    plain_relative,
                    &months_weeks_days,
                    ArithmeticOverflow::Constrain,
                )?;

                // e. Let untilResult be ? CalendarDateUntil(calendarRec, plainRelativeTo, later, untilOptions).
                let until =
                    plain_relative
                        .calendar()
                        .date_until(plain_relative, &later, largest_unit)?;

                // f. Return ! CreateDateDurationRecord(0, untilResult.[[Months]], untilResult.[[Weeks]], untilResult.[[Days]]).
                Self::new(0.0, until.months(), until.weeks(), until.days())
            }
            // 12. Assert: largestUnit is "week".
            TemporalUnit::Week => {
                // 13. Assert: years = 0.
                // 14. Assert: months = 0.
                // 15. Let weeksDaysDuration be ! CreateTemporalDuration(0, 0, weeks, days, 0, 0, 0, 0, 0, 0).
                let weeks_days = Duration::from_date_duration(&Self::new_unchecked(
                    0.0, 0.0, self.weeks, self.days,
                ));

                // 16. Let later be ? AddDate(calendarRec, plainRelativeTo, weeksDaysDuration).
                let later = plain_relative.calendar().date_add(
                    plain_relative,
                    &weeks_days,
                    ArithmeticOverflow::Constrain,
                )?;

                // 17. Let untilResult be ? CalendarDateUntil(calendarRec, plainRelativeTo, later, untilOptions).
                let until =
                    plain_relative
                        .calendar()
                        .date_until(plain_relative, &later, largest_unit)?;

                // 18. Return ! CreateDateDurationRecord(0, 0, untilResult.[[Weeks]], untilResult.[[Days]]).
                Self::new(0.0, 0.0, until.weeks(), until.days())
            }
            _ => Err(TemporalError::general(
                "largestUnit in BalanceDateDurationRelative exceeded possible values.",
            )),
        }
    }

    /// Returns the iterator for `DateDuration`
    #[inline]
    #[must_use]
    pub(crate) fn fields(&self) -> Vec<f64> {
        Vec::from(&[self.years, self.months, self.weeks, self.days])
    }
}

impl DateDuration {
    /// Creates a new `DateDuration` with provided values.
    #[inline]
    pub fn new(years: f64, months: f64, weeks: f64, days: f64) -> TemporalResult<Self> {
        let result = Self::new_unchecked(years, months, weeks, days);
        if !super::is_valid_duration(&result.fields()) {
            return Err(TemporalError::range().with_message("Invalid DateDuration."));
        }
        Ok(result)
    }

    /// Returns a `PartialDateDuration` with all fields set to `NaN`.
    #[must_use]
    pub const fn partial() -> Self {
        Self {
            years: f64::NAN,
            months: f64::NAN,
            weeks: f64::NAN,
            days: f64::NAN,
        }
    }

    /// Creates a `DateDuration` from a provided partial `DateDuration`.
    #[must_use]
    pub fn from_partial(partial: &DateDuration) -> Self {
        Self {
            years: if partial.years.is_nan() {
                0.0
            } else {
                partial.years
            },
            months: if partial.months.is_nan() {
                0.0
            } else {
                partial.months
            },
            weeks: if partial.weeks.is_nan() {
                0.0
            } else {
                partial.weeks
            },
            days: if partial.days.is_nan() {
                0.0
            } else {
                partial.days
            },
        }
    }

    /// Returns a negated `DateDuration`.
    #[inline]
    #[must_use]
    pub fn negated(&self) -> Self {
        Self {
            years: self.years * -1.0,
            months: self.months * -1.0,
            weeks: self.weeks * -1.0,
            days: self.days * -1.0,
        }
    }

    /// Returns a new `DateDuration` representing the absolute value of the current.
    #[inline]
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            years: self.years.abs(),
            months: self.months.abs(),
            weeks: self.weeks.abs(),
            days: self.days.abs(),
        }
    }

    /// Returns the sign for the current `DateDuration`.
    #[inline]
    #[must_use]
    pub fn sign(&self) -> Sign {
        super::duration_sign(&self.fields())
    }
}
