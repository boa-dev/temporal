//! This module implements `Date` and any directly related algorithms.

use tinystr::TinyAsciiStr;

use crate::{
    components::{
        calendar::{Calendar, CalendarDateLike, GetTemporalCalendar},
        duration::DateDuration,
        Duration, PlainDateTime,
    },
    iso::{IsoDate, IsoDateSlots, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, DifferenceOperation, DifferenceSettings, ResolvedRoundingOptions,
        TemporalUnit,
    },
    parsers::parse_date_time,
    primitive::FiniteF64,
    Sign, TemporalError, TemporalResult, TemporalUnwrap,
};

use alloc::format;
use core::str::FromStr;

use super::{
    calendar::{ascii_four_to_integer, month_to_month_code},
    duration::{normalized::NormalizedDurationRecord, TimeDuration},
    PlainMonthDay, PlainTime, PlainYearMonth,
};

// TODO (potentially): Bump era up to TinyAsciiStr<18> to accomodate
// "ethiopic-amete-alem". TODO: PrepareTemporalFields expects a type
// error to be thrown when all partial fields are None/undefined.
/// A partial PlainDate that may or may not be complete.
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct PartialDate {
    // A potentially set `year` field.
    pub year: Option<i32>,
    // A potentially set `month` field.
    pub month: Option<i32>,
    // A potentially set `month_code` field.
    pub month_code: Option<TinyAsciiStr<4>>,
    // A potentially set `day` field.
    pub day: Option<i32>,
    // A potentially set `era` field.
    pub era: Option<TinyAsciiStr<19>>,
    // A potentially set `era_year` field.
    pub era_year: Option<i32>,
}

impl PartialDate {
    /// Returns a boolean for if the current `PartialDate` is empty.
    pub(crate) fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    pub(crate) fn try_from_year_month(year_month: &PlainYearMonth) -> TemporalResult<Self> {
        let (year, era, era_year) = if year_month.era()?.is_some() {
            (
                None,
                year_month
                    .era()?
                    .map(|t| TinyAsciiStr::<19>::from_bytes(t.as_bytes()))
                    .transpose()
                    .map_err(|e| TemporalError::general(format!("{e}")))?,
                year_month.era_year()?,
            )
        } else {
            (Some(year_month.year()?), None, None)
        };
        Ok(Self {
            year,
            month: Some(year_month.month()?.into()),
            month_code: Some(year_month.month_code()?),
            day: Some(1),
            era,
            era_year,
        })
    }

    crate::impl_with_fallback_method!(with_fallback_date, PlainDate);
    crate::impl_with_fallback_method!(with_fallback_datetime, PlainDateTime);
    // TODO: ZonedDateTime
}

// Use macro to impl fallback methods to avoid having a trait method.
#[doc(hidden)]
#[macro_export]
macro_rules! impl_with_fallback_method {
    ($method_name:ident, $component_type:ty) => {
        pub(crate) fn $method_name(&self, fallback: &$component_type) -> TemporalResult<Self> {
            let era = if let Some(era) = self.era {
                Some(era)
            } else {
                let era = fallback.era()?;
                era.map(|e| {
                    TinyAsciiStr::<19>::from_bytes(e.as_bytes())
                        .map_err(|e| TemporalError::general(format!("{e}")))
                })
                .transpose()?
            };
            let era_year = self
                .era_year
                .map_or_else(|| fallback.era_year(), |ey| Ok(Some(ey)))?;

            let (month, month_code) = match (self.month, self.month_code) {
                (Some(month), Some(mc)) => (Some(month), Some(mc)),
                (Some(month), None) => (Some(month), Some(month_to_month_code(month)?)),
                (None, Some(mc)) => (Some(ascii_four_to_integer(mc)?).map(Into::into), Some(mc)),
                (None, None) => (
                    Some(fallback.month()?).map(Into::into),
                    Some(fallback.month_code()?),
                ),
            };

            Ok(Self {
                year: Some(self.year.unwrap_or(fallback.year()?)),
                month,
                month_code,
                day: Some(self.day.unwrap_or(fallback.day()?.into())),
                era,
                era_year,
            })
        }
    };
}

/// The native Rust implementation of `Temporal.PlainDate`.
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PlainDate {
    pub(crate) iso: IsoDate,
    calendar: Calendar,
}

impl Ord for PlainDate {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.iso.cmp(&other.iso)
    }
}

impl PartialOrd for PlainDate {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// ==== Private API ====

impl PlainDate {
    /// Create a new `Date` with the date values and calendar slot.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    /// Returns the date after adding the given duration to date.
    ///
    /// Temporal Equivalent: 3.5.13 `AddDate ( calendar, plainDate, duration [ , options [ , dateAdd ] ] )`
    #[inline]
    pub(crate) fn add_date(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        // 2. If options is not present, set options to undefined.
        let overflow = overflow.unwrap_or(ArithmeticOverflow::Constrain);
        // 3. If duration.[[Years]] ≠ 0, or duration.[[Months]] ≠ 0, or duration.[[Weeks]] ≠ 0, then
        if duration.date().years != 0.0
            || duration.date().months != 0.0
            || duration.date().weeks != 0.0
        {
            // a. If dateAdd is not present, then
            // i. Set dateAdd to unused.
            // ii. If calendar is an Object, set dateAdd to ? GetMethod(calendar, "dateAdd").
            // b. Return ? CalendarDateAdd(calendar, plainDate, duration, options, dateAdd).
            return self.calendar().date_add(self, duration, overflow);
        }

        // 4. Let overflow be ? ToTemporalOverflow(options).
        // 5. Let norm be NormalizeTimeDuration(duration.[[Hours]],
        //    duration.[[Minutes]], duration.[[Seconds]],
        //    duration.[[Milliseconds]], duration.[[Microseconds]],
        //    duration.[[Nanoseconds]]).
        // 6. Let days be duration.[[Days]] + BalanceTimeDuration(norm,
        //    "day").[[Days]].
        let days = duration.days().checked_add(
            &TimeDuration::from_normalized(duration.time().to_normalized(), TemporalUnit::Day)?.0,
        )?;

        // 7. Let result be ? AddISODate(plainDate.[[ISOYear]], plainDate.[[ISOMonth]], plainDate.[[ISODay]], 0, 0, 0, days, overflow).
        let result = self.iso.add_date_duration(
            &DateDuration::new(
                FiniteF64::default(),
                FiniteF64::default(),
                FiniteF64::default(),
                days,
            )?,
            overflow,
        )?;

        Ok(Self::new_unchecked(result, self.calendar().clone()))
    }

    /// Returns a duration representing the difference between the dates one and two.
    ///
    /// Temporal Equivalent: 3.5.6 `DifferenceDate ( calendar, one, two, options )`
    #[inline]
    pub(crate) fn internal_diff_date(
        &self,
        other: &Self,
        largest_unit: TemporalUnit,
    ) -> TemporalResult<Duration> {
        if self.iso.year == other.iso.year
            && self.iso.month == other.iso.month
            && self.iso.day == other.iso.day
        {
            return Ok(Duration::default());
        }

        if largest_unit == TemporalUnit::Day {
            let days = self.days_until(other);
            return Ok(Duration::from(DateDuration::new(
                FiniteF64::default(),
                FiniteF64::default(),
                FiniteF64::default(),
                FiniteF64::from(days),
            )?));
        }

        self.calendar().date_until(self, other, largest_unit)
    }

    /// Equivalent: DifferenceTemporalPlainDate
    pub(crate) fn diff_date(
        &self,
        op: DifferenceOperation,
        other: &Self,
        settings: DifferenceSettings,
    ) -> TemporalResult<Duration> {
        // 1. If operation is SINCE, let sign be -1. Otherwise, let sign be 1.
        // 2. Set other to ? ToTemporalDate(other).

        // 3. If ? CalendarEquals(temporalDate.[[Calendar]], other.[[Calendar]]) is false, throw a RangeError exception.
        if self.calendar().identifier() != other.calendar().identifier() {
            return Err(TemporalError::range()
                .with_message("Calendars are for difference operation are not the same."));
        }

        // 4. Let resolvedOptions be ? SnapshotOwnProperties(? GetOptionsObject(options), null).
        // 5. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, DATE, « », "day", "day").
        let (sign, resolved) = ResolvedRoundingOptions::from_diff_settings(
            settings,
            op,
            TemporalUnit::Day,
            TemporalUnit::Day,
        )?;

        // 6. If temporalDate.[[ISOYear]] = other.[[ISOYear]], and temporalDate.[[ISOMonth]] = other.[[ISOMonth]],
        // and temporalDate.[[ISODay]] = other.[[ISODay]], then
        if self.iso == other.iso {
            // a. Return ! CreateTemporalDuration(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).
            return Ok(Duration::default());
        }

        // 7. Let calendarRec be ? CreateCalendarMethodsRecord(temporalDate.[[Calendar]], « DATE-ADD, DATE-UNTIL »).
        // 8. Perform ! CreateDataPropertyOrThrow(resolvedOptions, "largestUnit", settings.[[LargestUnit]]).
        // 9. Let result be ? DifferenceDate(calendarRec, temporalDate, other, resolvedOptions).
        let result = self.internal_diff_date(other, resolved.largest_unit)?;

        // 10. Let duration be ! CreateNormalizedDurationRecord(result.[[Years]], result.[[Months]], result.[[Weeks]], result.[[Days]], ZeroTimeDuration()).
        let duration = NormalizedDurationRecord::from_date_duration(*result.date())?;
        // 11. If settings.[[SmallestUnit]] is "day" and settings.[[RoundingIncrement]] = 1, let roundingGranularityIsNoop be true; else let roundingGranularityIsNoop be false.
        let rounding_granularity_is_noop =
            resolved.smallest_unit == TemporalUnit::Day && resolved.increment.get() == 1;
        // 12. If roundingGranularityIsNoop is false, then
        let date_duration = if !rounding_granularity_is_noop {
            // a. Let destEpochNs be GetUTCEpochNanoseconds(other.[[ISOYear]], other.[[ISOMonth]], other.[[ISODay]], 0, 0, 0, 0, 0, 0).
            let dest_epoch_ns = other.iso.as_nanoseconds()?;
            // b. Let dateTime be ISO Date-Time Record { [[Year]]: temporalDate.[[ISOYear]], [[Month]]: temporalDate.[[ISOMonth]], [[Day]]: temporalDate.[[ISODay]], [[Hour]]: 0, [[Minute]]: 0, [[Second]]: 0, [[Millisecond]]: 0, [[Microsecond]]: 0, [[Nanosecond]]: 0 }.
            let dt = PlainDateTime::new_unchecked(
                IsoDateTime::new_unchecked(self.iso, IsoTime::default()),
                self.calendar.clone(),
            );
            // c. Set duration to ? RoundRelativeDuration(duration, destEpochNs, dateTime, calendarRec, unset, settings.[[LargestUnit]], settings.[[RoundingIncrement]], settings.[[SmallestUnit]], settings.[[RoundingMode]]).
            *duration
                .round_relative_duration(dest_epoch_ns.0, &dt, None, resolved)?
                .0
                .date()
        } else {
            duration.date()
        };

        // 13. Return ! CreateTemporalDuration(sign × duration.[[Years]], sign × duration.[[Months]], sign × duration.[[Weeks]], sign × duration.[[Days]], 0, 0, 0, 0, 0, 0).
        match sign {
            Sign::Positive | Sign::Zero => Ok(Duration::from(date_duration)),
            Sign::Negative => Ok(Duration::from(date_duration.negated())),
        }
    }
}

// ==== Public API ====

impl PlainDate {
    /// Creates a new `PlainDate` automatically constraining any values that may be invalid.
    pub fn new(year: i32, month: i32, day: i32, calendar: Calendar) -> TemporalResult<Self> {
        Self::new_with_overflow(year, month, day, calendar, ArithmeticOverflow::Constrain)
    }

    /// Creates a new `PlainDate` rejecting any date that may be invalid.
    pub fn try_new(year: i32, month: i32, day: i32, calendar: Calendar) -> TemporalResult<Self> {
        Self::new_with_overflow(year, month, day, calendar, ArithmeticOverflow::Reject)
    }

    /// Creates a new `PlainDate` with the specified overflow.
    ///
    /// This operation is the public facing API to Temporal's `RegulateIsoDate`
    #[inline]
    pub fn new_with_overflow(
        year: i32,
        month: i32,
        day: i32,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let iso = IsoDate::new_with_overflow(year, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
    }

    /// Create a `PlainDate` from a `PartialDate`
    ///
    /// ```rust
    /// use temporal_rs::{PlainDate, partial::PartialDate};
    ///
    /// let partial = PartialDate {
    ///     year: Some(2000),
    ///     month: Some(13),
    ///     day: Some(2),
    ///     ..Default::default()
    /// };
    ///
    /// let date = PlainDate::from_partial(partial, None, None).unwrap();
    ///
    /// assert_eq!(date.year().unwrap(), 2000);
    /// assert_eq!(date.month().unwrap(), 12);
    /// assert_eq!(date.day().unwrap(), 2);
    /// assert_eq!(date.calendar().identifier(), "iso8601");
    ///
    /// ```
    #[inline]
    pub fn from_partial(
        partial: PartialDate,
        calendar: Option<Calendar>,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        let year_check =
            partial.year.is_some() || (partial.era.is_some() && partial.era_year.is_some());
        let month_check = partial.month.is_some() || partial.month_code.is_some();
        if !year_check || !month_check || partial.day.is_none() {
            return Err(TemporalError::range().with_message("Invalid PlainDate fields provided."));
        }
        let calendar = calendar.unwrap_or_default();
        let overflow = overflow.unwrap_or_default();
        calendar.date_from_partial(&partial, overflow)
    }

    /// Creates a date time with values from a `PartialDate`.
    pub fn with(
        &self,
        partial: PartialDate,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        if partial.is_empty() {
            return Err(TemporalError::r#type().with_message("A PartialDate must have a field."));
        }
        // 6. Let fieldsResult be ? PrepareCalendarFieldsAndFieldNames(calendarRec, temporalDate, « "day", "month", "monthCode", "year" »).
        // 7. Let partialDate be ? PrepareTemporalFields(temporalDateLike, fieldsResult.[[FieldNames]], partial).
        // 8. Let fields be ? CalendarMergeFields(calendarRec, fieldsResult.[[Fields]], partialDate).
        // 9. Set fields to ? PrepareTemporalFields(fields, fieldsResult.[[FieldNames]], «»).
        // 10. Return ? CalendarDateFromFields(calendarRec, fields, resolvedOptions).
        self.calendar.date_from_partial(
            &partial.with_fallback_date(self)?,
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
        )
    }

    /// Creates a new `Date` from the current `Date` and the provided calendar.
    pub fn with_calendar(&self, calendar: Calendar) -> TemporalResult<Self> {
        Self::try_new(
            self.iso_year(),
            self.iso_month().into(),
            self.iso_day().into(),
            calendar,
        )
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO year value.
    pub const fn iso_year(&self) -> i32 {
        self.iso.year
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO month value.
    pub const fn iso_month(&self) -> u8 {
        self.iso.month
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO day value.
    pub const fn iso_day(&self) -> u8 {
        self.iso.day
    }

    #[inline]
    #[must_use]
    /// Returns a reference to this `Date`'s calendar slot.
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// 3.5.7 `IsValidISODate`
    ///
    /// Checks if the current date is a valid `ISODate`.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.iso.is_valid()
    }

    /// `DaysUntil`
    ///
    /// Calculates the epoch days between two `Date`s
    #[inline]
    #[must_use]
    pub fn days_until(&self, other: &Self) -> i32 {
        other.iso.to_epoch_days() - self.iso.to_epoch_days()
    }

    #[inline]
    /// Adds a `Duration` to the current `Date`
    pub fn add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.add_date(duration, overflow)
    }

    #[inline]
    /// Subtracts a `Duration` to the current `Date`
    pub fn subtract(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.add_date(&duration.negated(), overflow)
    }

    #[inline]
    /// Returns a `Duration` representing the time from this `Date` until the other `Date`.
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff_date(DifferenceOperation::Until, other, settings)
    }

    #[inline]
    /// Returns a `Duration` representing the time passed from this `Date` since the other `Date`.
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff_date(DifferenceOperation::Since, other, settings)
    }
}

// ==== Calendar-derived Public API ====

impl PlainDate {
    /// Returns the calendar year value.
    pub fn year(&self) -> TemporalResult<i32> {
        self.calendar.year(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar month value.
    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar.month(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar month code value.
    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.calendar.month_code(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar day value.
    pub fn day(&self) -> TemporalResult<u8> {
        self.calendar.day(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        self.calendar.day_of_week(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        self.calendar.day_of_year(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<Option<u16>> {
        self.calendar.week_of_year(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<Option<i32>> {
        self.calendar.year_of_week(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        self.calendar.days_in_week(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        self.calendar.days_in_month(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        self.calendar.days_in_year(&CalendarDateLike::Date(self))
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        self.calendar.months_in_year(&CalendarDateLike::Date(self))
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        self.calendar.in_leap_year(&CalendarDateLike::Date(self))
    }

    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        self.calendar.era(&CalendarDateLike::Date(self))
    }

    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        self.calendar.era_year(&CalendarDateLike::Date(self))
    }
}

// ==== ToX Methods ====

impl PlainDate {
    /// Converts the current `Date` into a `DateTime`
    ///
    /// # Notes
    ///
    /// If no time is provided, then the time will default to midnight.
    #[inline]
    pub fn to_date_time(&self, time: Option<PlainTime>) -> TemporalResult<PlainDateTime> {
        let time = time.unwrap_or_default();
        let iso = IsoDateTime::new(self.iso_date(), time.iso)?;
        Ok(PlainDateTime::new_unchecked(iso, self.get_calendar()))
    }

    /// Converts the current `Date<C>` into a `PlainYearMonth`
    #[inline]
    pub fn to_year_month(&self) -> TemporalResult<PlainYearMonth> {
        self.get_calendar().year_month_from_partial(
            &PartialDate::default().with_fallback_date(self)?,
            ArithmeticOverflow::Constrain,
        )
    }

    /// Converts the current `Date<C>` into a `PlainMonthDay`
    #[inline]
    pub fn to_month_day(&self) -> TemporalResult<PlainMonthDay> {
        self.get_calendar().month_day_from_partial(
            &PartialDate::default().with_fallback_date(self)?,
            ArithmeticOverflow::Constrain,
        )
    }
}

// ==== Trait impls ====

impl GetTemporalCalendar for PlainDate {
    fn get_calendar(&self) -> Calendar {
        self.calendar.clone()
    }
}

impl IsoDateSlots for PlainDate {
    /// Returns the structs `IsoDate`
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}

impl From<PlainDateTime> for PlainDate {
    fn from(value: PlainDateTime) -> Self {
        PlainDate::new_unchecked(value.iso.date, value.calendar().clone())
    }
}

// TODO: impl From<ZonedDateTime> for PlainDate

impl FromStr for PlainDate {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parse_record = parse_date_time(s)?;

        let calendar = parse_record.calendar.unwrap_or("iso8601");

        // Assertion: PlainDate must exist on a DateTime parse.
        let date = parse_record.date.temporal_unwrap()?;
        let calendar = Calendar::from_str(calendar)?;

        Self::try_new(date.year, date.month.into(), date.day.into(), calendar)
    }
}

#[cfg(test)]
mod tests {
    use tinystr::tinystr;

    use super::*;

    #[test]
    fn simple_date_add() {
        let base = PlainDate::from_str("1976-11-18").unwrap();

        // Test 1
        let result = base
            .add(&Duration::from_str("P43Y").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 2019,
                month: 11,
                day: 18,
            }
        );

        // Test 2
        let result = base.add(&Duration::from_str("P3M").unwrap(), None).unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 1977,
                month: 2,
                day: 18,
            }
        );

        // Test 3
        let result = base
            .add(&Duration::from_str("P20D").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 1976,
                month: 12,
                day: 8,
            }
        )
    }

    #[test]
    fn simple_date_subtract() {
        let base = PlainDate::from_str("2019-11-18").unwrap();

        // Test 1
        let result = base
            .subtract(&Duration::from_str("P43Y").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 1976,
                month: 11,
                day: 18,
            }
        );

        // Test 2
        let result = base
            .subtract(&Duration::from_str("P11M").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 2018,
                month: 12,
                day: 18,
            }
        );

        // Test 3
        let result = base
            .subtract(&Duration::from_str("P20D").unwrap(), None)
            .unwrap();
        assert_eq!(
            result.iso,
            IsoDate {
                year: 2019,
                month: 10,
                day: 29,
            }
        )
    }

    #[test]
    fn simple_date_until() {
        let earlier = PlainDate::from_str("1969-07-24").unwrap();
        let later = PlainDate::from_str("1969-10-05").unwrap();
        let result = earlier
            .until(&later, DifferenceSettings::default())
            .unwrap();
        assert_eq!(result.days(), 73.0,);

        let later = PlainDate::from_str("1996-03-03").unwrap();
        let result = earlier
            .until(&later, DifferenceSettings::default())
            .unwrap();
        assert_eq!(result.days(), 9719.0,);
    }

    #[test]
    fn simple_date_since() {
        let earlier = PlainDate::from_str("1969-07-24").unwrap();
        let later = PlainDate::from_str("1969-10-05").unwrap();
        let result = later
            .since(&earlier, DifferenceSettings::default())
            .unwrap();
        assert_eq!(result.days(), 73.0,);

        let later = PlainDate::from_str("1996-03-03").unwrap();
        let result = later
            .since(&earlier, DifferenceSettings::default())
            .unwrap();
        assert_eq!(result.days(), 9719.0,);
    }

    #[test]
    fn date_with_empty_error() {
        let base = PlainDate::new(1976, 11, 18, Calendar::default()).unwrap();

        let err = base.with(PartialDate::default(), None);
        assert!(err.is_err());
    }

    #[test]
    fn basic_date_with() {
        let base = PlainDate::new(1976, 11, 18, Calendar::default()).unwrap();

        // Year
        let partial = PartialDate {
            year: Some(2019),
            ..Default::default()
        };
        let with_year = base.with(partial, None).unwrap();
        assert_eq!(with_year.year().unwrap(), 2019);
        assert_eq!(with_year.month().unwrap(), 11);
        assert_eq!(
            with_year.month_code().unwrap(),
            TinyAsciiStr::<4>::from_str("M11").unwrap()
        );
        assert_eq!(with_year.day().unwrap(), 18);

        // Month
        let partial = PartialDate {
            month: Some(5),
            ..Default::default()
        };
        let with_month = base.with(partial, None).unwrap();
        assert_eq!(with_month.year().unwrap(), 1976);
        assert_eq!(with_month.month().unwrap(), 5);
        assert_eq!(
            with_month.month_code().unwrap(),
            TinyAsciiStr::<4>::from_str("M05").unwrap()
        );
        assert_eq!(with_month.day().unwrap(), 18);

        // Month Code
        let partial = PartialDate {
            month_code: Some(tinystr!(4, "M05")),
            ..Default::default()
        };
        let with_mc = base.with(partial, None).unwrap();
        assert_eq!(with_mc.year().unwrap(), 1976);
        assert_eq!(with_mc.month().unwrap(), 5);
        assert_eq!(
            with_mc.month_code().unwrap(),
            TinyAsciiStr::<4>::from_str("M05").unwrap()
        );
        assert_eq!(with_mc.day().unwrap(), 18);

        // Day
        let partial = PartialDate {
            day: Some(17),
            ..Default::default()
        };
        let with_day = base.with(partial, None).unwrap();
        assert_eq!(with_day.year().unwrap(), 1976);
        assert_eq!(with_day.month().unwrap(), 11);
        assert_eq!(
            with_day.month_code().unwrap(),
            TinyAsciiStr::<4>::from_str("M11").unwrap()
        );
        assert_eq!(with_day.day().unwrap(), 17);
    }

    // test262/test/built-ins/Temporal/Calendar/prototype/month/argument-string-invalid.js
    #[test]
    fn invalid_strings() {
        const INVALID_STRINGS: [&str; 35] = [
            // invalid ISO strings:
            "",
            "invalid iso8601",
            "2020-01-00",
            "2020-01-32",
            "2020-02-30",
            "2021-02-29",
            "2020-00-01",
            "2020-13-01",
            "2020-01-01T",
            "2020-01-01T25:00:00",
            "2020-01-01T01:60:00",
            "2020-01-01T01:60:61",
            "2020-01-01junk",
            "2020-01-01T00:00:00junk",
            "2020-01-01T00:00:00+00:00junk",
            "2020-01-01T00:00:00+00:00[UTC]junk",
            "2020-01-01T00:00:00+00:00[UTC][u-ca=iso8601]junk",
            "02020-01-01",
            "2020-001-01",
            "2020-01-001",
            "2020-01-01T001",
            "2020-01-01T01:001",
            "2020-01-01T01:01:001",
            // valid, but forms not supported in Temporal:
            "2020-W01-1",
            "2020-001",
            "+0002020-01-01",
            // valid, but this calendar must not exist:
            "2020-01-01[u-ca=notexist]",
            // may be valid in other contexts, but insufficient information for PlainDate:
            "2020-01",
            "+002020-01",
            "01-01",
            "2020-W01",
            "P1Y",
            "-P12Y",
            // valid, but outside the supported range:
            "-999999-01-01",
            "+999999-01-01",
        ];
        for s in INVALID_STRINGS {
            assert!(PlainDate::from_str(s).is_err())
        }
    }

    // test262/test/built-ins/Temporal/Calendar/prototype/day/argument-string-critical-unknown-annotation.js
    #[test]
    fn argument_string_critical_unknown_annotation() {
        const INVALID_STRINGS: [&str; 6] = [
            "1970-01-01[!foo=bar]",
            "1970-01-01T00:00[!foo=bar]",
            "1970-01-01T00:00[UTC][!foo=bar]",
            "1970-01-01T00:00[u-ca=iso8601][!foo=bar]",
            "1970-01-01T00:00[UTC][!foo=bar][u-ca=iso8601]",
            "1970-01-01T00:00[foo=bar][!_foo-bar0=Dont-Ignore-This-99999999999]",
        ];
        for s in INVALID_STRINGS {
            assert!(PlainDate::from_str(s).is_err())
        }
    }
}
