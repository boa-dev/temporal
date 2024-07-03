//! Implementation of a `DateDuration`

use crate::{
    components::{duration::TimeDuration, Date, DateTime, Duration},
    options::{
        ArithmeticOverflow, RelativeTo, RoundingIncrement, TemporalRoundingMode, TemporalUnit,
    },
    rounding::{IncrementRounder, Round},
    TemporalError, TemporalResult, TemporalUnwrap,
};

use super::normalized::NormalizedTimeDuration;

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
    pub fn sign(&self) -> i32 {
        super::duration_sign(&self.fields())
    }
}

// ==== DateDuration Operations ====

impl DateDuration {
    /// Rounds the current `DateDuration` returning a tuple of the rounded `DateDuration` and
    /// the `total` value of the smallest unit prior to rounding.
    #[allow(
        clippy::type_complexity,
        clippy::let_and_return,
        clippy::too_many_arguments
    )]
    pub fn round(
        &self,
        normalized_time: Option<NormalizedTimeDuration>,
        increment: RoundingIncrement,
        unit: TemporalUnit,
        rounding_mode: TemporalRoundingMode,
        relative_to: &RelativeTo,
        _precalculated_dt: Option<DateTime>,
    ) -> TemporalResult<(Self, f64)> {
        // 1. If plainRelativeTo is not present, set plainRelativeTo to undefined.
        let plain_relative_to = relative_to.date;
        // 2. If zonedRelativeTo is not present, set zonedRelativeTo to undefined.
        let zoned_relative_to = relative_to.zdt;
        // 3. If precalculatedPlainDateTime is not present, set precalculatedPlainDateTime to undefined.

        let mut fractional_days = match unit {
            // 4. If unit is "year", "month", or "week", and plainRelativeTo is undefined, then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week
                if plain_relative_to.is_none() =>
            {
                // a. Throw a RangeError exception.
                return Err(TemporalError::range()
                    .with_message("plainRelativeTo canot be undefined with given TemporalUnit"));
            }
            // 5. If unit is one of "year", "month", "week", or "day", then
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                // a. If zonedRelativeTo is not undefined, then
                if let Some(_zoned_relative) = zoned_relative_to {
                    // TODO:
                    // i. Let intermediate be ? MoveRelativeZonedDateTime(zonedRelativeTo, calendarRec, timeZoneRec, years, months, weeks, days, precalculatedPlainDateTime).
                    // ii. Let result be ? NormalizedTimeDurationToDays(norm, intermediate, timeZoneRec).
                    // iii. Let fractionalDays be days + result.[[Days]] + DivideNormalizedTimeDuration(result.[[Remainder]], result.[[DayLength]]).
                    return Err(TemporalError::general("Not yet implemented."));
                // b. Else,
                } else {
                    // TODO: fix the below cast
                    // i. Let fractionalDays be days + DivideNormalizedTimeDuration(norm, nsPerDay).
                    self.days + normalized_time.unwrap_or_default().as_fractional_days()
                }
                // c. Set days to 0.
            }
            _ => {
                return Err(TemporalError::range()
                    .with_message("Invalid TemporalUnit provided to DateDuration.round"))
            }
        };
        // 7. let total be unset.
        // We begin matching against unit and return the remainder value.
        match unit {
            // 8. If unit is "year", then
            TemporalUnit::Year => {
                let plain_relative_to = plain_relative_to.expect("this must exist.");
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let calendar = plain_relative_to.calendar();

                // b. Let yearsDuration be ! CreateTemporalDuration(years, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years = DateDuration::new_unchecked(self.years, 0.0, 0.0, 0.0);
                let years_duration = Duration::new_unchecked(years, TimeDuration::default());

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsLater be ? AddDate(calendar, plainRelativeTo, yearsDuration, undefined, dateAdd).
                let years_later = plain_relative_to.add_date(&years_duration, None)?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Duration::new_unchecked(
                    Self::new_unchecked(self.years, self.months, self.weeks, 0.0),
                    TimeDuration::default(),
                );

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later =
                    plain_relative_to.add_date(&years_months_weeks, None)?;

                // h. Let monthsWeeksInDays be DaysUntil(yearsLater, yearsMonthsWeeksLater).
                let months_weeks_in_days = years_later.days_until(&years_months_weeks_later);

                // i. Set plainRelativeTo to yearsLater.
                let plain_relative_to = years_later;

                // j. Set fractionalDays to fractionalDays + monthsWeeksInDays.
                fractional_days += f64::from(months_weeks_in_days);

                // k. Let isoResult be ! AddISODate(plainRelativeTo.[[ISOYear]]. plainRelativeTo.[[ISOMonth]], plainRelativeTo.[[ISODay]], 0, 0, 0, truncate(fractionalDays), "constrain").
                let iso_result = plain_relative_to.iso.add_date_duration(
                    &DateDuration::new_unchecked(0.0, 0.0, 0.0, fractional_days.trunc()),
                    ArithmeticOverflow::Constrain,
                )?;

                // l. Let wholeDaysLater be ? CreateDate(isoResult.[[Year]], isoResult.[[Month]], isoResult.[[Day]], calendar).
                let whole_days_later = Date::new_unchecked(iso_result, calendar.clone());

                // m. Let untilOptions be OrdinaryObjectCreate(null).
                // n. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "year").
                // o. Let timePassed be ? DifferenceDate(calendar, plainRelativeTo, wholeDaysLater, untilOptions).
                let time_passed =
                    plain_relative_to.internal_diff_date(&whole_days_later, TemporalUnit::Year)?;

                // p. Let yearsPassed be timePassed.[[Years]].
                let years_passed = time_passed.date.years;

                // q. Set years to years + yearsPassed.
                let years = self.years + years_passed;

                // r. Let yearsDuration be ! CreateTemporalDuration(yearsPassed, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_duration = Duration::one_year(years_passed);

                // s. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, yearsDuration, dateAdd).
                // t. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // u. Let daysPassed be moveResult.[[Days]].
                let (plain_relative_to, days_passed) =
                    plain_relative_to.move_relative_date(&years_duration)?;

                // v. Set fractionalDays to fractionalDays - daysPassed.
                fractional_days -= days_passed;

                // w. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if fractional_days < 0.0 { -1 } else { 1 };

                // x. Let oneYear be ! CreateTemporalDuration(sign, 0, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_year = Duration::one_year(f64::from(sign));

                // y. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneYear, dateAdd).
                // z. Let oneYearDays be moveResult.[[Days]].
                let (_, one_year_days) = plain_relative_to.move_relative_date(&one_year)?;

                if one_year_days == 0.0 {
                    return Err(TemporalError::range().with_message("oneYearDays exceeds ranges."));
                }
                // aa. Let fractionalYears be years + fractionalDays / abs(oneYearDays).
                let frac_years = years + (fractional_days / one_year_days.abs());

                // ab. Set years to RoundNumberToIncrement(fractionalYears, increment, roundingMode).
                let rounded_years = IncrementRounder::<f64>::from_potentially_negative_parts(
                    frac_years,
                    increment.as_extended_increment(),
                )?
                .round(rounding_mode);

                // ac. Set total to fractionalYears.
                // ad. Set months and weeks to 0.
                let result = Self::new(rounded_years as f64, 0f64, 0f64, 0f64)?;
                Ok((result, frac_years))
            }
            // 9. Else if unit is "month", then
            TemporalUnit::Month => {
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("this must exist.");

                // b. Let yearsMonths be ! CreateTemporalDuration(years, months, 0, 0, 0, 0, 0, 0, 0, 0).
                let years_months = Duration::from_date_duration(&DateDuration::new_unchecked(
                    self.years,
                    self.months,
                    0.0,
                    0.0,
                ));

                // c. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // d. Else,
                // i. Let dateAdd be unused.

                // e. Let yearsMonthsLater be ? AddDate(calendar, plainRelativeTo, yearsMonths, undefined, dateAdd).
                let years_months_later = plain_relative_to.add_date(&years_months, None)?;

                // f. Let yearsMonthsWeeks be ! CreateTemporalDuration(years, months, weeks, 0, 0, 0, 0, 0, 0, 0).
                let years_months_weeks = Duration::from_date_duration(
                    &DateDuration::new_unchecked(self.years, self.months, self.weeks, 0.0),
                );

                // g. Let yearsMonthsWeeksLater be ? AddDate(calendar, plainRelativeTo, yearsMonthsWeeks, undefined, dateAdd).
                let years_months_weeks_later =
                    plain_relative_to.add_date(&years_months_weeks, None)?;

                // h. Let weeksInDays be DaysUntil(yearsMonthsLater, yearsMonthsWeeksLater).
                let weeks_in_days = years_months_later.days_until(&years_months_weeks_later);

                // i. Set plainRelativeTo to yearsMonthsLater.
                let plain_relative_to = years_months_later;

                // j. Set fractionalDays to fractionalDays + weeksInDays.
                fractional_days += f64::from(weeks_in_days);

                // k. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if fractional_days < 0.0 { -1f64 } else { 1f64 };

                // l. Let oneMonth be ! CreateTemporalDuration(0, sign, 0, 0, 0, 0, 0, 0, 0, 0).
                let one_month = Duration::one_month(sign);

                // m. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                // n. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // o. Let oneMonthDays be moveResult.[[Days]].
                let (mut plain_relative_to, mut one_month_days) =
                    plain_relative_to.move_relative_date(&one_month)?;

                let mut months = self.months;
                // p. Repeat, while abs(fractionalDays) ≥ abs(oneMonthDays),
                while fractional_days.abs() >= one_month_days.abs() {
                    // i. Set months to months + sign.
                    months += sign;

                    // ii. Set fractionalDays to fractionalDays - oneMonthDays.
                    fractional_days -= one_month_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneMonth, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_month)?;

                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneMonthDays to moveResult.[[Days]].
                    one_month_days = move_result.1;
                }

                // q. Let fractionalMonths be months + fractionalDays / abs(oneMonthDays).
                let frac_months = months + fractional_days / one_month_days.abs();

                // r. Set months to RoundNumberToIncrement(fractionalMonths, increment, roundingMode).
                let rounded_months = IncrementRounder::<f64>::from_potentially_negative_parts(
                    frac_months,
                    increment.as_extended_increment(),
                )?
                .round(rounding_mode);

                // s. Set total to fractionalMonths.
                // t. Set weeks to 0.
                let result = Self::new(self.years, rounded_months as f64, 0f64, 0f64)?;
                Ok((result, frac_months))
            }
            // 10. Else if unit is "week", then
            TemporalUnit::Week => {
                // a. Let calendar be plainRelativeTo.[[Calendar]].
                let plain_relative_to = plain_relative_to.expect("date must exist given Week");

                // b. If fractionalDays < 0, let sign be -1; else, let sign be 1.
                let sign = if fractional_days < 0.0 { -1f64 } else { 1f64 };

                // c. Let oneWeek be ! CreateTemporalDuration(0, 0, sign, 0, 0, 0, 0, 0, 0, 0).
                let one_week = Duration::one_week(sign);

                // d. If calendar is an Object, then
                // i. Let dateAdd be ? GetMethod(calendar, "dateAdd").
                // e. Else,
                // i. Let dateAdd be unused.

                // f. Let moveResult be ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
                // g. Set plainRelativeTo to moveResult.[[RelativeTo]].
                // h. Let oneWeekDays be moveResult.[[Days]].
                let (mut plain_relative_to, mut one_week_days) =
                    plain_relative_to.move_relative_date(&one_week)?;

                let mut weeks = self.weeks;
                // i. Repeat, while abs(fractionalDays) ≥ abs(oneWeekDays),
                while fractional_days.abs() >= one_week_days.abs() {
                    // i. Set weeks to weeks + sign.
                    weeks += sign;

                    // ii. Set fractionalDays to fractionalDays - oneWeekDays.
                    fractional_days -= one_week_days;

                    // iii. Set moveResult to ? MoveRelativeDate(calendar, plainRelativeTo, oneWeek, dateAdd).
                    let move_result = plain_relative_to.move_relative_date(&one_week)?;

                    // iv. Set plainRelativeTo to moveResult.[[RelativeTo]].
                    plain_relative_to = move_result.0;
                    // v. Set oneWeekDays to moveResult.[[Days]].
                    one_week_days = move_result.1;
                }

                // j. Let fractionalWeeks be weeks + fractionalDays / abs(oneWeekDays).
                let frac_weeks = weeks + fractional_days / one_week_days.abs();

                // k. Set weeks to RoundNumberToIncrement(fractionalWeeks, increment, roundingMode).
                let rounded_weeks = IncrementRounder::<f64>::from_potentially_negative_parts(
                    frac_weeks,
                    increment.as_extended_increment(),
                )?
                .round(rounding_mode);
                // l. Set total to fractionalWeeks.
                let result = Self::new(self.years, self.months, rounded_weeks as f64, 0f64)?;
                Ok((result, frac_weeks))
            }
            // 11. Else if unit is "day", then
            TemporalUnit::Day => {
                // a. Set days to RoundNumberToIncrement(fractionalDays, increment, roundingMode).
                let rounded_days = IncrementRounder::<f64>::from_potentially_negative_parts(
                    fractional_days,
                    increment.as_extended_increment(),
                )?
                .round(rounding_mode);

                // b. Set total to fractionalDays.
                // c. Set norm to ZeroTimeDuration().
                let result = Self::new(self.years, self.months, self.weeks, rounded_days as f64)?;
                Ok((result, fractional_days))
            }
            _ => unreachable!("All other TemporalUnits were returned early as invalid."),
        }
    }
}
