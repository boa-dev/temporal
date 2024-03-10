//! This module implements `Duration` along with it's methods and components.

use crate::{
    components::DateTime,
    options::{RelativeTo, TemporalRoundingMode, TemporalUnit},
    parser::{duration::parse_duration, Cursor},
    utils::{self, validate_temporal_rounding_increment},
    TemporalError, TemporalResult,
};
use std::str::FromStr;

use self::normalized::{NormalizedDurationRecord, NormalizedTimeDuration};

use super::{calendar::CalendarProtocol, tz::TzProtocol};

mod date;
pub(crate) mod normalized;
mod time;

#[cfg(test)]
mod tests;

#[doc(inline)]
pub use date::DateDuration;
#[doc(inline)]
pub use time::TimeDuration;

/// The native Rust implementation of `Temporal.Duration`.
///
/// `Duration` is made up of a `DateDuration` and `TimeDuration` as primarily
/// defined by Abtract Operation 7.5.1-5.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default)]
pub struct Duration {
    date: DateDuration,
    time: TimeDuration,
}

// NOTE(nekevss): Structure of the below is going to be a little convoluted,
// but intended to section everything based on the below
//
// Notation - [section](sub-section(s)).
//
// Sections:
//   - Creation (private/public)
//   - Getters/Setters
//   - Methods (private/public/feature)
//

// ==== Private Creation methods ====

impl Duration {
    /// Creates a new `Duration` from a `DateDuration` and `TimeDuration`.
    #[inline]
    pub(crate) const fn new_unchecked(date: DateDuration, time: TimeDuration) -> Self {
        Self { date, time }
    }

    /// Utility function to create a year duration.
    #[inline]
    pub(crate) fn one_year(year_value: f64) -> Self {
        Self::from_date_duration(&DateDuration::new_unchecked(year_value, 1f64, 0f64, 0f64))
    }

    /// Utility function to create a month duration.
    #[inline]
    pub(crate) fn one_month(month_value: f64) -> Self {
        Self::from_date_duration(&DateDuration::new_unchecked(0f64, month_value, 0f64, 0f64))
    }

    /// Utility function to create a week duration.
    #[inline]
    pub(crate) fn one_week(week_value: f64) -> Self {
        Self::from_date_duration(&DateDuration::new_unchecked(0f64, 0f64, week_value, 0f64))
    }

    /// Returns the a `Vec` of the fields values.
    #[inline]
    #[must_use]
    pub(crate) fn fields(&self) -> Vec<f64> {
        Vec::from(&[
            self.years(),
            self.months(),
            self.weeks(),
            self.days(),
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.milliseconds(),
            self.microseconds(),
            self.nanoseconds(),
        ])
    }

    /// Returns whether `Duration`'s `DateDuration` is empty and is therefore a `TimeDuration`.
    #[inline]
    #[must_use]
    pub(crate) fn is_time_duration(&self) -> bool {
        self.time().fields().iter().any(|x| x != &0.0)
            && self.date().fields().iter().all(|x| x == &0.0)
    }

    /// Returns the `TemporalUnit` corresponding to the largest non-zero field.
    #[inline]
    pub(crate) fn default_largest_unit(&self) -> TemporalUnit {
        self.fields()
            .iter()
            .enumerate()
            .find(|x| x.1 != &0.0)
            .map(|x| TemporalUnit::from(10 - x.0))
            .unwrap_or(TemporalUnit::Nanosecond)
    }
}

// ==== Public Duration API ====

impl Duration {
    /// Creates a new validated `Duration`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        years: f64,
        months: f64,
        weeks: f64,
        days: f64,
        hours: f64,
        minutes: f64,
        seconds: f64,
        milliseconds: f64,
        microseconds: f64,
        nanoseconds: f64,
    ) -> TemporalResult<Self> {
        let duration = Self::new_unchecked(
            DateDuration::new_unchecked(years, months, weeks, days),
            TimeDuration::new_unchecked(
                hours,
                minutes,
                seconds,
                milliseconds,
                microseconds,
                nanoseconds,
            ),
        );
        if !is_valid_duration(&duration.fields()) {
            return Err(TemporalError::range().with_message("Duration was not valid."));
        }
        Ok(duration)
    }

    /// Creates a partial `Duration` with all fields set to `NaN`.
    #[must_use]
    pub const fn partial() -> Self {
        Self {
            date: DateDuration::partial(),
            time: TimeDuration::partial(),
        }
    }

    /// Creates a `Duration` from only a `DateDuration`.
    #[must_use]
    pub fn from_date_duration(date: &DateDuration) -> Self {
        Self {
            date: *date,
            time: TimeDuration::default(),
        }
    }

    /// Creates a `Duration` from a provided a day and a `TimeDuration`.
    ///
    /// Note: `TimeDuration` records can store a day value to deal with overflow.
    #[must_use]
    pub fn from_day_and_time(day: f64, time: &TimeDuration) -> Self {
        Self {
            date: DateDuration::new_unchecked(0.0, 0.0, 0.0, day),
            time: *time,
        }
    }

    /// Creates a new valid `Duration` from a partial `Duration`.
    pub fn from_partial(partial: &Duration) -> TemporalResult<Self> {
        let duration = Self {
            date: DateDuration::from_partial(partial.date()),
            time: TimeDuration::from_partial(partial.time()),
        };
        if !is_valid_duration(&duration.fields()) {
            return Err(TemporalError::range().with_message("Duration was not valid."));
        }
        Ok(duration)
    }

    /// Return if the Durations values are within their valid ranges.
    #[inline]
    #[must_use]
    pub fn is_time_within_range(&self) -> bool {
        self.time.is_within_range()
    }
}

// ==== Public `Duration` Getters/Setters ====

impl Duration {
    /// Returns a reference to the inner `TimeDuration`
    #[inline]
    #[must_use]
    pub fn time(&self) -> &TimeDuration {
        &self.time
    }

    /// Returns a reference to the inner `DateDuration`
    #[inline]
    #[must_use]
    pub fn date(&self) -> &DateDuration {
        &self.date
    }

    /// Set this `DurationRecord`'s `TimeDuration`.
    #[inline]
    pub fn set_time_duration(&mut self, time: TimeDuration) {
        self.time = time;
    }

    /// Set the value for `years`.
    #[inline]
    pub fn set_years(&mut self, y: f64) {
        self.date.years = y;
    }

    /// Returns the `years` field of duration.
    #[inline]
    #[must_use]
    pub const fn years(&self) -> f64 {
        self.date.years
    }

    /// Set the value for `months`.
    #[inline]
    pub fn set_months(&mut self, mo: f64) {
        self.date.months = mo;
    }

    /// Returns the `months` field of duration.
    #[inline]
    #[must_use]
    pub const fn months(&self) -> f64 {
        self.date.months
    }

    /// Set the value for `weeks`.
    #[inline]
    pub fn set_weeks(&mut self, w: f64) {
        self.date.weeks = w;
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn weeks(&self) -> f64 {
        self.date.weeks
    }

    /// Set the value for `days`.
    #[inline]
    pub fn set_days(&mut self, d: f64) {
        self.date.days = d;
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn days(&self) -> f64 {
        self.date.days
    }

    /// Set the value for `hours`.
    #[inline]
    pub fn set_hours(&mut self, h: f64) {
        self.time.hours = h;
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn hours(&self) -> f64 {
        self.time.hours
    }

    /// Set the value for `minutes`.
    #[inline]
    pub fn set_minutes(&mut self, m: f64) {
        self.time.minutes = m;
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn minutes(&self) -> f64 {
        self.time.minutes
    }

    /// Set the value for `seconds`.
    #[inline]
    pub fn set_seconds(&mut self, s: f64) {
        self.time.seconds = s;
    }

    /// Returns the `seconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn seconds(&self) -> f64 {
        self.time.seconds
    }

    /// Set the value for `milliseconds`.
    #[inline]
    pub fn set_milliseconds(&mut self, ms: f64) {
        self.time.milliseconds = ms;
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn milliseconds(&self) -> f64 {
        self.time.milliseconds
    }

    /// Set the value for `microseconds`.
    #[inline]
    pub fn set_microseconds(&mut self, mis: f64) {
        self.time.microseconds = mis;
    }

    /// Returns the `microseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn microseconds(&self) -> f64 {
        self.time.microseconds
    }

    /// Set the value for `nanoseconds`.
    #[inline]
    pub fn set_nanoseconds(&mut self, ns: f64) {
        self.time.nanoseconds = ns;
    }

    /// Returns the `nanoseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn nanoseconds(&self) -> f64 {
        self.time.nanoseconds
    }
}

// ==== Private Duration methods ====

impl Duration {
    // TODO (nekevss): Build out `RelativeTo` handling
    /// Abstract Operation 7.5.26 `RoundDuration ( years, months, weeks, days, hours, minutes,
    ///   seconds, milliseconds, microseconds, nanoseconds, increment, unit,
    ///   roundingMode [ , plainRelativeTo [, zonedRelativeTo [, precalculatedDateTime]]] )`
    #[allow(clippy::type_complexity)]
    pub(crate) fn round_internal<C: CalendarProtocol, Z: TzProtocol>(
        &self,
        increment: u64,
        unit: TemporalUnit,
        rounding_mode: TemporalRoundingMode,
        relative_to: &RelativeTo<C, Z>,
        precalculated_dt: Option<DateTime<C>>,
        context: &mut C::Context,
    ) -> TemporalResult<(NormalizedDurationRecord, f64)> {
        match unit {
            TemporalUnit::Year | TemporalUnit::Month | TemporalUnit::Week | TemporalUnit::Day => {
                let round_result = self.date().round(
                    Some(self.time.to_normalized()),
                    increment,
                    unit,
                    rounding_mode,
                    relative_to,
                    precalculated_dt,
                    context,
                )?;
                let norm_record = NormalizedDurationRecord::new(
                    round_result.0,
                    NormalizedTimeDuration::default(),
                )?;
                Ok((norm_record, round_result.1))
            }
            TemporalUnit::Hour
            | TemporalUnit::Minute
            | TemporalUnit::Second
            | TemporalUnit::Millisecond
            | TemporalUnit::Microsecond
            | TemporalUnit::Nanosecond => {
                let round_result = self.time().round(increment, unit, rounding_mode)?;
                let norm = NormalizedDurationRecord::new(*self.date(), round_result.0)?;
                Ok((norm, round_result.1 as f64))
            }
            TemporalUnit::Auto => {
                Err(TemporalError::range().with_message("Invalid TemporalUnit for Duration.round"))
            }
        }
        // 18. Let duration be ? CreateDurationRecord(years, months, weeks, days, hours,
        // minutes, seconds, milliseconds, microseconds, nanoseconds).
        // 19. Return the Record { [[DurationRecord]]: duration, [[Total]]: total }.
    }
}

// ==== Public Duration methods ====

impl Duration {
    /// Determines the sign for the current self.
    #[inline]
    #[must_use]
    pub fn sign(&self) -> i32 {
        duration_sign(&self.fields())
    }

    /// Returns whether the current `Duration` is zero.
    ///
    /// Equivalant to `Temporal.Duration.blank()`.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.sign() == 0
    }

    /// Returns a negated `Duration`
    #[inline]
    #[must_use]
    pub fn negated(&self) -> Self {
        Self {
            date: self.date().negated(),
            time: self.time().negated(),
        }
    }

    /// Returns the absolute value of `Duration`.
    #[inline]
    #[must_use]
    pub fn abs(&self) -> Self {
        Self {
            date: self.date().abs(),
            time: self.time().abs(),
        }
    }

    /// Rounds the current `Duration`.
    #[inline]
    pub fn round<C: CalendarProtocol, Z: TzProtocol>(
        &self,
        increment: Option<f64>,
        smallest_unit: Option<TemporalUnit>,
        largest_unit: Option<TemporalUnit>,
        rounding_mode: Option<TemporalRoundingMode>,
        relative_to: &RelativeTo<C, Z>,
        context: &mut C::Context,
    ) -> TemporalResult<Self> {
        // NOTE: Steps 1-14 seem to be implementation specific steps.

        // 22. If smallestUnitPresent is false and largestUnitPresent is false, then
        if largest_unit.is_none() && smallest_unit.is_none() {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range()
                .with_message("smallestUnit and largestUnit cannot both be None."));
        }

        // 14. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let increment = utils::to_rounding_increment(increment)?;
        // 15. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let mode = rounding_mode.unwrap_or_default();

        // 16. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", DATETIME, undefined).
        // 17. If smallestUnit is undefined, then
        // a. Set smallestUnitPresent to false.
        // b. Set smallestUnit to "nanosecond".
        let smallest_unit = smallest_unit.unwrap_or(TemporalUnit::Nanosecond);

        // 18. Let existingLargestUnit be ! DefaultTemporalLargestUnit(duration.[[Years]],
        // duration.[[Months]], duration.[[Weeks]], duration.[[Days]], duration.[[Hours]],
        // duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]],
        // duration.[[Microseconds]]).
        let existing_largest_unit = self.default_largest_unit();

        // 19. Let defaultLargestUnit be LargerOfTwoTemporalUnits(existingLargestUnit, smallestUnit).
        let default_largest = existing_largest_unit.max(smallest_unit);

        // 20. If largestUnit is undefined, then
        // a. Set largestUnitPresent to false.
        // b. Set largestUnit to defaultLargestUnit.
        // 21. Else if largestUnit is "auto", then
        // a. Set largestUnit to defaultLargestUnit.
        let largest_unit = match largest_unit {
            Some(TemporalUnit::Auto) | None => default_largest,
            Some(unit) => unit,
        };

        // 23. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        if largest_unit.max(smallest_unit) != largest_unit {
            return Err(TemporalError::range().with_message(
                "largestUnit when rounding Duration was not the largest provided unit",
            ));
        }

        // 24. Let maximum be MaximumTemporalDurationRoundingIncrement(smallestUnit).
        let maximum = smallest_unit.to_maximum_rounding_increment();
        // 25. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        if let Some(max) = maximum {
            validate_temporal_rounding_increment(increment, max.into(), false)?;
        }

        // 26. Let hoursToDaysConversionMayOccur be false.
        // 27. If duration.[[Days]] ≠ 0 and zonedRelativeTo is not undefined, set hoursToDaysConversionMayOccur to true.
        // 28. Else if abs(duration.[[Hours]]) ≥ 24, set hoursToDaysConversionMayOccur to true.
        let hours_to_days_may_occur =
            (self.days() != 0.0 && relative_to.zdt.is_some()) || self.hours().abs() >= 24.0;

        // 29. If smallestUnit is "nanosecond" and roundingIncrement = 1, let roundingGranularityIsNoop
        // be true; else let roundingGranularityIsNoop be false.
        let is_noop = smallest_unit == TemporalUnit::Nanosecond && increment == 1;
        // 30. If duration.[[Years]] = 0 and duration.[[Months]] = 0 and duration.[[Weeks]] = 0,
        // let calendarUnitsPresent be false; else let calendarUnitsPresent be true.
        let calendar_units_present =
            !(self.years() == 0.0 && self.months() == 0.0 && self.weeks() == 0.0);

        // 31. If roundingGranularityIsNoop is true, and largestUnit is existingLargestUnit, and calendarUnitsPresent is false,
        // and hoursToDaysConversionMayOccur is false, and abs(duration.[[Minutes]]) < 60, and abs(duration.[[Seconds]]) < 60,
        // and abs(duration.[[Milliseconds]]) < 1000, and abs(duration.[[Microseconds]]) < 1000, and abs(duration.[[Nanoseconds]]) < 1000, then
        if is_noop
            && largest_unit == existing_largest_unit
            && !calendar_units_present
            && !hours_to_days_may_occur
            && self.minutes().abs() < 60.0
            && self.seconds().abs() < 60.0
            && self.milliseconds() < 1000.0
            && self.microseconds() < 1000.0
            && self.nanoseconds() < 1000.0
        {
            // a. NOTE: The above conditions mean that the operation will have no effect: the
            // smallest unit and rounding increment will leave the total duration unchanged,
            // and it can be determined without calling a calendar or time zone method that
            // no balancing will take place.
            // b. Return ! CreateTemporalDuration(duration.[[Years]], duration.[[Months]],
            // duration.[[Weeks]], duration.[[Days]], duration.[[Hours]], duration.[[Minutes]],
            // duration.[[Seconds]], duration.[[Milliseconds]], duration.[[Microseconds]],
            // duration.[[Nanoseconds]]).
            return Ok(*self);
        }

        // 32. Let precalculatedPlainDateTime be undefined.
        // 33. If roundingGranularityIsNoop is false, or IsCalendarUnit(largestUnit) is true, or largestUnit is "day",
        // or calendarUnitsPresent is true, or duration.[[Days]] ≠ 0, let plainDateTimeOrRelativeToWillBeUsed be true;
        // else let plainDateTimeOrRelativeToWillBeUsed be false.
        let pdtr_will_be_used = !is_noop
            || largest_unit.is_calendar_unit()
            || largest_unit == TemporalUnit::Day
            || calendar_units_present
            || self.days() == 0.0;

        // 34. If zonedRelativeTo is not undefined and plainDateTimeOrRelativeToWillBeUsed is true, then
        let precalculated = if relative_to.zdt.is_some() && pdtr_will_be_used {
            return Err(TemporalError::general("Not yet implemented."));
            // a. NOTE: The above conditions mean that the corresponding Temporal.PlainDateTime or
            // Temporal.PlainDate for zonedRelativeTo will be used in one of the operations below.
            // b. Let instant be ! CreateTemporalInstant(zonedRelativeTo.[[Nanoseconds]]).
            // c. Set precalculatedPlainDateTime to ? GetPlainDateTimeFor(timeZoneRec, instant, zonedRelativeTo.[[Calendar]]).
            // d. Set plainRelativeTo to ! CreateTemporalDate(precalculatedPlainDateTime.[[ISOYear]],
            // precalculatedPlainDateTime.[[ISOMonth]], precalculatedPlainDateTime.[[ISODay]], zonedRelativeTo.[[Calendar]]).
        } else {
            None
        };
        // 35. Let calendarRec be ? CreateCalendarMethodsRecordFromRelativeTo(plainRelativeTo, zonedRelativeTo, « DATE-ADD, DATE-UNTIL »).

        // let relative_to_date = relative_to.date;

        let (round_result, _) = if let Some(relative_to_date) = relative_to.date {
            // 36. Let unbalanceResult be ? UnbalanceDateDurationRelative(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], largestUnit, plainRelativeTo, calendarRec).
            let unbalanced =
                self.date()
                    .unbalance_relative(largest_unit, Some(relative_to_date), context)?;

            // NOTE: Step 37 handled in round duration
            // 37. Let norm be NormalizeTimeDuration(duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]],
            // duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]]).
            // 38. Let roundRecord be ? RoundDuration(unbalanceResult.[[Years]], unbalanceResult.[[Months]],
            // unbalanceResult.[[Weeks]], unbalanceResult.[[Days]], norm, roundingIncrement, smallestUnit,
            // roundingMode, plainRelativeTo, calendarRec, zonedRelativeTo, timeZoneRec, precalculatedPlainDateTime).
            Self::new_unchecked(unbalanced, *self.time())
        } else {
            *self
        }
        .round_internal(
            increment,
            smallest_unit,
            mode,
            relative_to,
            precalculated,
            context,
        )?;

        // 39. Let roundResult be roundRecord.[[NormalizedDuration]].
        // 40. If zonedRelativeTo is not undefined, then
        let balance_result = if relative_to.zdt.is_some() {
            return Err(TemporalError::general("Not yet implemented."));
            // a. Set roundResult to ? AdjustRoundedDurationDays(roundResult.[[Years]], roundResult.[[Months]],
            // roundResult.[[Weeks]], roundResult.[[Days]], roundResult.[[NormalizedTime]], roundingIncrement,
            // smallestUnit, roundingMode, zonedRelativeTo, calendarRec, timeZoneRec, precalculatedPlainDateTime).
            // b. Let balanceResult be ? BalanceTimeDurationRelative(roundResult.[[Days]],
            // roundResult.[[NormalizedTime]], largestUnit, zonedRelativeTo, timeZoneRec, precalculatedPlainDateTime).
            // 41. Else,
        } else {
            // NOTE: DateDuration::round will always return a NormalizedTime::default as per spec.
            // a. Let normWithDays be ? Add24HourDaysToNormalizedTimeDuration(roundResult.[[NormalizedTime]], roundResult.[[Days]]).
            let norm_with_days = round_result.0 .1.add_days(round_result.0 .0.days)?;
            // b. Let balanceResult be BalanceTimeDuration(normWithDays, largestUnit).
            TimeDuration::from_normalized(norm_with_days, largest_unit)?
        };

        // 42. Let result be ? BalanceDateDurationRelative(roundResult.[[Years]],
        // roundResult.[[Months]], roundResult.[[Weeks]], balanceResult.[[Days]],
        // largestUnit, smallestUnit, plainRelativeTo, calendarRec).
        let intermediate = DateDuration::new_unchecked(
            round_result.0 .0.years,
            round_result.0 .0.months,
            round_result.0 .0.weeks,
            balance_result.0,
        );
        let result = if let Some(relative_to_date) = relative_to.date {
            intermediate.balance_relative(
                largest_unit,
                smallest_unit,
                Some(relative_to_date),
                context,
            )?
        } else {
            intermediate
        };

        // 43. Return ! CreateTemporalDuration(result.[[Years]], result.[[Months]],
        // result.[[Weeks]], result.[[Days]], balanceResult.[[Hours]], balanceResult.[[Minutes]],
        // balanceResult.[[Seconds]], balanceResult.[[Milliseconds]], balanceResult.[[Microseconds]],
        // balanceResult.[[Nanoseconds]]).
        Self::new(
            result.years,
            result.months,
            result.weeks,
            result.days,
            balance_result.1.hours,
            balance_result.1.minutes,
            balance_result.1.seconds,
            balance_result.1.milliseconds,
            balance_result.1.microseconds,
            balance_result.1.nanoseconds,
        )
    }
}

/// Utility function to check whether the `Duration` fields are valid.
#[inline]
#[must_use]
pub(crate) fn is_valid_duration(set: &Vec<f64>) -> bool {
    // 1. Let sign be ! DurationSign(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    let sign = duration_sign(set);
    // 2. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in set {
        // a. If 𝔽(v) is not finite, return false.
        if !v.is_finite() {
            return false;
        }
        // b. If v < 0 and sign > 0, return false.
        if *v < 0f64 && sign > 0 {
            return false;
        }
        // c. If v > 0 and sign < 0, return false.
        if *v > 0f64 && sign < 0 {
            return false;
        }
    }
    // 3. Return true.
    true
}

/// Utility function for determining the sign for the current set of `Duration` fields.
///
/// Equivalent: 7.5.10 `DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
#[inline]
#[must_use]
fn duration_sign(set: &Vec<f64>) -> i32 {
    // 1. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in set {
        // a. If v < 0, return -1.
        if *v < 0f64 {
            return -1;
        // b. If v > 0, return 1.
        } else if *v > 0f64 {
            return 1;
        }
    }
    // 2. Return 0.
    0
}

impl From<TimeDuration> for Duration {
    fn from(value: TimeDuration) -> Self {
        Self {
            time: value,
            date: DateDuration::default(),
        }
    }
}

// ==== FromStr trait impl ====

impl FromStr for Duration {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_duration(&mut Cursor::new(s))?;

        let minutes = if parse_record.time.fhours > 0.0 {
            parse_record.time.fhours * 60.0
        } else {
            f64::from(parse_record.time.minutes)
        };

        let seconds = if parse_record.time.fminutes > 0.0 {
            parse_record.time.fminutes * 60.0
        } else if parse_record.time.seconds > 0 {
            f64::from(parse_record.time.seconds)
        } else {
            minutes.rem_euclid(1.0) * 60.0
        };

        let milliseconds = if parse_record.time.fseconds > 0.0 {
            parse_record.time.fseconds * 1000.0
        } else {
            seconds.rem_euclid(1.0) * 1000.0
        };

        let micro = milliseconds.rem_euclid(1.0) * 1000.0;
        let nano = micro.rem_euclid(1.0) * 1000.0;

        let sign = if parse_record.sign { 1f64 } else { -1f64 };

        Ok(Self {
            date: DateDuration::new(
                f64::from(parse_record.date.years) * sign,
                f64::from(parse_record.date.months) * sign,
                f64::from(parse_record.date.weeks) * sign,
                f64::from(parse_record.date.days) * sign,
            )?,
            time: TimeDuration::new(
                f64::from(parse_record.time.hours) * sign,
                minutes.floor() * sign,
                seconds.floor() * sign,
                milliseconds.floor() * sign,
                micro.floor() * sign,
                nano.floor() * sign,
            )?,
        })
    }
}
