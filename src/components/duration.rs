//! This module implements `Duration` along with it's methods and components.

use crate::{
    components::DateTime,
    iso::{IsoDate, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, RelativeTo, RoundingIncrement, TemporalRoundingMode, TemporalUnit,
    },
    rounding::{IncrementRounder, Round},
    TemporalError, TemporalResult, TemporalUnwrap, NS_PER_DAY,
};
use ixdtf::parsers::{records::TimeDurationRecord, IsoDurationParser};
use std::{num::NonZeroU64, str::FromStr};

use self::normalized::{NormalizedDurationRecord, NormalizedTimeDuration};

use super::{tz::TimeZone, Date, Time};

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

#[cfg(test)]
impl Duration {
    pub(crate) fn hour(value: f64) -> Self {
        Self::new_unchecked(
            DateDuration::default(),
            TimeDuration::new_unchecked(value, 0.0, 0.0, 0.0, 0.0, 0.0),
        )
    }
}

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
    pub(crate) fn round_internal(
        &self,
        increment: RoundingIncrement,
        unit: TemporalUnit,
        rounding_mode: TemporalRoundingMode,
        relative_to: &RelativeTo,
        precalculated_dt: Option<DateTime>,
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

    #[inline]
    pub fn round_v2(
        &self,
        increment: Option<RoundingIncrement>,
        smallest_unit: Option<TemporalUnit>,
        largest_unit: Option<TemporalUnit>,
        rounding_mode: Option<TemporalRoundingMode>,
        relative_to: &RelativeTo,
    ) -> TemporalResult<Self> {
        // NOTE: Steps 1-14 seem to be implementation specific steps.

        // 22. If smallestUnitPresent is false and largestUnitPresent is false, then
        if largest_unit.is_none() && smallest_unit.is_none() {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range()
                .with_message("smallestUnit and largestUnit cannot both be None."));
        }

        // 14. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let increment = increment.unwrap_or_default();
        // 15. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        let rounding_mode = rounding_mode.unwrap_or_default();

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
            increment.validate(max.into(), false)?;
        }

        // 26. Let hoursToDaysConversionMayOccur be false.
        // 27. If duration.[[Days]] ≠ 0 and zonedRelativeTo is not undefined, set hoursToDaysConversionMayOccur to true.
        // 28. Else if abs(duration.[[Hours]]) ≥ 24, set hoursToDaysConversionMayOccur to true.
        let hours_to_days_may_occur =
            (self.days() != 0.0 && relative_to.zdt.is_some()) || self.hours().abs() >= 24.0;

        // 29. If smallestUnit is "nanosecond" and roundingIncrement = 1, let roundingGranularityIsNoop
        // be true; else let roundingGranularityIsNoop be false.
        let is_noop =
            smallest_unit == TemporalUnit::Nanosecond && increment == RoundingIncrement::ONE;
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
        let _precalculated: Option<DateTime> = if relative_to.zdt.is_some() && pdtr_will_be_used
        {
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

        // 36. Let norm be NormalizeTimeDuration(duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]],
        // duration.[[Microseconds]], duration.[[Nanoseconds]]).
        let norm = NormalizedTimeDuration::from_time_duration(self.time());
        // 37. Let emptyOptions be OrdinaryObjectCreate(null).

        // 38. If zonedRelativeTo is not undefined, then
        let round_result = if let Some(_zdt) = relative_to.zdt {
            // a. Let relativeEpochNs be zonedRelativeTo.[[Nanoseconds]].
            // b. Let relativeInstant be ! CreateTemporalInstant(relativeEpochNs).
            // c. Let targetEpochNs be ? AddZonedDateTime(relativeInstant, timeZoneRec, calendarRec, duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], norm, precalculatedPlainDateTime).
            // d. Let roundRecord be ? DifferenceZonedDateTimeWithRounding(relativeEpochNs, targetEpochNs, calendarRec, timeZoneRec, precalculatedPlainDateTime, emptyOptions, largestUnit, roundingIncrement, smallestUnit, roundingMode).
            // e. Let roundResult be roundRecord.[[DurationRecord]].
            return Err(TemporalError::general("Not yet implemented."));
        // 39. Else if plainRelativeTo is not undefined, then
        } else if let Some(plain_date) = relative_to.date {
            // a. Let targetTime be AddTime(0, 0, 0, 0, 0, 0, norm).
            let (balanced_days, time) = Time::default().add_norm(norm);
            // b. Let dateDuration be ? CreateTemporalDuration(duration.[[Years]], duration.[[Months]], duration.[[Weeks]],
            // duration.[[Days]] + targetTime.[[Days]], 0, 0, 0, 0, 0, 0).
            let date_duraiton = DateDuration::new(
                self.years(),
                self.months(),
                self.weeks(),
                self.days() + f64::from(balanced_days),
            )?;

            // c. Let targetDate be ? AddDate(calendarRec, plainRelativeTo, dateDuration).
            let target_date = plain_date.add_date(
                &Duration::from_date_duration(&date_duraiton),
                None,
            )?;

            let plain_dt = DateTime::new_unchecked(
                IsoDateTime::new(plain_date.iso, IsoTime::default())?,
                plain_date.calendar().clone(),
            );
            let target_dt = DateTime::new_unchecked(
                IsoDateTime::new(target_date.iso, time.iso)?,
                target_date.calendar().clone(),
            );

            // d. Let roundRecord be ? DifferencePlainDateTimeWithRounding(plainRelativeTo.[[ISOYear]], plainRelativeTo.[[ISOMonth]],
            // plainRelativeTo.[[ISODay]], 0, 0, 0, 0, 0, 0, targetDate.[[ISOYear]], targetDate.[[ISOMonth]], targetDate.[[ISODay]],
            // targetTime.[[Hours]], targetTime.[[Minutes]], targetTime.[[Seconds]], targetTime.[[Milliseconds]],
            // targetTime.[[Microseconds]], targetTime.[[Nanoseconds]], calendarRec, largestUnit, roundingIncrement,
            // smallestUnit, roundingMode, emptyOptions).
            let round_record = plain_dt.diff_dt_with_rounding(
                &target_dt,
                largest_unit,
                increment,
                smallest_unit,
                rounding_mode,
            )?;
            // e. Let roundResult be roundRecord.[[DurationRecord]].
            round_record.0
        // 40. Else,
        } else {
            // a. If calendarUnitsPresent is true, or IsCalendarUnit(largestUnit) is true, throw a RangeError exception.
            if calendar_units_present || largest_unit.is_calendar_unit() {
                return Err(TemporalError::range()
                    .with_message("Calendar units cannot be present without a relative point."));
            }
            // b. Assert: IsCalendarUnit(smallestUnit) is false.
            debug_assert!(!smallest_unit.is_calendar_unit());

            // c. Let roundRecord be ? RoundTimeDuration(duration.[[Days]], norm, roundingIncrement, smallestUnit, roundingMode).
            let (round_record, _) = TimeDuration::round_v2(
                self.days(),
                &norm,
                increment,
                smallest_unit,
                rounding_mode,
            )?;
            // d. Let normWithDays be ? Add24HourDaysToNormalizedTimeDuration(roundRecord.[[NormalizedDuration]].[[NormalizedTime]],
            // roundRecord.[[NormalizedDuration]].[[Days]]).
            let norm_with_days = round_record.0 .1.add_days(round_record.0 .0.days as i64)?;
            // e. Let balanceResult be ? BalanceTimeDuration(normWithDays, largestUnit).
            let (balanced_days, balanced_time) =
                TimeDuration::from_normalized(norm_with_days, largest_unit)?;
            // f. Let roundResult be CreateDurationRecord(0, 0, 0, balanceResult.[[Days]], balanceResult.[[Hours]],
            // balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]],
            // balanceResult.[[Microseconds]], balanceResult.[[Nanoseconds]]).
            Duration::from_day_and_time(balanced_days, &balanced_time)
        };

        // 41. Return ? CreateTemporalDuration(roundResult.[[Years]], roundResult.[[Months]], roundResult.[[Weeks]], roundResult.[[Days]], roundResult.[[Hours]], roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]], roundResult.[[Microseconds]], roundResult.[[Nanoseconds]]).
        Ok(round_result)
    }

    /// Rounds the current `Duration`.
    #[inline]
    pub fn round(
        &self,
        increment: Option<RoundingIncrement>,
        smallest_unit: Option<TemporalUnit>,
        largest_unit: Option<TemporalUnit>,
        rounding_mode: Option<TemporalRoundingMode>,
        relative_to: &RelativeTo,
    ) -> TemporalResult<Self> {
        // NOTE: Steps 1-14 seem to be implementation specific steps.

        // 22. If smallestUnitPresent is false and largestUnitPresent is false, then
        if largest_unit.is_none() && smallest_unit.is_none() {
            // a. Throw a RangeError exception.
            return Err(TemporalError::range()
                .with_message("smallestUnit and largestUnit cannot both be None."));
        }

        // 14. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        let increment = increment.unwrap_or_default();
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
            increment.validate(max.into(), false)?;
        }

        // 26. Let hoursToDaysConversionMayOccur be false.
        // 27. If duration.[[Days]] ≠ 0 and zonedRelativeTo is not undefined, set hoursToDaysConversionMayOccur to true.
        // 28. Else if abs(duration.[[Hours]]) ≥ 24, set hoursToDaysConversionMayOccur to true.
        let hours_to_days_may_occur =
            (self.days() != 0.0 && relative_to.zdt.is_some()) || self.hours().abs() >= 24.0;

        // 29. If smallestUnit is "nanosecond" and roundingIncrement = 1, let roundingGranularityIsNoop
        // be true; else let roundingGranularityIsNoop be false.
        let is_noop =
            smallest_unit == TemporalUnit::Nanosecond && increment == RoundingIncrement::ONE;
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

        // TODO: relativeTo will need to be removed soon.
        let relative_to_date = relative_to.date;

        // 36. Let unbalanceResult be ? UnbalanceDateDurationRelative(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], largestUnit, plainRelativeTo, calendarRec).
        let unbalanced = self
            .date()
            .unbalance_relative(largest_unit, relative_to_date)?;

        // NOTE: Step 37 handled in round duration
        // 37. Let norm be NormalizeTimeDuration(duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]],
        // duration.[[Milliseconds]], duration.[[Microseconds]], duration.[[Nanoseconds]]).
        // 38. Let roundRecord be ? RoundDuration(unbalanceResult.[[Years]], unbalanceResult.[[Months]],
        // unbalanceResult.[[Weeks]], unbalanceResult.[[Days]], norm, roundingIncrement, smallestUnit,
        // roundingMode, plainRelativeTo, calendarRec, zonedRelativeTo, timeZoneRec, precalculatedPlainDateTime).
        let (round_result, _) = Self::new_unchecked(unbalanced, *self.time()).round_internal(
            increment,
            smallest_unit,
            mode,
            relative_to,
            precalculated,
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
            let norm_with_days = round_result.0 .1.add_days(round_result.0 .0.days as i64)?;
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
        let result =
            intermediate.balance_relative(largest_unit, smallest_unit, relative_to_date)?;

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

// TODO: Update, optimize, and fix the below. is_valid_duration should probably be generic over a T.

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
        let parse_record = IsoDurationParser::new(s)
            .parse()
            .map_err(|e| TemporalError::general(format!("{e}")))?;

        let (hours, minutes, seconds, millis, micros, nanos) = match parse_record.time {
            Some(TimeDurationRecord::Hours { hours, fraction }) => {
                let minutes = fraction.div_euclid(60 * 1_000_000_000);
                let rem = fraction.rem_euclid(60 * 1_000_000_000);

                let seconds = rem.div_euclid(1_000_000_000);
                let rem = rem.rem_euclid(1_000_000_000);

                let milliseconds = rem.div_euclid(1_000_000);
                let rem = rem.rem_euclid(1_000_000);

                let microseconds = rem.div_euclid(1_000);
                let nanoseconds = rem.rem_euclid(1_000);

                (
                    f64::from(hours),
                    minutes as f64,
                    seconds as f64,
                    milliseconds as f64,
                    microseconds as f64,
                    nanoseconds as f64,
                )
            }
            // Minutes variant is defined as { hours: u32, minutes: u32, fraction: u64 }
            Some(TimeDurationRecord::Minutes {
                hours,
                minutes,
                fraction,
            }) => {
                let seconds = fraction.div_euclid(1_000_000_000);
                let rem = fraction.rem_euclid(1_000_000_000);

                let milliseconds = rem.div_euclid(1_000_000);
                let rem = rem.rem_euclid(1_000_000);

                let microseconds = rem.div_euclid(1_000);
                let nanoseconds = rem.rem_euclid(1_000);

                (
                    f64::from(hours),
                    f64::from(minutes),
                    seconds as f64,
                    milliseconds as f64,
                    microseconds as f64,
                    nanoseconds as f64,
                )
            }
            // Seconds variant is defined as { hours: u32, minutes: u32, seconds: u32, fraction: u32 }
            Some(TimeDurationRecord::Seconds {
                hours,
                minutes,
                seconds,
                fraction,
            }) => {
                let milliseconds = fraction.div_euclid(1_000_000);
                let rem = fraction.rem_euclid(1_000_000);

                let microseconds = rem.div_euclid(1_000);
                let nanoseconds = rem.rem_euclid(1_000);

                (
                    f64::from(hours),
                    f64::from(minutes),
                    f64::from(seconds),
                    milliseconds as f64,
                    microseconds as f64,
                    nanoseconds as f64,
                )
            }
            None => (0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
        };

        let (years, months, weeks, days) = if let Some(date) = parse_record.date {
            (date.years, date.months, date.weeks, date.days)
        } else {
            (0, 0, 0, 0)
        };

        let sign = f64::from(parse_record.sign as i8);

        Ok(Self {
            date: DateDuration::new(
                f64::from(years) * sign,
                f64::from(months) * sign,
                f64::from(weeks) * sign,
                f64::from(days) * sign,
            )?,
            time: TimeDuration::new(
                hours * sign,
                minutes * sign,
                seconds * sign,
                millis * sign,
                micros * sign,
                nanos * sign,
            )?,
        })
    }
}

// ==== New Nudge Algo Functions ====

// TODO: Reorganize as needed.

pub(crate) struct NudgeRecord {
    normalized: NormalizedDurationRecord,
    total: Option<i128>, // TODO: adjust
    nudge_epoch_ns: i128,
    expanded: bool,
}

// TODO: Add assertion into impl.
// TODO: Add unit tests specifically for nudge_calendar_unit if possible.
#[allow(clippy::too_many_arguments)]
pub(crate) fn nudge_calendar_unit(
    sign: i32,
    duration: &NormalizedDurationRecord,
    dest_epoch_ns: i128,
    dt: &DateTime,
    tz: Option<TimeZone>, // ???
    increment: RoundingIncrement,
    unit: TemporalUnit,
    rounding_mode: TemporalRoundingMode,
) -> TemporalResult<NudgeRecord> {
    // NOTE: r2 may never be used...need to test.
    let (r1, r2, start_duration, end_duration) = match unit {
        // 1. If unit is "year", then
        TemporalUnit::Year => {
            // a. Let years be RoundNumberToIncrement(duration.[[Years]], increment, "trunc").
            let years = IncrementRounder::from_potentially_negative_parts(
                duration.date().years,
                increment.as_extended_increment(),
            )?
            .round(TemporalRoundingMode::Trunc);
            // b. Let r1 be years.
            let r1 = years;
            // c. Let r2 be years + increment × sign.
            let r2 = years + i128::from(increment.get()) * i128::from(sign);
            // d. Let startDuration be ? CreateNormalizedDurationRecord(r1, 0, 0, 0, ZeroTimeDuration()).
            // e. Let endDuration be ? CreateNormalizedDurationRecord(r2, 0, 0, 0, ZeroTimeDuration()).
            (
                r1,
                r2,
                DateDuration::new(r1 as f64, 0.0, 0.0, 0.0)?,
                DateDuration::new(r2 as f64, 0.0, 0.0, 0.0)?,
            )
        }
        // 2. Else if unit is "month", then
        TemporalUnit::Month => {
            // a. Let months be RoundNumberToIncrement(duration.[[Months]], increment, "trunc").
            let months = IncrementRounder::from_potentially_negative_parts(
                duration.date().months,
                increment.as_extended_increment(),
            )?
            .round(TemporalRoundingMode::Trunc);
            // b. Let r1 be months.
            let r1 = months;
            // c. Let r2 be months + increment × sign.
            let r2 = months + i128::from(increment.get()) * i128::from(sign);
            // d. Let startDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], r1, 0, 0, ZeroTimeDuration()).
            // e. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], r2, 0, 0, ZeroTimeDuration()).
            (
                r1,
                r2,
                DateDuration::new(duration.date().years, r1 as f64, 0.0, 0.0)?,
                DateDuration::new(duration.date().years, r2 as f64, 0.0, 0.0)?,
            )
        }
        // 3. Else if unit is "week", then
        TemporalUnit::Week => {
            // TODO: Reconcile potential overflow on years as i32. `ValidateDuration` requires years, months, weeks to be abs(x) <= 2^32

            // a. Let isoResult1 be BalanceISODate(dateTime.[[Year]] + duration.[[Years]], dateTime.[[Month]] + duration.[[Months]], dateTime.[[Day]]).
            let iso_one = IsoDate::balance(
                dt.iso_year() + duration.date().years as i32,
                i32::from(dt.iso_month()) + duration.date().months as i32,
                i32::from(dt.iso_day()),
            );

            // b. Let isoResult2 be BalanceISODate(dateTime.[[Year]] + duration.[[Years]], dateTime.[[Month]] + duration.[[Months]], dateTime.[[Day]] + duration.[[Days]]).
            let iso_two = IsoDate::balance(
                dt.iso_year() + duration.date().years as i32,
                i32::from(dt.iso_month()) + duration.date().months as i32,
                i32::from(dt.iso_day()) + duration.date().days as i32,
            );

            // c. Let weeksStart be ! CreateTemporalDate(isoResult1.[[Year]], isoResult1.[[Month]], isoResult1.[[Day]], calendarRec.[[Receiver]]).
            let weeks_start = Date::new(
                iso_one.year,
                iso_one.month.into(),
                iso_one.day.into(),
                dt.calendar().clone(),
                ArithmeticOverflow::Constrain,
            )?;

            // d. Let weeksEnd be ! CreateTemporalDate(isoResult2.[[Year]], isoResult2.[[Month]], isoResult2.[[Day]], calendarRec.[[Receiver]]).
            let weeks_end = Date::new(
                iso_two.year,
                iso_two.month.into(),
                iso_two.day.into(),
                dt.calendar().clone(),
                ArithmeticOverflow::Constrain,
            )?;

            // e. Let untilOptions be OrdinaryObjectCreate(null).
            // f. Perform ! CreateDataPropertyOrThrow(untilOptions, "largestUnit", "week").
            // g. Let untilResult be ? DifferenceDate(calendarRec, weeksStart, weeksEnd, untilOptions).
            let until_result =
                weeks_start.internal_diff_date(&weeks_end, TemporalUnit::Week)?;

            // h. Let weeks be RoundNumberToIncrement(duration.[[Weeks]] + untilResult.[[Weeks]], increment, "trunc").
            let weeks = IncrementRounder::from_potentially_negative_parts(
                duration.date().weeks + until_result.weeks(),
                increment.as_extended_increment(),
            )?
            .round(TemporalRoundingMode::Trunc);

            // i. Let r1 be weeks.
            let r1 = weeks;
            // j. Let r2 be weeks + increment × sign.
            let r2 = weeks + i128::from(increment.get()) * i128::from(sign);
            // k. Let startDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], r1, 0, ZeroTimeDuration()).
            // l. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], r2, 0, ZeroTimeDuration()).
            (
                r1,
                r2,
                DateDuration::new(
                    duration.date().years,
                    duration.date().months,
                    r1 as f64,
                    0.0,
                )?,
                DateDuration::new(
                    duration.date().years,
                    duration.date().months,
                    r2 as f64,
                    0.0,
                )?,
            )
        }
        TemporalUnit::Day => {
            // 4. Else,
            // a. Assert: unit is "day".
            // b. Let days be RoundNumberToIncrement(duration.[[Days]], increment, "trunc").
            let days = IncrementRounder::from_potentially_negative_parts(
                duration.date().days,
                increment.as_extended_increment(),
            )?
            .round(TemporalRoundingMode::Trunc);
            // c. Let r1 be days.
            let r1 = days;
            // d. Let r2 be days + increment × sign.
            let r2 = days + i128::from(increment.get()) * i128::from(sign);
            // e. Let startDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], r1, ZeroTimeDuration()).
            // f. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], r2, ZeroTimeDuration()).
            (
                r1,
                r2,
                DateDuration::new(
                    duration.date().years,
                    duration.date().months,
                    duration.date().weeks,
                    r1 as f64,
                )?,
                DateDuration::new(
                    duration.date().years,
                    duration.date().months,
                    duration.date().weeks,
                    r2 as f64,
                )?,
            )
        }
        _ => unreachable!(), // TODO: potentially reject with range error?
    };

    // 5. Let start be ? AddDateTime(dateTime.[[Year]], dateTime.[[Month]], dateTime.[[Day]], dateTime.[[Hour]], dateTime.[[Minute]],
    // dateTime.[[Second]], dateTime.[[Millisecond]], dateTime.[[Microsecond]], dateTime.[[Nanosecond]], calendarRec,
    // startDuration.[[Years]], startDuration.[[Months]], startDuration.[[Weeks]], startDuration.[[Days]], startDuration.[[NormalizedTime]], undefined).
    let start = dt.iso.add_date_duration(
        dt.calendar().clone(),
        &start_duration,
        NormalizedTimeDuration::default(),
        None,
    )?;

    // 6. Let end be ? AddDateTime(dateTime.[[Year]], dateTime.[[Month]], dateTime.[[Day]], dateTime.[[Hour]],
    // dateTime.[[Minute]], dateTime.[[Second]], dateTime.[[Millisecond]], dateTime.[[Microsecond]],
    // dateTime.[[Nanosecond]], calendarRec, endDuration.[[Years]], endDuration.[[Months]], endDuration.[[Weeks]],
    // endDuration.[[Days]], endDuration.[[NormalizedTime]], undefined).
    let end = dt.iso.add_date_duration(
        dt.calendar().clone(),
        &end_duration,
        NormalizedTimeDuration::default(),
        None,
    )?;

    // 7. If timeZoneRec is unset, then
    let (start_epoch_ns, end_epoch_ns) = if tz.is_none() {
        // TODO: Test valid range of EpochNanoseconds in order to add `expect` over `unwrap_or`
        // a. Let startEpochNs be GetUTCEpochNanoseconds(start.[[Year]], start.[[Month]], start.[[Day]], start.[[Hour]], start.[[Minute]], start.[[Second]], start.[[Millisecond]], start.[[Microsecond]], start.[[Nanosecond]]).
        // b. Let endEpochNs be GetUTCEpochNanoseconds(end.[[Year]], end.[[Month]], end.[[Day]], end.[[Hour]], end.[[Minute]], end.[[Second]], end.[[Millisecond]], end.[[Microsecond]], end.[[Nanosecond]]).
        (
            start.as_nanoseconds(0.0).unwrap_or(0),
            end.as_nanoseconds(0.0).unwrap_or(0),
        )
    // 8. Else,
    } else {
        // a. Let startDateTime be ! CreateTemporalDateTime(start.[[Year]], start.[[Month]], start.[[Day]],
        // start.[[Hour]], start.[[Minute]], start.[[Second]], start.[[Millisecond]], start.[[Microsecond]],
        // start.[[Nanosecond]], calendarRec.[[Receiver]]).
        // b. Let startInstant be ? GetInstantFor(timeZoneRec, startDateTime, "compatible").
        // c. Let startEpochNs be startInstant.[[Nanoseconds]].
        // d. Let endDateTime be ! CreateTemporalDateTime(end.[[Year]], end.[[Month]], end.[[Day]], end.[[Hour]], end.[[Minute]], end.[[Second]], end.[[Millisecond]], end.[[Microsecond]], end.[[Nanosecond]], calendarRec.[[Receiver]]).
        // e. Let endInstant be ? GetInstantFor(timeZoneRec, endDateTime, "compatible").
        // f. Let endEpochNs be endInstant.[[Nanoseconds]].
        return Err(TemporalError::general(
            "TimeZone handling not yet implemented.",
        ));
    };

    // 9. If endEpochNs = startEpochNs, throw a RangeError exception.
    if end_epoch_ns == start_epoch_ns {
        return Err(
            TemporalError::range().with_message("endEpochNs cannot be equal to startEpochNs")
        );
    }

    // TODO: Add early RangeError steps

    // NOTE: Below is removed in place of using `IncrementRounder`
    // 10. If sign < 0, let isNegative be negative; else let isNegative be positive.
    // 11. Let unsignedRoundingMode be GetUnsignedRoundingMode(roundingMode, isNegative).

    // TODO: Step 12..13 could be problematic...need tests
    // and verify, or completely change the approach involved.
    // 12. Let progress be (destEpochNs - startEpochNs) / (endEpochNs - startEpochNs).
    // 13. Let total be r1 + progress × increment × sign.

    // TODO: Remove if invalid
    // NOTE: Changes to 12 -> 12. Let progress be ((destEpochNs - startEpochNs) * increment) / (endEpochNs - startEpochNs).
    // let progress = ((dest_epoch_ns - start_epoch_ns) * i128::from(increment.get()))
    //     .div_euclid(end_epoch_ns - start_epoch_ns);
    // NOTE: Changes to 13 -> 13. let total be r1 + progress * sign.
    // let total = r1 + progress * i128::from(sign);

    let progress = (dest_epoch_ns - start_epoch_ns) as f64 / (end_epoch_ns - start_epoch_ns) as f64;
    let total = r1 as f64 + progress * increment.get() as f64 + f64::from(sign);

    // TODO: Test and verify that `IncrementRounder` handles the below case.
    // NOTE(nekevss): Below will not return the calculated r1 or r2, so it is imporant to not use
    // the result beyond determining rounding direction.
    // 14. NOTE: The above two steps cannot be implemented directly using floating-point arithmetic.
    // This division can be implemented as if constructing Normalized Time Duration Records for the denominator
    // and numerator of total and performing one division operation with a floating-point result.
    // 15. Let roundedUnit be ApplyUnsignedRoundingMode(total, r1, r2, unsignedRoundingMode).
    let rounded_unit = IncrementRounder::from_potentially_negative_parts(
        total,
        increment.as_extended_increment(),
    )?
    .round(rounding_mode);

    // 16. If roundedUnit - total < 0, let roundedSign be -1; else let roundedSign be 1.
    // 19. Return Duration Nudge Result Record { [[Duration]]: resultDuration, [[Total]]: total, [[NudgedEpochNs]]: nudgedEpochNs, [[DidExpandCalendarUnit]]: didExpandCalendarUnit }.
    // 17. If roundedSign = sign, then
    if rounded_unit == r2.abs() {
        // a. Let didExpandCalendarUnit be true.
        // b. Let resultDuration be endDuration.
        // c. Let nudgedEpochNs be endEpochNs.
        Ok(NudgeRecord {
            normalized: NormalizedDurationRecord::new(
                end_duration,
                NormalizedTimeDuration::default(),
            )?,
            total: Some(total as i128),
            nudge_epoch_ns: end_epoch_ns,
            expanded: true,
        })
    // 18. Else,
    } else {
        // a. Let didExpandCalendarUnit be false.
        // b. Let resultDuration be startDuration.
        // c. Let nudgedEpochNs be startEpochNs.
        Ok(NudgeRecord {
            normalized: NormalizedDurationRecord::new(
                start_duration,
                NormalizedTimeDuration::default(),
            )?,
            total: Some(total as i128),
            nudge_epoch_ns: start_epoch_ns,
            expanded: false,
        })
    }
}

#[inline]
fn nudge_to_zoned_time() -> TemporalResult<NudgeRecord> {
    // TODO: Implement
    Err(TemporalError::general("Not yet implemented."))
}

#[inline]
fn nudge_to_day_or_time(
    duration: &NormalizedDurationRecord,
    dest_epoch_ns: i128,
    largest_unit: TemporalUnit,
    increment: RoundingIncrement,
    smallest_unit: TemporalUnit,
    rounding_mode: TemporalRoundingMode,
) -> TemporalResult<NudgeRecord> {
    // 1. Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains smallestUnit, is time.
    // 2. Let norm be ! Add24HourDaysToNormalizedTimeDuration(duration.[[NormalizedTime]], duration.[[Days]]).
    let norm = duration.norm().add_days(duration.date().days as i64)?;

    // 3. Let unitLength be the value in the "Length in Nanoseconds" column of the row of Table 22 whose "Singular" column contains smallestUnit.
    let unit_length = smallest_unit.as_nanoseconds().temporal_unwrap()?;
    // 4. Let total be DivideNormalizedTimeDuration(norm, unitLength).
    let total = norm.divide(unit_length as i64);

    // TODO: Adjust TemporalUnit::as_nanoseconds
    // 5. Let roundedNorm be ? RoundNormalizedTimeDurationToIncrement(norm, unitLength × increment, roundingMode).
    let rounded_norm = norm.round(
        unsafe {
            NonZeroU64::new_unchecked(unit_length)
                .checked_mul(increment.as_extended_increment())
                .temporal_unwrap()?
        },
        rounding_mode,
    )?;

    // 6. Let diffNorm be ! SubtractNormalizedTimeDuration(roundedNorm, norm).
    let diff_norm = rounded_norm.checked_sub(&norm)?;

    // 7. Let wholeDays be truncate(DivideNormalizedTimeDuration(norm, nsPerDay)).
    let whole_days = norm.divide(NS_PER_DAY as i64);

    // 8. Let roundedFractionalDays be DivideNormalizedTimeDuration(roundedNorm, nsPerDay).
    let (rounded_whole_days, rounded_remainder) = rounded_norm.div_rem(NS_PER_DAY);

    // 9. Let roundedWholeDays be truncate(roundedFractionalDays).
    // 10. Let dayDelta be roundedWholeDays - wholeDays.
    let delta = rounded_whole_days - whole_days;
    // 11. If dayDelta < 0, let dayDeltaSign be -1; else if dayDelta > 0, let dayDeltaSign be 1; else let dayDeltaSign be 0.
    // 12. If dayDeltaSign = NormalizedTimeDurationSign(norm), let didExpandDays be true; else let didExpandDays be false.
    let did_expand_days = delta.signum() == norm.sign().into();

    // 13. Let nudgedEpochNs be AddNormalizedTimeDurationToEpochNanoseconds(diffNorm, destEpochNs).
    let nudged_ns = diff_norm.0 + dest_epoch_ns;

    // 14. Let days be 0.
    let mut days = 0;
    // 15. Let remainder be roundedNorm.
    let mut remainder = rounded_norm;
    // 16. If LargerOfTwoTemporalUnits(largestUnit, "day") is largestUnit, then
    if largest_unit.max(TemporalUnit::Day) == largest_unit {
        // a. Set days to roundedWholeDays.
        days = rounded_whole_days;
        // b. Set remainder to remainder(roundedFractionalDays, 1) × nsPerDay.
        remainder = NormalizedTimeDuration(rounded_remainder);
    }
    // 17. Let resultDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], days, remainder).
    let result_duration = NormalizedDurationRecord::new(
        DateDuration::new(
            duration.date().years,
            duration.date().months,
            duration.date().weeks,
            days as f64,
        )?,
        remainder,
    )?;
    // 18. Return Duration Nudge Result Record { [[Duration]]: resultDuration, [[Total]]: total,
    // [[NudgedEpochNs]]: nudgedEpochNs, [[DidExpandCalendarUnit]]: didExpandDays }.
    Ok(NudgeRecord {
        normalized: result_duration,
        total: Some(total),
        nudge_epoch_ns: nudged_ns,
        expanded: did_expand_days,
    })
}

// 7.5.43 BubbleRelativeDuration ( sign, duration, nudgedEpochNs, dateTime, calendarRec, timeZoneRec, largestUnit, smallestUnit )
#[inline]
#[allow(clippy::too_many_arguments)]
fn bubble_relative_duration(
    sign: i32,
    duration: &NormalizedDurationRecord,
    nudge_epoch_ns: i128,
    date_time: &DateTime,
    tz: Option<TimeZone>,
    largest_unit: TemporalUnit,
    smallest_unit: TemporalUnit,
) -> TemporalResult<NormalizedDurationRecord> {
    // Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains largestUnit, is date.
    // 2. Assert: The value in the "Category" column of the row of Table 22 whose "Singular" column contains smallestUnit, is date.
    let mut duration = *duration;
    // 3. If smallestUnit is "year", return duration.
    if smallest_unit == TemporalUnit::Year {
        return Ok(duration);
    }
    // 4. Let largestUnitIndex be the ordinal index of the row of Table 22 whose "Singular" column contains largestUnit.
    // 5. Let smallestUnitIndex be the ordinal index of the row of Table 22 whose "Singular" column contains smallestUnit.
    // 6. Let unitIndex be smallestUnitIndex - 1.
    let mut unit = smallest_unit - 1;
    // 7. Let done be false.
    let mut done = false;
    // 8. Repeat, while unitIndex ≤ largestUnitIndex and done is false,
    while unit <= largest_unit && !done {
        // a. Let unit be the value in the "Singular" column of Table 22 in the row whose ordinal index is unitIndex.
        // b. If unit is not "week", or largestUnit is "week", then
        if unit != TemporalUnit::Week || largest_unit == TemporalUnit::Week {
            let end_duration = match unit {
                // i. If unit is "year", then
                TemporalUnit::Year => {
                    // 1. Let years be duration.[[Years]] + sign.
                    // 2. Let endDuration be ? CreateNormalizedDurationRecord(years, 0, 0, 0, ZeroTimeDuration()).
                    DateDuration::new(duration.date().years + f64::from(sign), 0.0, 0.0, 0.0)?
                }
                // ii. Else if unit is "month", then
                TemporalUnit::Month => {
                    // 1. Let months be duration.[[Months]] + sign.
                    // 2. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], months, 0, 0, ZeroTimeDuration()).
                    DateDuration::new(
                        duration.date().years,
                        duration.date().months + f64::from(sign),
                        0.0,
                        0.0,
                    )?
                }
                // iii. Else if unit is "week", then
                TemporalUnit::Week => {
                    // 1. Let weeks be duration.[[Weeks]] + sign.
                    // 2. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], weeks, 0, ZeroTimeDuration()).
                    DateDuration::new(
                        duration.date().years,
                        duration.date().months,
                        duration.date().weeks + f64::from(sign),
                        0.0,
                    )?
                }
                // iv. Else,
                TemporalUnit::Day => {
                    // 1. Assert: unit is "day".
                    // 2. Let days be duration.[[Days]] + sign.
                    // 3. Let endDuration be ? CreateNormalizedDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], days, ZeroTimeDuration()).
                    DateDuration::new(
                        duration.date().years,
                        duration.date().months,
                        duration.date().weeks,
                        duration.date().days + f64::from(sign),
                    )?
                }
                _ => unreachable!(),
            };

            // v. Let end be ? AddDateTime(dateTime.[[Year]], dateTime.[[Month]], dateTime.[[Day]], dateTime.[[Hour]], dateTime.[[Minute]],
            // dateTime.[[Second]], dateTime.[[Millisecond]], dateTime.[[Microsecond]], dateTime.[[Nanosecond]], calendarRec,
            // endDuration.[[Years]], endDuration.[[Months]], endDuration.[[Weeks]], endDuration.[[Days]], endDuration.[[NormalizedTime]], undefined).
            let end = date_time.iso.add_date_duration(
                date_time.calendar().clone(),
                &end_duration,
                NormalizedTimeDuration::default(),
                None,
            )?;

            // vi. If timeZoneRec is unset, then
            let end_epoch_ns = if let Some(ref _tz) = tz {
                // 1. Let endDateTime be ! CreateTemporalDateTime(end.[[Year]], end.[[Month]], end.[[Day]],
                // end.[[Hour]], end.[[Minute]], end.[[Second]], end.[[Millisecond]], end.[[Microsecond]],
                // end.[[Nanosecond]], calendarRec.[[Receiver]]).
                // 2. Let endInstant be ? GetInstantFor(timeZoneRec, endDateTime, "compatible").
                // 3. Let endEpochNs be endInstant.[[Nanoseconds]].
                todo!()
            // vii. Else,
            } else {
                // 1. Let endEpochNs be GetUTCEpochNanoseconds(end.[[Year]], end.[[Month]], end.[[Day]], end.[[Hour]],
                // end.[[Minute]], end.[[Second]], end.[[Millisecond]], end.[[Microsecond]], end.[[Nanosecond]]).
                end.as_nanoseconds(0.0).temporal_unwrap()?
            };
            // viii. Let beyondEnd be nudgedEpochNs - endEpochNs.
            let beyond_end = nudge_epoch_ns - end_epoch_ns;
            // ix. If beyondEnd < 0, let beyondEndSign be -1; else if beyondEnd > 0, let beyondEndSign be 1; else let beyondEndSign be 0.
            // x. If beyondEndSign ≠ -sign, then
            if beyond_end.signum() != -i128::from(sign) {
                // 1. Set duration to endDuration.
                duration = NormalizedDurationRecord::from_date_duration(end_duration)?;
            // xi. Else,
            } else {
                // 1. Set done to true.
                done = true
            }
        }
        // c. Set unitIndex to unitIndex - 1.
        unit = unit - 1;
    }

    Ok(duration)
}

pub(crate) type RelativeRoundResult = (Duration, Option<i128>);

// 7.5.44 RoundRelativeDuration ( duration, destEpochNs, dateTime, calendarRec, timeZoneRec, largestUnit, increment, smallestUnit, roundingMode )
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) fn round_relative_duration(
    duration: &NormalizedDurationRecord,
    dest_epoch_ns: i128,
    dt: &DateTime,
    tz: Option<TimeZone>,
    largest_unit: TemporalUnit,
    increment: RoundingIncrement,
    smallest_unit: TemporalUnit,
    rounding_mode: TemporalRoundingMode,
) -> TemporalResult<RelativeRoundResult> {
    // 1. Let irregularLengthUnit be false.
    // 2. If IsCalendarUnit(smallestUnit) is true, set irregularLengthUnit to true.
    // 3. If timeZoneRec is not unset and smallestUnit is "day", set irregularLengthUnit to true.
    let irregular_unit =
        smallest_unit.is_calendar_unit() || (tz.is_some() && smallest_unit == TemporalUnit::Day);
    
    // 4. If DurationSign(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], NormalizedTimeDurationSign(duration.[[NormalizedTime]]), 0, 0, 0, 0, 0) < 0, let sign be -1; else let sign be 1.
    let sign = if duration.sign()? < 0 { -1 } else { 1 };

    // 5. If irregularLengthUnit is true, then
    let nudge_result = if irregular_unit {
        // a. Let nudgeResult be ? NudgeToCalendarUnit(sign, duration, destEpochNs, dateTime, calendarRec, timeZoneRec, increment, smallestUnit, roundingMode).
        nudge_calendar_unit(
            sign,
            duration,
            dest_epoch_ns,
            dt,
            tz.clone(),
            increment,
            smallest_unit,
            rounding_mode,
        )?
    // 6. Else if timeZoneRec is not unset, then
    } else if let Some(ref _tz) = tz {
        // a. Let nudgeResult be ? NudgeToZonedTime(sign, duration, dateTime, calendarRec, timeZoneRec, increment, smallestUnit, roundingMode).
        nudge_to_zoned_time()?
    // 7. Else,
    } else {
        // a. Let nudgeResult be ? NudgeToDayOrTime(duration, destEpochNs, largestUnit, increment, smallestUnit, roundingMode).
        nudge_to_day_or_time(
            duration,
            dest_epoch_ns,
            largest_unit,
            increment,
            smallest_unit,
            rounding_mode,
        )?
    };

    // 8. Set duration to nudgeResult.[[Duration]].
    let mut duration = nudge_result.normalized;

    // 9. If nudgeResult.[[DidExpandCalendarUnit]] is true and smallestUnit is not "week", then
    if nudge_result.expanded && smallest_unit != TemporalUnit::Week {
        // a. Let startUnit be LargerOfTwoTemporalUnits(smallestUnit, "day").
        let start_unit = smallest_unit.max(TemporalUnit::Day);
        // b. Set duration to ? BubbleRelativeDuration(sign, duration, nudgeResult.[[NudgedEpochNs]], dateTime, calendarRec, timeZoneRec, largestUnit, startUnit).
        duration = bubble_relative_duration(
            sign,
            &duration,
            nudge_result.nudge_epoch_ns,
            dt,
            tz,
            largest_unit,
            start_unit,
        )?
    };

    // 10. If IsCalendarUnit(largestUnit) is true or largestUnit is "day", then
    let largest_unit = if largest_unit.is_calendar_unit() || largest_unit == TemporalUnit::Day {
        // a. Set largestUnit to "hour".
        TemporalUnit::Hour
    } else {
        largest_unit
    };

    // 11. Let balanceResult be ? BalanceTimeDuration(duration.[[NormalizedTime]], largestUnit).
    let balance_result = TimeDuration::from_normalized(duration.0 .1, largest_unit)?;

    // TODO: Need to validate the below.
    // 12. Return the Record { [[Duration]]: CreateDurationRecord(duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], balanceResult.[[Hours]], balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]], balanceResult.[[Microseconds]], balanceResult.[[Nanoseconds]]), [[Total]]: nudgeResult.[[Total]]  }.
    Ok((
        Duration::new_unchecked(duration.date(), balance_result.1),
        nudge_result.total,
    ))
}
