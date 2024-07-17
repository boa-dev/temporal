//! This module implements `DateTime` any directly related algorithms.

use crate::{
    components::{calendar::Calendar, duration::TimeDuration, Instant},
    iso::{IsoDate, IsoDateSlots, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, DifferenceOperation, DifferenceSettings, ResolvedRoundingOptions,
        TemporalUnit,
    },
    parsers::parse_date_time,
    temporal_assert, Sign, TemporalError, TemporalResult, TemporalUnwrap,
};

use std::{cmp::Ordering, str::FromStr};
use tinystr::TinyAsciiStr;

use super::{
    calendar::{CalendarDateLike, GetTemporalCalendar},
    duration::normalized::{NormalizedTimeDuration, RelativeRoundResult},
    Date, Duration, Time,
};

/// The native Rust implementation of `Temporal.PlainDateTime`
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct DateTime {
    pub(crate) iso: IsoDateTime,
    calendar: Calendar,
}

// ==== Private DateTime API ====

impl DateTime {
    /// Creates a new unchecked `DateTime`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDateTime, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    #[inline]
    #[must_use]
    /// Utility function for validating `IsoDate`s
    fn validate_iso(iso: IsoDate) -> bool {
        IsoDateTime::new_unchecked(iso, IsoTime::noon()).is_within_limits()
    }

    /// Create a new `DateTime` from an `Instant`.
    #[inline]
    pub(crate) fn from_instant(
        instant: &Instant,
        offset: f64,
        calendar: Calendar,
    ) -> TemporalResult<Self> {
        let iso = IsoDateTime::from_epoch_nanos(&instant.nanos, offset)?;
        Ok(Self { iso, calendar })
    }

    // 5.5.14 AddDurationToOrSubtractDurationFromPlainDateTime ( operation, dateTime, temporalDurationLike, options )
    fn add_or_subtract_duration(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        // SKIP: 1, 2, 3, 4
        // 1. If operation is subtract, let sign be -1. Otherwise, let sign be 1.
        // 2. Let duration be ? ToTemporalDurationRecord(temporalDurationLike).
        // 3. Set options to ? GetOptionsObject(options).
        // 4. Let calendarRec be ? CreateCalendarMethodsRecord(dateTime.[[Calendar]], « date-add »).

        // 5. Let norm be NormalizeTimeDuration(sign × duration.[[Hours]], sign × duration.[[Minutes]], sign × duration.[[Seconds]], sign × duration.[[Milliseconds]], sign × duration.[[Microseconds]], sign × duration.[[Nanoseconds]]).
        let norm = NormalizedTimeDuration::from_time_duration(duration.time());

        // TODO: validate Constrain is default with all the recent changes.
        // 6. Let result be ? AddDateTime(dateTime.[[ISOYear]], dateTime.[[ISOMonth]], dateTime.[[ISODay]], dateTime.[[ISOHour]], dateTime.[[ISOMinute]], dateTime.[[ISOSecond]], dateTime.[[ISOMillisecond]], dateTime.[[ISOMicrosecond]], dateTime.[[ISONanosecond]], calendarRec, sign × duration.[[Years]], sign × duration.[[Months]], sign × duration.[[Weeks]], sign × duration.[[Days]], norm, options).
        let result =
            self.iso
                .add_date_duration(self.calendar().clone(), duration.date(), norm, overflow)?;

        // 7. Assert: IsValidISODate(result.[[Year]], result.[[Month]], result.[[Day]]) is true.
        // 8. Assert: IsValidTime(result.[[Hour]], result.[[Minute]], result.[[Second]], result.[[Millisecond]],
        // result.[[Microsecond]], result.[[Nanosecond]]) is true.
        temporal_assert!(
            result.is_within_limits(),
            "Assertion failed: the below datetime is not within valid limits:\n{:?}",
            result
        );

        // 9. Return ? CreateTemporalDateTime(result.[[Year]], result.[[Month]], result.[[Day]], result.[[Hour]],
        // result.[[Minute]], result.[[Second]], result.[[Millisecond]], result.[[Microsecond]],
        // result.[[Nanosecond]], dateTime.[[Calendar]]).
        Ok(Self::new_unchecked(result, self.calendar.clone()))
    }

    /// Difference two `DateTime`s together.
    pub(crate) fn diff(
        &self,
        op: DifferenceOperation,
        other: &Self,
        settings: DifferenceSettings,
    ) -> TemporalResult<Duration> {
        // 3. If ? CalendarEquals(dateTime.[[Calendar]], other.[[Calendar]]) is false, throw a RangeError exception.
        if self.calendar != other.calendar {
            return Err(TemporalError::range()
                .with_message("Calendar must be the same when diffing two DateTimes"));
        }

        // 5. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, datetime, « », "nanosecond", "day").
        let (sign, options) = ResolvedRoundingOptions::from_diff_settings(
            settings,
            op,
            TemporalUnit::Day,
            TemporalUnit::Nanosecond,
        )?;

        // Step 7-8 combined.
        if self.iso == other.iso {
            return Ok(Duration::default());
        }

        // Step 10-11.
        let (result, _) = self.diff_dt_with_rounding(other, options)?;

        // Step 12
        match sign {
            Sign::Positive | Sign::Zero => Ok(result),
            Sign::Negative => Ok(result.negated()),
        }
    }

    // TODO: Figure out whether to handle resolvedOptions
    // 5.5.12 DifferencePlainDateTimeWithRounding ( y1, mon1, d1, h1, min1, s1, ms1, mus1, ns1, y2, mon2, d2, h2, min2, s2, ms2,
    // mus2, ns2, calendarRec, largestUnit, roundingIncrement, smallestUnit, roundingMode, resolvedOptions )
    pub(crate) fn diff_dt_with_rounding(
        &self,
        other: &Self,
        options: ResolvedRoundingOptions,
    ) -> TemporalResult<RelativeRoundResult> {
        // 1. Assert: IsValidISODate(y1, mon1, d1) is true.
        // 2. Assert: IsValidISODate(y2, mon2, d2) is true.
        // 3. If CompareISODateTime(y1, mon1, d1, h1, min1, s1, ms1, mus1, ns1, y2, mon2, d2, h2, min2, s2, ms2, mus2, ns2) = 0, then
        if matches!(self.iso.cmp(&other.iso), Ordering::Equal) {
            // a. Let durationRecord be CreateDurationRecord(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).
            // b. Return the Record { [[DurationRecord]]: durationRecord, [[Total]]: 0 }.
            return Ok((Duration::default(), Some(0)));
        }

        // 4. Let diff be ? DifferenceISODateTime(y1, mon1, d1, h1, min1, s1, ms1, mus1, ns1, y2, mon2, d2, h2, min2, s2, ms2, mus2, ns2, calendarRec, largestUnit, resolvedOptions).
        let diff = self
            .iso
            .diff(&other.iso, &self.calendar, options.largest_unit)?;

        // 5. If smallestUnit is "nanosecond" and roundingIncrement = 1, then
        if options.smallest_unit == TemporalUnit::Nanosecond && options.increment.get() == 1 {
            // a. Let normWithDays be ? Add24HourDaysToNormalizedTimeDuration(diff.[[NormalizedTime]], diff.[[Days]]).
            let norm_with_days = diff
                .normalized_time_duration()
                .add_days(diff.date().days as i64)?;
            // b. Let timeResult be ! BalanceTimeDuration(normWithDays, largestUnit).
            let (days, time_duration) =
                TimeDuration::from_normalized(norm_with_days, options.largest_unit)?;

            // c. Let total be NormalizedTimeDurationSeconds(normWithDays) × 10**9 + NormalizedTimeDurationSubseconds(normWithDays).
            let total =
                norm_with_days.seconds() * 1_000_000_000 + i64::from(norm_with_days.subseconds());

            // d. Let durationRecord be CreateDurationRecord(diff.[[Years]], diff.[[Months]], diff.[[Weeks]], timeResult.[[Days]],
            // timeResult.[[Hours]], timeResult.[[Minutes]], timeResult.[[Seconds]], timeResult.[[Milliseconds]],
            // timeResult.[[Microseconds]], timeResult.[[Nanoseconds]]).
            let duration = Duration::new(
                diff.date().years,
                diff.date().months,
                diff.date().weeks,
                days,
                time_duration.hours,
                time_duration.minutes,
                time_duration.seconds,
                time_duration.milliseconds,
                time_duration.microseconds,
                time_duration.nanoseconds,
            )?;

            // e. Return the Record { [[DurationRecord]]: durationRecord, [[Total]]: total }.
            return Ok((duration, Some(i128::from(total))));
        }

        // 6. Let dateTime be ISO Date-TimeRecord { [[Year]]: y1, [[Month]]: mon1,
        // [[Day]]: d1, [[Hour]]: h1, [[Minute]]: min1, [[Second]]: s1, [[Millisecond]]:
        // ms1, [[Microsecond]]: mus1, [[Nanosecond]]: ns1 }.
        // 7. Let destEpochNs be GetUTCEpochNanoseconds(y2, mon2, d2, h2, min2, s2, ms2, mus2, ns2).
        let dest_epoch_ns = other.iso.as_nanoseconds(0.0).temporal_unwrap()?;

        // 8. Return ? RoundRelativeDuration(diff, destEpochNs, dateTime, calendarRec, unset, largestUnit,
        // roundingIncrement, smallestUnit, roundingMode).
        diff.round_relative_duration(dest_epoch_ns, self, None, options)
    }
}

// ==== Public DateTime API ====

impl DateTime {
    /// Creates a new validated `DateTime`.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        year: i32,
        month: i32,
        day: i32,
        hour: i32,
        minute: i32,
        second: i32,
        millisecond: i32,
        microsecond: i32,
        nanosecond: i32,
        calendar: Calendar,
    ) -> TemporalResult<Self> {
        let iso_date = IsoDate::new(year, month, day, ArithmeticOverflow::Reject)?;
        let iso_time = IsoTime::new(
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            ArithmeticOverflow::Reject,
        )?;
        Ok(Self::new_unchecked(
            IsoDateTime::new(iso_date, iso_time)?,
            calendar,
        ))
    }

    /// Creates a new `DateTime` from the current `DateTime` and the provided `Time`.
    pub fn with_time(&self, time: Time) -> TemporalResult<Self> {
        Self::new(
            self.iso_year(),
            self.iso_month().into(),
            self.iso_day().into(),
            time.hour().into(),
            time.minute().into(),
            time.second().into(),
            time.millisecond().into(),
            time.microsecond().into(),
            time.nanosecond().into(),
            self.calendar.clone(),
        )
    }

    /// Creates a new `DateTime` from the current `DateTime` and a provided `Calendar`.
    pub fn with_calendar(&self, calendar: Calendar) -> TemporalResult<Self> {
        Self::new(
            self.iso_year(),
            self.iso_month().into(),
            self.iso_day().into(),
            self.hour().into(),
            self.minute().into(),
            self.second().into(),
            self.millisecond().into(),
            self.microsecond().into(),
            self.nanosecond().into(),
            calendar,
        )
    }

    /// Validates whether ISO date slots are within iso limits at noon.
    #[inline]
    pub fn validate<T: IsoDateSlots>(target: &T) -> bool {
        Self::validate_iso(target.iso_date())
    }

    /// Returns this `Date`'s ISO year value.
    #[inline]
    #[must_use]
    pub const fn iso_year(&self) -> i32 {
        self.iso.date.year
    }

    /// Returns this `Date`'s ISO month value.
    #[inline]
    #[must_use]
    pub const fn iso_month(&self) -> u8 {
        self.iso.date.month
    }

    /// Returns this `Date`'s ISO day value.
    #[inline]
    #[must_use]
    pub const fn iso_day(&self) -> u8 {
        self.iso.date.day
    }

    /// Returns the hour value
    #[inline]
    #[must_use]
    pub fn hour(&self) -> u8 {
        self.iso.time.hour
    }

    /// Returns the minute value
    #[inline]
    #[must_use]
    pub fn minute(&self) -> u8 {
        self.iso.time.minute
    }

    /// Returns the second value
    #[inline]
    #[must_use]
    pub fn second(&self) -> u8 {
        self.iso.time.second
    }

    /// Returns the `millisecond` value
    #[inline]
    #[must_use]
    pub fn millisecond(&self) -> u16 {
        self.iso.time.millisecond
    }

    /// Returns the `microsecond` value
    #[inline]
    #[must_use]
    pub fn microsecond(&self) -> u16 {
        self.iso.time.microsecond
    }

    /// Returns the `nanosecond` value
    #[inline]
    #[must_use]
    pub fn nanosecond(&self) -> u16 {
        self.iso.time.nanosecond
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }
}

// ==== Calendar-derived public API ====

impl DateTime {
    /// Returns the calendar year value.
    pub fn year(&self) -> TemporalResult<i32> {
        self.calendar
            .year(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar month value.
    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar
            .month(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar month code value.
    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.calendar
            .month_code(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar day value.
    pub fn day(&self) -> TemporalResult<u8> {
        self.calendar.day(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        self.calendar
            .day_of_week(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        self.calendar
            .day_of_year(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<u16> {
        self.calendar
            .week_of_year(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<i32> {
        self.calendar
            .year_of_week(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_week(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_month(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_year(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .months_in_year(&CalendarDateLike::DateTime(self.clone()))
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        self.calendar
            .in_leap_year(&CalendarDateLike::DateTime(self.clone()))
    }

    #[inline]
    /// Adds a `Duration` to the current `DateTime`.
    pub fn add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.add_or_subtract_duration(duration, overflow)
    }

    #[inline]
    /// Subtracts a `Duration` to the current `DateTime`.
    pub fn subtract(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.add_or_subtract_duration(&duration.negated(), overflow)
    }

    #[inline]
    /// Returns a `Duration` representing the period of time from this `DateTime` until the other `DateTime`.
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff(DifferenceOperation::Until, other, settings)
    }

    #[inline]
    /// Returns a `Duration` representing the period of time from this `DateTime` since the other `DateTime`.
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff(DifferenceOperation::Since, other, settings)
    }
}

// ==== Trait impls ====

impl GetTemporalCalendar for DateTime {
    fn get_calendar(&self) -> Calendar {
        self.calendar.clone()
    }
}

impl IsoDateSlots for DateTime {
    fn iso_date(&self) -> IsoDate {
        self.iso.date
    }
}

impl From<Date> for DateTime {
    fn from(value: Date) -> Self {
        DateTime::new_unchecked(
            IsoDateTime::new_unchecked(value.iso, IsoTime::default()),
            value.calendar().clone(),
        )
    }
}

impl FromStr for DateTime {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_date_time(s)?;

        let calendar = parse_record.calendar.unwrap_or("iso8601");

        let time = if let Some(time) = parse_record.time {
            IsoTime::from_components(
                i32::from(time.hour),
                i32::from(time.minute),
                i32::from(time.second),
                f64::from(time.nanosecond),
            )?
        } else {
            IsoTime::default()
        };

        let parsed_date = parse_record.date.temporal_unwrap()?;

        let date = IsoDate::new(
            parsed_date.year,
            parsed_date.month.into(),
            parsed_date.day.into(),
            ArithmeticOverflow::Reject,
        )?;

        Ok(Self::new_unchecked(
            IsoDateTime::new(date, time)?,
            Calendar::from_str(calendar)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::{
        components::{calendar::Calendar, duration::DateDuration, Duration},
        iso::{IsoDate, IsoTime},
        options::{DifferenceSettings, RoundingIncrement, TemporalRoundingMode, TemporalUnit},
    };

    use super::DateTime;

    #[test]
    #[allow(clippy::float_cmp)]
    fn plain_date_time_limits() {
        // This test is primarily to assert that the `expect` in the epoch methods is
        // valid, i.e., a valid instant is within the range of an f64.
        let negative_limit = DateTime::new(
            -271_821,
            4,
            19,
            0,
            0,
            0,
            0,
            0,
            0,
            Calendar::from_str("iso8601").unwrap(),
        );
        let positive_limit = DateTime::new(275_760, 9, 14, 0, 0, 0, 0, 0, 0, Calendar::default());

        assert!(negative_limit.is_err());
        assert!(positive_limit.is_err());
    }

    // options-undefined.js
    #[test]
    fn datetime_add_test() {
        let pdt =
            DateTime::new(2020, 1, 31, 12, 34, 56, 987, 654, 321, Calendar::default()).unwrap();

        let result = pdt
            .add(
                &Duration::from(DateDuration::new(0.0, 1.0, 0.0, 0.0).unwrap()),
                None,
            )
            .unwrap();

        assert_eq!(result.month(), Ok(2));
        assert_eq!(result.day(), Ok(29));
    }

    // options-undefined.js
    #[test]
    fn datetime_subtract_test() {
        let pdt =
            DateTime::new(2000, 3, 31, 12, 34, 56, 987, 654, 321, Calendar::default()).unwrap();

        let result = pdt
            .subtract(
                &Duration::from(DateDuration::new(0.0, 1.0, 0.0, 0.0).unwrap()),
                None,
            )
            .unwrap();

        assert_eq!(result.month(), Ok(2));
        assert_eq!(result.day(), Ok(29));
    }

    // subtract/hour-overflow.js
    #[test]
    fn datetime_subtract_hour_overflows() {
        let dt =
            DateTime::new(2019, 10, 29, 10, 46, 38, 271, 986, 102, Calendar::default()).unwrap();

        let result = dt.subtract(&Duration::hour(12.0), None).unwrap();

        assert_eq!(
            result.iso.date,
            IsoDate {
                year: 2019,
                month: 10,
                day: 28
            }
        );
        assert_eq!(
            result.iso.time,
            IsoTime {
                hour: 22,
                minute: 46,
                second: 38,
                millisecond: 271,
                microsecond: 986,
                nanosecond: 102
            }
        );

        let result = dt.add(&Duration::hour(-12.0), None).unwrap();

        assert_eq!(
            result.iso.date,
            IsoDate {
                year: 2019,
                month: 10,
                day: 28
            }
        );
        assert_eq!(
            result.iso.time,
            IsoTime {
                hour: 22,
                minute: 46,
                second: 38,
                millisecond: 271,
                microsecond: 986,
                nanosecond: 102
            }
        );
    }

    fn create_diff_setting(
        smallest: TemporalUnit,
        increment: u32,
        rounding_mode: TemporalRoundingMode,
    ) -> DifferenceSettings {
        DifferenceSettings {
            largest_unit: None,
            smallest_unit: Some(smallest),
            increment: Some(RoundingIncrement::try_new(increment).unwrap()),
            rounding_mode: Some(rounding_mode),
        }
    }

    #[test]
    fn dt_until_basic() {
        let earlier =
            DateTime::new(2019, 1, 8, 8, 22, 36, 123, 456, 789, Calendar::default()).unwrap();
        let later =
            DateTime::new(2021, 9, 7, 12, 39, 40, 987, 654, 321, Calendar::default()).unwrap();

        let settings = create_diff_setting(TemporalUnit::Hour, 3, TemporalRoundingMode::HalfExpand);
        let result = earlier.until(&later, settings).unwrap();

        assert_eq!(result.days(), 973.0);
        assert_eq!(result.hours(), 3.0);

        let settings =
            create_diff_setting(TemporalUnit::Minute, 30, TemporalRoundingMode::HalfExpand);
        let result = earlier.until(&later, settings).unwrap();

        assert_eq!(result.days(), 973.0);
        assert_eq!(result.hours(), 4.0);
        assert_eq!(result.minutes(), 30.0);
    }

    #[test]
    fn dt_since_basic() {
        let earlier =
            DateTime::new(2019, 1, 8, 8, 22, 36, 123, 456, 789, Calendar::default()).unwrap();
        let later =
            DateTime::new(2021, 9, 7, 12, 39, 40, 987, 654, 321, Calendar::default()).unwrap();

        let settings = create_diff_setting(TemporalUnit::Hour, 3, TemporalRoundingMode::HalfExpand);
        let result = later.since(&earlier, settings).unwrap();

        assert_eq!(result.days(), 973.0);
        assert_eq!(result.hours(), 3.0);

        let settings =
            create_diff_setting(TemporalUnit::Minute, 30, TemporalRoundingMode::HalfExpand);
        let result = later.since(&earlier, settings).unwrap();

        assert_eq!(result.days(), 973.0);
        assert_eq!(result.hours(), 4.0);
        assert_eq!(result.minutes(), 30.0);
    }
}
