//! This module implements `DateTime` any directly related algorithms.

use crate::{
    components::{calendar::Calendar, duration::TimeDuration, Instant},
    iso::{IsoDate, IsoDateSlots, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, DifferenceOperation, DifferenceSettings, ResolvedRoundingOptions,
        RoundingOptions, TemporalUnit,
    },
    parsers::parse_date_time,
    temporal_assert, Sign, TemporalError, TemporalResult, TemporalUnwrap,
};

use core::{cmp::Ordering, str::FromStr};
use num_traits::AsPrimitive;
use tinystr::TinyAsciiStr;

use super::{
    calendar::{CalendarDateLike, GetTemporalCalendar},
    duration::normalized::{NormalizedTimeDuration, RelativeRoundResult},
    Duration, PartialDate, PartialTime, PlainDate, PlainTime,
};

/// A partial PlainDateTime record
#[derive(Debug, Default, Copy, Clone)]
pub struct PartialDateTime {
    /// The `PartialDate` portion of a `PartialDateTime`
    pub date: PartialDate,
    /// The `PartialTime` portion of a `PartialDateTime`
    pub time: PartialTime,
}

/// The native Rust implementation of `Temporal.PlainDateTime`
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PlainDateTime {
    pub(crate) iso: IsoDateTime,
    calendar: Calendar,
}

impl Ord for PlainDateTime {
    fn cmp(&self, other: &Self) -> Ordering {
        self.iso.cmp(&other.iso)
    }
}

impl PartialOrd for PlainDateTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// ==== Private PlainDateTime API ====

impl PlainDateTime {
    /// Creates a new unchecked `DateTime`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDateTime, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    // TODO: Potentially deprecate and remove.
    /// Utility function for validating `IsoDate`s
    #[inline]
    #[must_use]
    fn validate_iso(iso: IsoDate) -> bool {
        IsoDateTime::new_unchecked(iso, IsoTime::noon()).is_within_limits()
    }

    // TODO: Potentially deprecate and remove.
    /// Create a new `DateTime` from an `Instant`.
    #[inline]
    #[allow(unused)]
    pub(crate) fn from_instant(
        instant: &Instant,
        offset: i64,
        calendar: Calendar,
    ) -> TemporalResult<Self> {
        let iso = IsoDateTime::from_epoch_nanos(&instant.as_i128(), offset)?;
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
                .with_message("Calendar must be the same when diffing two PlainDateTimes"));
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
                .add_days(diff.date().days.as_())?;
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
        let dest_epoch_ns = other.iso.as_nanoseconds()?;

        // 8. Return ? RoundRelativeDuration(diff, destEpochNs, dateTime, calendarRec, unset, largestUnit,
        // roundingIncrement, smallestUnit, roundingMode).
        diff.round_relative_duration(dest_epoch_ns.0, self, None, options)
    }
}

// ==== Public PlainDateTime API ====

impl PlainDateTime {
    /// Creates a new `DateTime`, constraining any arguments that into a valid range.
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
        Self::new_with_overflow(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            calendar,
            ArithmeticOverflow::Constrain,
        )
    }

    /// Creates a new `DateTime`, rejecting any arguments that are not in a valid range.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
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
        Self::new_with_overflow(
            year,
            month,
            day,
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            calendar,
            ArithmeticOverflow::Reject,
        )
    }

    /// Creates a new `DateTime` with the provided [`ArithmeticOverflow`] option.
    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_overflow(
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
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso_date = IsoDate::new_with_overflow(year, month, day, overflow)?;
        let iso_time = IsoTime::new(
            hour,
            minute,
            second,
            millisecond,
            microsecond,
            nanosecond,
            overflow,
        )?;
        Ok(Self::new_unchecked(
            IsoDateTime::new(iso_date, iso_time)?,
            calendar,
        ))
    }

    /// Create a `DateTime` from a `Date` and a `Time`.
    pub fn from_date_and_time(date: PlainDate, time: PlainTime) -> TemporalResult<Self> {
        Ok(Self::new_unchecked(
            IsoDateTime::new(date.iso, time.iso)?,
            date.calendar().clone(),
        ))
    }

    /// Creates a `DateTime` from a `PartialDateTime`.
    ///
    /// ```rust
    /// use temporal_rs::{PlainDateTime, partial::{PartialDateTime, PartialTime, PartialDate}};
    ///
    /// let date = PartialDate {
    ///     year: Some(2000),
    ///     month: Some(13),
    ///     day: Some(2),
    ///     ..Default::default()
    /// };
    ///
    /// let time = PartialTime {
    ///     hour: Some(4),
    ///     minute: Some(25),
    ///     ..Default::default()
    /// };
    ///
    /// let partial = PartialDateTime { date, time };
    ///
    /// let date = PlainDateTime::from_partial(partial, None, None).unwrap();
    ///
    /// assert_eq!(date.year().unwrap(), 2000);
    /// assert_eq!(date.month().unwrap(), 12);
    /// assert_eq!(date.day().unwrap(), 2);
    /// assert_eq!(date.calendar().identifier(), "iso8601");
    /// assert_eq!(date.hour(), 4);
    /// assert_eq!(date.minute(), 25);
    /// assert_eq!(date.second(), 0);
    /// assert_eq!(date.millisecond(), 0);
    ///
    /// ```
    pub fn from_partial(
        partial: PartialDateTime,
        calendar: Option<Calendar>,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        let date = PlainDate::from_partial(partial.date, calendar, overflow)?;
        let time = PlainTime::from_partial(partial.time, overflow)?;
        Self::from_date_and_time(date, time)
    }

    /// Creates a new `DateTime` with the fields of a `PartialDateTime`.
    ///
    /// ```rust
    /// use temporal_rs::{Calendar, PlainDateTime, partial::{PartialDateTime, PartialTime, PartialDate}};
    ///
    /// let initial = PlainDateTime::try_new(2000, 12, 2, 0,0,0,0,0,0, Calendar::default()).unwrap();
    ///
    /// let date = PartialDate {
    ///     month: Some(5),
    ///     ..Default::default()
    /// };
    ///
    /// let time = PartialTime {
    ///     hour: Some(4),
    ///     second: Some(30),
    ///     ..Default::default()
    /// };
    ///
    /// let partial = PartialDateTime { date, time };
    ///
    /// let date = initial.with(partial, None).unwrap();
    ///
    /// assert_eq!(date.year().unwrap(), 2000);
    /// assert_eq!(date.month().unwrap(), 5);
    /// assert_eq!(date.day().unwrap(), 2);
    /// assert_eq!(date.calendar().identifier(), "iso8601");
    /// assert_eq!(date.hour(), 4);
    /// assert_eq!(date.minute(), 0);
    /// assert_eq!(date.second(), 30);
    /// assert_eq!(date.millisecond(), 0);
    ///
    /// ```
    #[inline]
    pub fn with(
        &self,
        partial_datetime: PartialDateTime,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        if partial_datetime.date.is_empty() && partial_datetime.time.is_empty() {
            return Err(
                TemporalError::r#type().with_message("A PartialDateTime must have a valid field.")
            );
        }

        let result_date = self.calendar.date_from_partial(
            &partial_datetime.date.with_fallback_datetime(self)?,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
        )?;

        // Determine the `Time` based off the partial values.
        let time = self.iso.time.with(
            partial_datetime.time,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
        )?;

        let iso_datetime = IsoDateTime::new(result_date.iso, time)?;

        Ok(Self::new_unchecked(iso_datetime, self.calendar().clone()))
    }

    /// Creates a new `DateTime` from the current `DateTime` and the provided `Time`.
    pub fn with_time(&self, time: PlainTime) -> TemporalResult<Self> {
        Self::try_new(
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
        Self::try_new(
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

impl PlainDateTime {
    /// Returns the calendar year value.
    pub fn year(&self) -> TemporalResult<i32> {
        self.calendar.year(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar month value.
    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar.month(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar month code value.
    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.calendar.month_code(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar day value.
    pub fn day(&self) -> TemporalResult<u8> {
        self.calendar.day(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        self.calendar.day_of_week(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        self.calendar.day_of_year(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<Option<u16>> {
        self.calendar
            .week_of_year(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<Option<i32>> {
        self.calendar
            .year_of_week(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_week(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_month(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .days_in_year(&CalendarDateLike::DateTime(self))
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        self.calendar
            .months_in_year(&CalendarDateLike::DateTime(self))
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        self.calendar
            .in_leap_year(&CalendarDateLike::DateTime(self))
    }

    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        self.calendar.era(&CalendarDateLike::DateTime(self))
    }

    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        self.calendar.era_year(&CalendarDateLike::DateTime(self))
    }
}

impl PlainDateTime {
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

    /// Rounds the current datetime based on provided options.
    pub fn round(&self, options: RoundingOptions) -> TemporalResult<Self> {
        let resolved = ResolvedRoundingOptions::from_dt_options(options)?;

        if resolved.is_noop() {
            return Ok(self.clone());
        }

        let result = self.iso.round(resolved)?;

        Ok(Self::new_unchecked(result, self.calendar.clone()))
    }
}

// ==== Trait impls ====

impl GetTemporalCalendar for PlainDateTime {
    fn get_calendar(&self) -> Calendar {
        self.calendar.clone()
    }
}

impl IsoDateSlots for PlainDateTime {
    fn iso_date(&self) -> IsoDate {
        self.iso.date
    }
}

impl From<PlainDate> for PlainDateTime {
    fn from(value: PlainDate) -> Self {
        PlainDateTime::new_unchecked(
            IsoDateTime::new_unchecked(value.iso, IsoTime::default()),
            value.calendar().clone(),
        )
    }
}

impl FromStr for PlainDateTime {
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

        let date = IsoDate::new_with_overflow(
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
    use tinystr::{tinystr, TinyAsciiStr};

    use crate::{
        components::{
            calendar::Calendar, duration::DateDuration, Duration, PartialDate, PartialDateTime,
            PartialTime, PlainDateTime,
        },
        iso::{IsoDate, IsoDateTime, IsoTime},
        options::{
            DifferenceSettings, RoundingIncrement, RoundingOptions, TemporalRoundingMode,
            TemporalUnit,
        },
        primitive::FiniteF64,
        TemporalResult,
    };

    fn assert_datetime(
        dt: PlainDateTime,
        fields: (i32, u8, TinyAsciiStr<4>, u8, u8, u8, u8, u16, u16, u16),
    ) {
        assert_eq!(dt.year().unwrap(), fields.0);
        assert_eq!(dt.month().unwrap(), fields.1);
        assert_eq!(dt.month_code().unwrap(), fields.2);
        assert_eq!(dt.day().unwrap(), fields.3);
        assert_eq!(dt.hour(), fields.4);
        assert_eq!(dt.minute(), fields.5);
        assert_eq!(dt.second(), fields.6);
        assert_eq!(dt.millisecond(), fields.7);
        assert_eq!(dt.microsecond(), fields.8);
        assert_eq!(dt.nanosecond(), fields.9);
    }

    fn pdt_from_date(year: i32, month: i32, day: i32) -> TemporalResult<PlainDateTime> {
        PlainDateTime::try_new(year, month, day, 0, 0, 0, 0, 0, 0, Calendar::default())
    }

    #[test]
    #[allow(clippy::float_cmp)]
    fn plain_date_time_limits() {
        // This test is primarily to assert that the `expect` in the epoch methods is
        // valid, i.e., a valid instant is within the range of an f64.
        let negative_limit = pdt_from_date(-271_821, 4, 19);
        assert!(negative_limit.is_err());
        let positive_limit = pdt_from_date(275_760, 9, 14);
        assert!(positive_limit.is_err());
        let within_negative_limit = pdt_from_date(-271_821, 4, 20);
        assert_eq!(
            within_negative_limit,
            Ok(PlainDateTime {
                iso: IsoDateTime {
                    date: IsoDate {
                        year: -271_821,
                        month: 4,
                        day: 20,
                    },
                    time: IsoTime::default(),
                },
                calendar: Calendar::default(),
            })
        );

        let within_positive_limit = pdt_from_date(275_760, 9, 13);
        assert_eq!(
            within_positive_limit,
            Ok(PlainDateTime {
                iso: IsoDateTime {
                    date: IsoDate {
                        year: 275_760,
                        month: 9,
                        day: 13,
                    },
                    time: IsoTime::default(),
                },
                calendar: Calendar::default(),
            })
        );
    }

    #[test]
    fn basic_with_test() {
        let pdt =
            PlainDateTime::try_new(1976, 11, 18, 15, 23, 30, 123, 456, 789, Calendar::default())
                .unwrap();

        // Test year
        let partial = PartialDateTime {
            date: PartialDate {
                year: Some(2019),
                ..Default::default()
            },
            time: PartialTime::default(),
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (2019, 11, tinystr!(4, "M11"), 18, 15, 23, 30, 123, 456, 789),
        );

        // Test month
        let partial = PartialDateTime {
            date: PartialDate {
                month: Some(5),
                ..Default::default()
            },
            time: PartialTime::default(),
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 5, tinystr!(4, "M05"), 18, 15, 23, 30, 123, 456, 789),
        );

        // Test monthCode
        let partial = PartialDateTime {
            date: PartialDate {
                month_code: Some(tinystr!(4, "M05")),
                ..Default::default()
            },
            time: PartialTime::default(),
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 5, tinystr!(4, "M05"), 18, 15, 23, 30, 123, 456, 789),
        );

        // Test day
        let partial = PartialDateTime {
            date: PartialDate {
                day: Some(5),
                ..Default::default()
            },
            time: PartialTime::default(),
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 11, tinystr!(4, "M11"), 5, 15, 23, 30, 123, 456, 789),
        );

        // Test hour
        let partial = PartialDateTime {
            date: PartialDate::default(),
            time: PartialTime {
                hour: Some(5),
                ..Default::default()
            },
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 11, tinystr!(4, "M11"), 18, 5, 23, 30, 123, 456, 789),
        );

        // Test minute
        let partial = PartialDateTime {
            date: PartialDate::default(),
            time: PartialTime {
                minute: Some(5),
                ..Default::default()
            },
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 11, tinystr!(4, "M11"), 18, 15, 5, 30, 123, 456, 789),
        );

        // Test second
        let partial = PartialDateTime {
            date: PartialDate::default(),
            time: PartialTime {
                second: Some(5),
                ..Default::default()
            },
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 11, tinystr!(4, "M11"), 18, 15, 23, 5, 123, 456, 789),
        );

        // Test second
        let partial = PartialDateTime {
            date: PartialDate::default(),
            time: PartialTime {
                millisecond: Some(5),
                ..Default::default()
            },
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 11, tinystr!(4, "M11"), 18, 15, 23, 30, 5, 456, 789),
        );

        // Test second
        let partial = PartialDateTime {
            date: PartialDate::default(),
            time: PartialTime {
                microsecond: Some(5),
                ..Default::default()
            },
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 11, tinystr!(4, "M11"), 18, 15, 23, 30, 123, 5, 789),
        );

        // Test second
        let partial = PartialDateTime {
            date: PartialDate::default(),
            time: PartialTime {
                nanosecond: Some(5),
                ..Default::default()
            },
        };
        let result = pdt.with(partial, None).unwrap();
        assert_datetime(
            result,
            (1976, 11, tinystr!(4, "M11"), 18, 15, 23, 30, 123, 456, 5),
        );
    }

    #[test]
    fn datetime_with_empty_partial() {
        let pdt =
            PlainDateTime::try_new(2020, 1, 31, 12, 34, 56, 987, 654, 321, Calendar::default())
                .unwrap();

        let err = pdt.with(PartialDateTime::default(), None);
        assert!(err.is_err());
    }

    // options-undefined.js
    #[test]
    fn datetime_add_test() {
        let pdt =
            PlainDateTime::try_new(2020, 1, 31, 12, 34, 56, 987, 654, 321, Calendar::default())
                .unwrap();

        let result = pdt
            .add(
                &Duration::from(
                    DateDuration::new(
                        FiniteF64::default(),
                        FiniteF64(1.0),
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )
                    .unwrap(),
                ),
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
            PlainDateTime::try_new(2000, 3, 31, 12, 34, 56, 987, 654, 321, Calendar::default())
                .unwrap();

        let result = pdt
            .subtract(
                &Duration::from(
                    DateDuration::new(
                        FiniteF64::default(),
                        FiniteF64(1.0),
                        FiniteF64::default(),
                        FiniteF64::default(),
                    )
                    .unwrap(),
                ),
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
            PlainDateTime::try_new(2019, 10, 29, 10, 46, 38, 271, 986, 102, Calendar::default())
                .unwrap();

        let result = dt.subtract(&Duration::hour(FiniteF64(12.0)), None).unwrap();
        assert_datetime(
            result,
            (2019, 10, tinystr!(4, "M10"), 28, 22, 46, 38, 271, 986, 102),
        );

        let result = dt.add(&Duration::hour(FiniteF64(-12.0)), None).unwrap();
        assert_datetime(
            result,
            (2019, 10, tinystr!(4, "M10"), 28, 22, 46, 38, 271, 986, 102),
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
            PlainDateTime::try_new(2019, 1, 8, 8, 22, 36, 123, 456, 789, Calendar::default())
                .unwrap();
        let later =
            PlainDateTime::try_new(2021, 9, 7, 12, 39, 40, 987, 654, 321, Calendar::default())
                .unwrap();

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
            PlainDateTime::try_new(2019, 1, 8, 8, 22, 36, 123, 456, 789, Calendar::default())
                .unwrap();
        let later =
            PlainDateTime::try_new(2021, 9, 7, 12, 39, 40, 987, 654, 321, Calendar::default())
                .unwrap();

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

    #[test]
    fn dt_round_basic() {
        let assert_datetime =
            |dt: PlainDateTime, expected: (i32, u8, u8, u8, u8, u8, u16, u16, u16)| {
                assert_eq!(dt.iso_year(), expected.0);
                assert_eq!(dt.iso_month(), expected.1);
                assert_eq!(dt.iso_day(), expected.2);
                assert_eq!(dt.hour(), expected.3);
                assert_eq!(dt.minute(), expected.4);
                assert_eq!(dt.second(), expected.5);
                assert_eq!(dt.millisecond(), expected.6);
                assert_eq!(dt.microsecond(), expected.7);
                assert_eq!(dt.nanosecond(), expected.8);
            };

        let gen_rounding_options = |smallest: TemporalUnit, increment: u32| -> RoundingOptions {
            RoundingOptions {
                largest_unit: None,
                smallest_unit: Some(smallest),
                increment: Some(RoundingIncrement::try_new(increment).unwrap()),
                rounding_mode: None,
            }
        };
        let dt =
            PlainDateTime::try_new(1976, 11, 18, 14, 23, 30, 123, 456, 789, Calendar::default())
                .unwrap();

        let result = dt
            .round(gen_rounding_options(TemporalUnit::Hour, 4))
            .unwrap();
        assert_datetime(result, (1976, 11, 18, 16, 0, 0, 0, 0, 0));

        let result = dt
            .round(gen_rounding_options(TemporalUnit::Minute, 15))
            .unwrap();
        assert_datetime(result, (1976, 11, 18, 14, 30, 0, 0, 0, 0));

        let result = dt
            .round(gen_rounding_options(TemporalUnit::Second, 30))
            .unwrap();
        assert_datetime(result, (1976, 11, 18, 14, 23, 30, 0, 0, 0));

        let result = dt
            .round(gen_rounding_options(TemporalUnit::Millisecond, 10))
            .unwrap();
        assert_datetime(result, (1976, 11, 18, 14, 23, 30, 120, 0, 0));

        let result = dt
            .round(gen_rounding_options(TemporalUnit::Microsecond, 10))
            .unwrap();
        assert_datetime(result, (1976, 11, 18, 14, 23, 30, 123, 460, 0));

        let result = dt
            .round(gen_rounding_options(TemporalUnit::Nanosecond, 10))
            .unwrap();
        assert_datetime(result, (1976, 11, 18, 14, 23, 30, 123, 456, 790));
    }
}
