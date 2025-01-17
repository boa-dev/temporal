//! This module implements `Duration` along with it's methods and components.

use crate::{
    components::{timezone::TimeZoneProvider, PlainDateTime, PlainTime},
    iso::{IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, RelativeTo, ResolvedRoundingOptions, RoundingOptions, TemporalUnit,
    },
    primitive::FiniteF64,
    temporal_assert, Sign, TemporalError, TemporalResult,
};
use alloc::format;
use alloc::vec;
use alloc::vec::Vec;
use core::str::FromStr;
use ixdtf::parsers::{records::TimeDurationRecord, IsoDurationParser};
use normalized::NormalizedDurationRecord;
use num_traits::AsPrimitive;

use self::normalized::NormalizedTimeDuration;

#[cfg(feature = "experimental")]
use crate::components::timezone::TZ_PROVIDER;

mod date;
pub(crate) mod normalized;
mod time;

#[cfg(feature = "experimental")]
#[cfg(test)]
mod tests;

#[doc(inline)]
pub use date::DateDuration;
#[doc(inline)]
pub use time::TimeDuration;

use super::ZonedDateTime;

/// A `PartialDuration` is a Duration that may have fields not set.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct PartialDuration {
    /// A potentially existent `years` field.
    pub years: Option<FiniteF64>,
    /// A potentially existent `months` field.
    pub months: Option<FiniteF64>,
    /// A potentially existent `weeks` field.
    pub weeks: Option<FiniteF64>,
    /// A potentially existent `days` field.
    pub days: Option<FiniteF64>,
    /// A potentially existent `hours` field.
    pub hours: Option<FiniteF64>,
    /// A potentially existent `minutes` field.
    pub minutes: Option<FiniteF64>,
    /// A potentially existent `seconds` field.
    pub seconds: Option<FiniteF64>,
    /// A potentially existent `milliseconds` field.
    pub milliseconds: Option<FiniteF64>,
    /// A potentially existent `microseconds` field.
    pub microseconds: Option<FiniteF64>,
    /// A potentially existent `nanoseconds` field.
    pub nanoseconds: Option<FiniteF64>,
}

impl PartialDuration {
    /// Returns whether the `PartialDuration` is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

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
    pub(crate) fn hour(value: FiniteF64) -> Self {
        Self::new_unchecked(
            DateDuration::default(),
            TimeDuration::new_unchecked(
                value,
                FiniteF64::default(),
                FiniteF64::default(),
                FiniteF64::default(),
                FiniteF64::default(),
                FiniteF64::default(),
            ),
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

    #[inline]
    pub(crate) fn from_normalized(
        duration_record: NormalizedDurationRecord,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<Self> {
        let (overflow_day, time) = TimeDuration::from_normalized(
            duration_record.normalized_time_duration(),
            largest_unit,
        )?;
        let date = DateDuration::new(
            duration_record.date().years,
            duration_record.date().months,
            duration_record.date().weeks,
            duration_record.date().days.checked_add(&overflow_day)?,
        )?;
        Ok(Self::new_unchecked(date, time))
    }

    /// Returns the a `Vec` of the fields values.
    #[inline]
    #[must_use]
    pub(crate) fn fields(&self) -> Vec<FiniteF64> {
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
        years: FiniteF64,
        months: FiniteF64,
        weeks: FiniteF64,
        days: FiniteF64,
        hours: FiniteF64,
        minutes: FiniteF64,
        seconds: FiniteF64,
        milliseconds: FiniteF64,
        microseconds: FiniteF64,
        nanoseconds: FiniteF64,
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
        if !is_valid_duration(
            years,
            months,
            weeks,
            days,
            hours,
            minutes,
            seconds,
            milliseconds,
            microseconds,
            nanoseconds,
        ) {
            return Err(TemporalError::range().with_message("Duration was not valid."));
        }
        Ok(duration)
    }

    /// Creates a `Duration` from a provided a day and a `TimeDuration`.
    ///
    /// Note: `TimeDuration` records can store a day value to deal with overflow.
    #[must_use]
    pub fn from_day_and_time(day: FiniteF64, time: &TimeDuration) -> Self {
        Self {
            date: DateDuration::new_unchecked(
                FiniteF64::default(),
                FiniteF64::default(),
                FiniteF64::default(),
                day,
            ),
            time: *time,
        }
    }

    /// Creates a `Duration` from a provided `PartialDuration`.
    pub fn from_partial_duration(partial: PartialDuration) -> TemporalResult<Self> {
        if partial == PartialDuration::default() {
            return Err(TemporalError::r#type()
                .with_message("PartialDuration cannot have all empty fields."));
        }
        Self::new(
            partial.years.unwrap_or_default(),
            partial.months.unwrap_or_default(),
            partial.weeks.unwrap_or_default(),
            partial.days.unwrap_or_default(),
            partial.hours.unwrap_or_default(),
            partial.minutes.unwrap_or_default(),
            partial.seconds.unwrap_or_default(),
            partial.milliseconds.unwrap_or_default(),
            partial.microseconds.unwrap_or_default(),
            partial.nanoseconds.unwrap_or_default(),
        )
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

    /// Returns the `years` field of duration.
    #[inline]
    #[must_use]
    pub const fn years(&self) -> FiniteF64 {
        self.date.years
    }

    /// Returns the `months` field of duration.
    #[inline]
    #[must_use]
    pub const fn months(&self) -> FiniteF64 {
        self.date.months
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn weeks(&self) -> FiniteF64 {
        self.date.weeks
    }

    /// Returns the `weeks` field of duration.
    #[inline]
    #[must_use]
    pub const fn days(&self) -> FiniteF64 {
        self.date.days
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn hours(&self) -> FiniteF64 {
        self.time.hours
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn minutes(&self) -> FiniteF64 {
        self.time.minutes
    }

    /// Returns the `seconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn seconds(&self) -> FiniteF64 {
        self.time.seconds
    }

    /// Returns the `hours` field of duration.
    #[inline]
    #[must_use]
    pub const fn milliseconds(&self) -> FiniteF64 {
        self.time.milliseconds
    }

    /// Returns the `microseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn microseconds(&self) -> FiniteF64 {
        self.time.microseconds
    }

    /// Returns the `nanoseconds` field of duration.
    #[inline]
    #[must_use]
    pub const fn nanoseconds(&self) -> FiniteF64 {
        self.time.nanoseconds
    }
}

// ==== Public Duration methods ====

impl Duration {
    /// Determines the sign for the current self.
    #[inline]
    #[must_use]
    pub fn sign(&self) -> Sign {
        duration_sign(&self.fields())
    }

    /// Returns whether the current `Duration` is zero.
    ///
    /// Equivalant to `Temporal.Duration.blank()`.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.sign() == Sign::Zero
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

    /// Returns the result of adding a `Duration` to the current `Duration`
    #[inline]
    pub fn add(&self, other: &Self) -> TemporalResult<Self> {
        // NOTE: Implemented from AddDurations
        // Steps 1-22 are functionally useless in this context.

        // 23. Let largestUnit1 be DefaultTemporalLargestUnit(y1, mon1, w1, d1, h1, min1, s1, ms1, mus1).
        let largest_one = self.default_largest_unit();
        // 24. Let largestUnit2 be DefaultTemporalLargestUnit(y2, mon2, w2, d2, h2, min2, s2, ms2, mus2).
        let largest_two = other.default_largest_unit();
        // 25. Let largestUnit be LargerOfTwoTemporalUnits(largestUnit1, largestUnit2).
        let largest_unit = largest_one.max(largest_two);
        // 26. Let norm1 be NormalizeTimeDuration(h1, min1, s1, ms1, mus1, ns1).
        let norm_one = NormalizedTimeDuration::from_time_duration(self.time());
        // 27. Let norm2 be NormalizeTimeDuration(h2, min2, s2, ms2, mus2, ns2).
        let norm_two = NormalizedTimeDuration::from_time_duration(other.time());

        // 28. If IsCalendarUnit(largestUnit), throw a RangeError exception.
        if largest_unit.is_calendar_unit() {
            return Err(TemporalError::range().with_message(
                "Largest unit cannot be a calendar unit when adding two durations.",
            ));
        }

        // 29. Let normResult be ? AddNormalizedTimeDuration(norm1, norm2).
        // 30. Set normResult to ? Add24HourDaysToNormalizedTimeDuration(normResult, d1 + d2).
        let result =
            (norm_one + norm_two)?.add_days((self.days().checked_add(&other.days())?).as_())?;

        // 31. Let result be ? BalanceTimeDuration(normResult, largestUnit).
        let (result_days, result_time) = TimeDuration::from_normalized(result, largest_unit)?;

        // 32. Return ! CreateTemporalDuration(0, 0, 0, result.[[Days]], result.[[Hours]], result.[[Minutes]],
        // result.[[Seconds]], result.[[Milliseconds]], result.[[Microseconds]], result.[[Nanoseconds]]).
        Ok(Duration::from_day_and_time(result_days, &result_time))
    }

    /// Returns the result of subtracting a `Duration` from the current `Duration`
    #[inline]
    pub fn subtract(&self, other: &Self) -> TemporalResult<Self> {
        self.add(&other.negated())
    }

    #[inline]
    pub fn round_with_provider(
        &self,
        options: RoundingOptions,
        relative_to: Option<RelativeTo>,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<Self> {
        // NOTE: Steps 1-14 seem to be implementation specific steps.
        // 14. Let roundingIncrement be ? ToTemporalRoundingIncrement(roundTo).
        // 15. Let roundingMode be ? ToTemporalRoundingMode(roundTo, "halfExpand").
        // 16. Let smallestUnit be ? GetTemporalUnit(roundTo, "smallestUnit", DATETIME, undefined).
        // 17. If smallestUnit is undefined, then
        // a. Set smallestUnitPresent to false.
        // b. Set smallestUnit to "nanosecond".
        // 18. Let existingLargestUnit be ! DefaultTemporalLargestUnit(duration.[[Years]],
        // duration.[[Months]], duration.[[Weeks]], duration.[[Days]], duration.[[Hours]],
        // duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]],
        // duration.[[Microseconds]]).
        // 19. Let defaultLargestUnit be LargerOfTwoTemporalUnits(existingLargestUnit, smallestUnit).
        // 20. If largestUnit is undefined, then
        // a. Set largestUnitPresent to false.
        // b. Set largestUnit to defaultLargestUnit.
        // 21. Else if largestUnit is "auto", then
        // a. Set largestUnit to defaultLargestUnit.
        // 23. If LargerOfTwoTemporalUnits(largestUnit, smallestUnit) is not largestUnit, throw a RangeError exception.
        // 24. Let maximum be MaximumTemporalDurationRoundingIncrement(smallestUnit).
        // 25. If maximum is not undefined, perform ? ValidateTemporalRoundingIncrement(roundingIncrement, maximum, false).
        let existing_largest_unit = self.default_largest_unit();
        let resolved_options =
            ResolvedRoundingOptions::from_duration_options(options, existing_largest_unit)?;

        let is_zoned_datetime = matches!(relative_to, Some(RelativeTo::ZonedDateTime(_)));

        // 26. Let hoursToDaysConversionMayOccur be false.
        // 27. If duration.[[Days]] ≠ 0 and zonedRelativeTo is not undefined, set hoursToDaysConversionMayOccur to true.
        // 28. Else if abs(duration.[[Hours]]) ≥ 24, set hoursToDaysConversionMayOccur to true.
        let hours_to_days_may_occur =
            (self.days() != 0.0 && is_zoned_datetime) || self.hours().abs() >= 24.0;

        // 29. If smallestUnit is "nanosecond" and roundingIncrement = 1, let roundingGranularityIsNoop
        // be true; else let roundingGranularityIsNoop be false.
        // 30. If duration.[[Years]] = 0 and duration.[[Months]] = 0 and duration.[[Weeks]] = 0,
        // let calendarUnitsPresent be false; else let calendarUnitsPresent be true.
        let calendar_units_present =
            !(self.years() == 0.0 && self.months() == 0.0 && self.weeks() == 0.0);

        let is_noop = resolved_options.is_noop();

        // 31. If roundingGranularityIsNoop is true, and largestUnit is existingLargestUnit, and calendarUnitsPresent is false,
        // and hoursToDaysConversionMayOccur is false, and abs(duration.[[Minutes]]) < 60, and abs(duration.[[Seconds]]) < 60,
        // and abs(duration.[[Milliseconds]]) < 1000, and abs(duration.[[Microseconds]]) < 1000, and abs(duration.[[Nanoseconds]]) < 1000, then
        if is_noop
            && resolved_options.largest_unit == existing_largest_unit
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
        // 34. If zonedRelativeTo is not undefined and plainDateTimeOrRelativeToWillBeUsed is true, then
        // 35. Let calendarRec be ? CreateCalendarMethodsRecordFromRelativeTo(plainRelativeTo, zonedRelativeTo, « DATE-ADD, DATE-UNTIL »).

        // 36. Let norm be NormalizeTimeDuration(duration.[[Hours]], duration.[[Minutes]], duration.[[Seconds]], duration.[[Milliseconds]],
        // duration.[[Microseconds]], duration.[[Nanoseconds]]).
        let norm = NormalizedTimeDuration::from_time_duration(self.time());
        // 37. Let emptyOptions be OrdinaryObjectCreate(null).

        match relative_to {
            // 38. If zonedRelativeTo is not undefined, then
            Some(RelativeTo::ZonedDateTime(zoned_datetime)) => {
                // a. Let relativeEpochNs be zonedRelativeTo.[[Nanoseconds]].
                // b. Let relativeInstant be ! CreateTemporalInstant(relativeEpochNs).
                // c. Let targetEpochNs be ? AddZonedDateTime(relativeInstant, timeZoneRec, calendarRec, duration.[[Years]], duration.[[Months]], duration.[[Weeks]], duration.[[Days]], norm, precalculatedPlainDateTime).
                let target_epoch_ns =
                    zoned_datetime.add_as_instant(self, ArithmeticOverflow::Constrain, provider)?;
                // d. Let roundRecord be ? DifferenceZonedDateTimeWithRounding(relativeEpochNs, targetEpochNs, calendarRec, timeZoneRec, precalculatedPlainDateTime, emptyOptions, largestUnit, roundingIncrement, smallestUnit, roundingMode).
                // e. Let roundResult be roundRecord.[[DurationRecord]].
                let internal = zoned_datetime.diff_with_rounding(
                    &ZonedDateTime::new_unchecked(
                        target_epoch_ns,
                        zoned_datetime.calendar().clone(),
                        zoned_datetime.timezone().clone(),
                    ),
                    resolved_options,
                    provider,
                )?;
                Duration::from_normalized(internal, resolved_options.largest_unit)
            }
            // 39. Else if plainRelativeTo is not undefined, then
            Some(RelativeTo::PlainDate(plain_date)) => {
                // a. Let targetTime be AddTime(0, 0, 0, 0, 0, 0, norm).
                let (balanced_days, time) = PlainTime::default().add_normalized_time_duration(norm);
                // b. Let dateDuration be ? CreateTemporalDuration(duration.[[Years]], duration.[[Months]], duration.[[Weeks]],
                // duration.[[Days]] + targetTime.[[Days]], 0, 0, 0, 0, 0, 0).
                let date_duration = DateDuration::new(
                    self.years(),
                    self.months(),
                    self.weeks(),
                    self.days().checked_add(&FiniteF64::from(balanced_days))?,
                )?;

                // c. Let targetDate be ? AddDate(calendarRec, plainRelativeTo, dateDuration).
                let target_date = plain_date.add_date(&Duration::from(date_duration), None)?;

                let plain_dt = PlainDateTime::new_unchecked(
                    IsoDateTime::new(plain_date.iso, IsoTime::default())?,
                    plain_date.calendar().clone(),
                );

                let target_dt = PlainDateTime::new_unchecked(
                    IsoDateTime::new(target_date.iso, time.iso)?,
                    target_date.calendar().clone(),
                );

                // d. Let roundRecord be ? DifferencePlainDateTimeWithRounding(plainRelativeTo.[[ISOYear]], plainRelativeTo.[[ISOMonth]],
                // plainRelativeTo.[[ISODay]], 0, 0, 0, 0, 0, 0, targetDate.[[ISOYear]], targetDate.[[ISOMonth]], targetDate.[[ISODay]],
                // targetTime.[[Hours]], targetTime.[[Minutes]], targetTime.[[Seconds]], targetTime.[[Milliseconds]],
                // targetTime.[[Microseconds]], targetTime.[[Nanoseconds]], calendarRec, largestUnit, roundingIncrement,
                // smallestUnit, roundingMode, emptyOptions).
                let round_record = plain_dt.diff_dt_with_rounding(&target_dt, resolved_options)?;
                // e. Let roundResult be roundRecord.[[DurationRecord]].
                Duration::from_normalized(round_record, resolved_options.largest_unit)
            }
            // 40. Else,
            None => {
                // a. If calendarUnitsPresent is true, or IsCalendarUnit(largestUnit) is true, throw a RangeError exception.
                if calendar_units_present || resolved_options.largest_unit.is_calendar_unit() {
                    return Err(TemporalError::range().with_message(
                        "Calendar units cannot be present without a relative point.",
                    ));
                }
                // b. Assert: IsCalendarUnit(smallestUnit) is false.
                temporal_assert!(
                    !resolved_options.smallest_unit.is_calendar_unit(),
                    "Assertion failed: resolvedOptions contains a calendar unit\n{:?}",
                    resolved_options
                );

                // c. Let roundRecord be ? RoundTimeDuration(duration.[[Days]], norm, roundingIncrement, smallestUnit, roundingMode).
                let (round_record, _) = norm.round(self.days(), resolved_options)?;
                // d. Let normWithDays be ? Add24HourDaysToNormalizedTimeDuration(roundRecord.[[NormalizedDuration]].[[NormalizedTime]],
                // roundRecord.[[NormalizedDuration]].[[Days]]).
                let norm_with_days = round_record
                    .normalized_time_duration()
                    .add_days(round_record.date().days.as_())?;
                // e. Let balanceResult be ? BalanceTimeDuration(normWithDays, largestUnit).
                let (balanced_days, balanced_time) =
                    TimeDuration::from_normalized(norm_with_days, resolved_options.largest_unit)?;
                // f. Let roundResult be CreateDurationRecord(0, 0, 0, balanceResult.[[Days]], balanceResult.[[Hours]],
                // balanceResult.[[Minutes]], balanceResult.[[Seconds]], balanceResult.[[Milliseconds]],
                // balanceResult.[[Microseconds]], balanceResult.[[Nanoseconds]]).

                // 41. Return ? CreateTemporalDuration(roundResult.[[Years]], roundResult.[[Months]],
                // roundResult.[[Weeks]], roundResult.[[Days]], roundResult.[[Hours]],
                // roundResult.[[Minutes]], roundResult.[[Seconds]], roundResult.[[Milliseconds]],
                // roundResult.[[Microseconds]], roundResult.[[Nanoseconds]]).

                Ok(Duration::from_day_and_time(balanced_days, &balanced_time))
            }
        }
    }
}

#[cfg(feature = "experimental")]
impl Duration {
    pub fn round(
        &self,
        options: RoundingOptions,
        relative_to: Option<RelativeTo>,
    ) -> TemporalResult<Self> {
        let provider = TZ_PROVIDER
            .lock()
            .map_err(|_| TemporalError::general("Unable to acquire lock"))?;
        self.round_with_provider(options, relative_to, &*provider)
    }
}

// TODO: Update, optimize, and fix the below. is_valid_duration should probably be generic over a T.

// NOTE: Can FiniteF64 optimize the duration_validation
/// Utility function to check whether the `Duration` fields are valid.
#[inline]
#[must_use]
#[allow(clippy::too_many_arguments)]
pub(crate) fn is_valid_duration(
    years: FiniteF64,
    months: FiniteF64,
    weeks: FiniteF64,
    days: FiniteF64,
    hours: FiniteF64,
    minutes: FiniteF64,
    seconds: FiniteF64,
    milliseconds: FiniteF64,
    microseconds: FiniteF64,
    nanoseconds: FiniteF64,
) -> bool {
    // 1. Let sign be ! DurationSign(years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds).
    let set = vec![
        years,
        months,
        weeks,
        days,
        hours,
        minutes,
        seconds,
        milliseconds,
        microseconds,
        nanoseconds,
    ];
    let sign = duration_sign(&set);
    // 2. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in set {
        // FiniteF64 must always be finite.
        // a. If 𝔽(v) is not finite, return false.
        // b. If v < 0 and sign > 0, return false.
        if v < 0f64 && sign == Sign::Positive {
            return false;
        }
        // c. If v > 0 and sign < 0, return false.
        if v > 0f64 && sign == Sign::Negative {
            return false;
        }
    }
    // 3. If abs(years) ≥ 2**32, return false.
    if years.abs() >= f64::from(u32::MAX) {
        return false;
    };
    // 4. If abs(months) ≥ 2**32, return false.
    if months.abs() >= f64::from(u32::MAX) {
        return false;
    };
    // 5. If abs(weeks) ≥ 2**32, return false.
    if weeks.abs() >= f64::from(u32::MAX) {
        return false;
    };

    // 6. Let normalizedSeconds be days × 86,400 + hours × 3600 + minutes × 60 + seconds
    // + ℝ(𝔽(milliseconds)) × 10**-3 + ℝ(𝔽(microseconds)) × 10**-6 + ℝ(𝔽(nanoseconds)) × 10**-9.
    // 7. NOTE: The above step cannot be implemented directly using floating-point arithmetic.
    // Multiplying by 10**-3, 10**-6, and 10**-9 respectively may be imprecise when milliseconds,
    // microseconds, or nanoseconds is an unsafe integer. This multiplication can be implemented
    // in C++ with an implementation of core::remquo() with sufficient bits in the quotient.
    // String manipulation will also give an exact result, since the multiplication is by a power of 10.
    // Seconds part
    let normalized_seconds = days.0.mul_add(
        86_400.0,
        hours.0.mul_add(3600.0, minutes.0.mul_add(60.0, seconds.0)),
    );
    // Subseconds part
    let normalized_subseconds_parts = milliseconds.0.mul_add(
        10e-3,
        microseconds
            .0
            .mul_add(10e-6, nanoseconds.0.mul_add(10e-9, 0.0)),
    );

    let normalized_seconds = normalized_seconds + normalized_subseconds_parts;
    // 8. If abs(normalizedSeconds) ≥ 2**53, return false.
    if normalized_seconds.abs() >= 2e53 {
        return false;
    }

    // 9. Return true.
    true
}

/// Utility function for determining the sign for the current set of `Duration` fields.
///
/// Equivalent: 7.5.10 `DurationSign ( years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds )`
#[inline]
#[must_use]
fn duration_sign(set: &Vec<FiniteF64>) -> Sign {
    // 1. For each value v of « years, months, weeks, days, hours, minutes, seconds, milliseconds, microseconds, nanoseconds », do
    for v in set {
        // a. If v < 0, return -1.
        if *v < 0f64 {
            return Sign::Negative;
        // b. If v > 0, return 1.
        } else if *v > 0f64 {
            return Sign::Positive;
        }
    }
    // 2. Return 0.
    Sign::Zero
}

impl From<TimeDuration> for Duration {
    fn from(value: TimeDuration) -> Self {
        Self {
            time: value,
            date: DateDuration::default(),
        }
    }
}

impl From<DateDuration> for Duration {
    fn from(value: DateDuration) -> Self {
        Self {
            date: value,
            time: TimeDuration::default(),
        }
    }
}

// ==== FromStr trait impl ====

impl FromStr for Duration {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = IsoDurationParser::from_str(s)
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

        Self::new(
            FiniteF64::from(years).copysign(sign),
            FiniteF64::from(months).copysign(sign),
            FiniteF64::from(weeks).copysign(sign),
            FiniteF64::from(days).copysign(sign),
            FiniteF64::try_from(hours)?.copysign(sign),
            FiniteF64::try_from(minutes)?.copysign(sign),
            FiniteF64::try_from(seconds)?.copysign(sign),
            FiniteF64::try_from(millis)?.copysign(sign),
            FiniteF64::try_from(micros)?.copysign(sign),
            FiniteF64::try_from(nanos)?.copysign(sign),
        )
    }
}
