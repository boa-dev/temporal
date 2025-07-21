//! This module implements `YearMonth` and any directly related algorithms.

use alloc::{format, string::String};
use core::{cmp::Ordering, str::FromStr};

use tinystr::TinyAsciiStr;

use crate::{
    iso::{year_month_within_limits, IsoDate, IsoDateTime, IsoTime},
    options::{
        ArithmeticOverflow, DifferenceOperation, DifferenceSettings, Disambiguation,
        DisplayCalendar, ResolvedRoundingOptions, RoundingIncrement, Unit, UnitGroup,
    },
    parsers::{FormattableCalendar, FormattableDate, FormattableYearMonth},
    provider::{NeverProvider, TimeZoneProvider},
    temporal_assert,
    unix_time::EpochNanoseconds,
    utils::pad_iso_year,
    Calendar, MonthCode, TemporalError, TemporalResult, TemporalUnwrap, TimeZone,
};
use icu_calendar::AnyCalendarKind;

use super::{
    calendar::month_to_month_code, duration::normalized::NormalizedDurationRecord, DateDuration,
    Duration, PartialDate, PlainDate, PlainDateTime,
};
use writeable::Writeable;

/// A partial PlainYearMonth record
#[derive(Debug, Default, Clone, PartialEq)]
pub struct PartialYearMonth {
    /// A potentially set `year` field.
    pub year: Option<i32>,
    /// A potentially set `month` field.
    pub month: Option<u8>,
    /// A potentially set `month_code` field.
    pub month_code: Option<MonthCode>,
    /// A potentially set `era` field.
    pub era: Option<TinyAsciiStr<19>>,
    /// A potentially set `era_year` field.
    pub era_year: Option<i32>,
    /// The calendar field
    pub calendar: Calendar,
}

impl PartialYearMonth {
    pub(crate) fn try_from_year_month(year_month: &PlainYearMonth) -> TemporalResult<Self> {
        let (year, era, era_year) = if year_month.era().is_some() {
            (
                None,
                year_month
                    .era()
                    .map(|t| TinyAsciiStr::<19>::try_from_utf8(t.as_bytes()))
                    .transpose()
                    .map_err(|e| TemporalError::general(format!("{e}")))?,
                year_month.era_year(),
            )
        } else {
            (Some(year_month.year()), None, None)
        };
        Ok(Self {
            year,
            month: Some(year_month.month()),
            month_code: Some(year_month.month_code()),
            era,
            era_year,
            calendar: year_month.calendar().clone(),
        })
    }

    crate::impl_with_fallback_method!(with_fallback_year_month, () PlainYearMonth);
}

impl From<&PartialYearMonth> for PartialDate {
    fn from(value: &PartialYearMonth) -> Self {
        Self {
            year: value.year,
            month: value.month,
            month_code: value.month_code,
            day: None,
            era: value.era,
            era_year: value.era_year,
            calendar: value.calendar.clone(),
        }
    }
}

impl From<&PartialDate> for PartialYearMonth {
    fn from(value: &PartialDate) -> Self {
        Self {
            year: value.year,
            month: value.month,
            month_code: value.month_code,
            era: value.era,
            era_year: value.era_year,
            calendar: value.calendar.clone(),
        }
    }
}

/// Convenience methods for building a `PartialYearMonth`
impl PartialYearMonth {
    pub const fn new() -> Self {
        Self {
            year: None,
            month: None,
            month_code: None,
            era: None,
            era_year: None,
            calendar: Calendar::new(AnyCalendarKind::Iso),
        }
    }

    pub const fn with_era(mut self, era: Option<TinyAsciiStr<19>>) -> Self {
        self.era = era;
        self
    }

    pub const fn with_era_year(mut self, era_year: Option<i32>) -> Self {
        self.era_year = era_year;
        self
    }

    pub const fn with_year(mut self, year: Option<i32>) -> Self {
        self.year = year;
        self
    }

    pub const fn with_month(mut self, month: Option<u8>) -> Self {
        self.month = month;
        self
    }

    pub const fn with_month_code(mut self, month_code: Option<MonthCode>) -> Self {
        self.month_code = month_code;
        self
    }

    pub const fn with_calendar(mut self, calendar: Calendar) -> Self {
        self.calendar = calendar;
        self
    }
}

/// The native Rust implementation of `Temporal.PlainYearMonth`.
///
/// Represents a specific month within a specific year, such as "January 2024"
/// or "December 2023", without a specific day component.
///
/// Useful for representing time periods at month granularity, such as billing
/// periods, reporting intervals, or any scenario where you need to work with
/// entire months rather than specific dates.
///
/// ## Examples
///
/// ### Creating a PlainYearMonth
///
/// ```rust
/// use temporal_rs::{PlainYearMonth, Calendar};
///
/// // Create with ISO 8601 calendar
/// let ym = PlainYearMonth::try_new_iso(2024, 3, None).unwrap();
/// assert_eq!(ym.year(), 2024);
/// assert_eq!(ym.month(), 3);
/// assert_eq!(ym.calendar().identifier(), "iso8601");
///
/// // Create with explicit calendar and reference day
/// let ym = PlainYearMonth::try_new(2024, 3, Some(15), Calendar::default()).unwrap();
/// assert_eq!(ym.year(), 2024);
/// assert_eq!(ym.month(), 3);
/// // Reference day helps with calendar calculations but doesn't affect the YearMonth itself
/// ```
///
/// ### Parsing ISO 8601 year-month strings
///
/// ```rust
/// use temporal_rs::PlainYearMonth;
/// use core::str::FromStr;
///
/// // Parse year-month strings
/// let ym = PlainYearMonth::from_str("2024-03").unwrap();
/// assert_eq!(ym.year(), 2024);
/// assert_eq!(ym.month(), 3);
///
/// // Also accepts full date strings (day is ignored for YearMonth semantics)
/// let ym2 = PlainYearMonth::from_str("2024-03-15").unwrap();
/// assert_eq!(ym2.year(), 2024);
/// assert_eq!(ym2.month(), 3);
/// assert_eq!(ym, ym2); // equivalent
/// ```
///
/// ### YearMonth arithmetic
///
/// ```rust
/// use temporal_rs::{PlainYearMonth, options::DifferenceSettings};
/// use core::str::FromStr;
///
/// let ym1 = PlainYearMonth::from_str("2024-01").unwrap();
/// let ym2 = PlainYearMonth::from_str("2024-04").unwrap();
///
/// // Calculate difference between year-months  
/// let duration = ym1.until(&ym2, DifferenceSettings::default()).unwrap();
/// assert_eq!(duration.months(), 3); // January to April = 3 months
/// ```
///
/// ### Working with partial fields
///
/// ```rust
/// use temporal_rs::{PlainYearMonth, partial::PartialYearMonth};
/// use core::str::FromStr;
///
/// let ym = PlainYearMonth::from_str("2024-01").unwrap();
///
/// // Change only the year
/// let partial = PartialYearMonth::new().with_year(Some(2025));
/// let modified = ym.with(partial, None).unwrap();
/// assert_eq!(modified.year(), 2025);
/// assert_eq!(modified.month(), 1); // unchanged
///
/// // Change only the month
/// let partial = PartialYearMonth::new().with_month(Some(6));
/// let modified = ym.with(partial, None).unwrap();
/// assert_eq!(modified.year(), 2024); // unchanged
/// assert_eq!(modified.month(), 6);
/// ```
///
/// ### Converting to PlainDate
///
/// ```rust
/// use temporal_rs::{PlainYearMonth, partial::PartialDate};
/// use core::str::FromStr;
///
/// let ym = PlainYearMonth::from_str("2024-03").unwrap();
///
/// // Convert to a specific date by providing a day
/// let day_partial = PartialDate::new().with_day(Some(15));
/// let date = ym.to_plain_date(Some(day_partial)).unwrap();
/// assert_eq!(date.year(), 2024);
/// assert_eq!(date.month(), 3);
/// assert_eq!(date.day(), 15);
/// ```
///
/// ### Calendar properties
///
/// ```rust
/// use temporal_rs::PlainYearMonth;
/// use core::str::FromStr;
///
/// let ym = PlainYearMonth::from_str("2024-02").unwrap(); // February 2024
///
/// // Get calendar-specific properties
/// assert_eq!(ym.days_in_month(), 29); // 2024 is a leap year
/// assert_eq!(ym.days_in_year(), 366); // leap year has 366 days
/// assert_eq!(ym.months_in_year(), 12); // ISO calendar has 12 months
/// assert!(ym.in_leap_year()); // 2024 is indeed a leap year
/// ```
///
/// ## Reference
///
/// For more information, see the [MDN documentation][mdn-plainyearmonth].
///
/// [mdn-plainyearmonth]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Temporal/PlainYearMonth
#[non_exhaustive]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct PlainYearMonth {
    pub(crate) iso: IsoDate,
    calendar: Calendar,
}

impl core::fmt::Display for PlainYearMonth {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.to_ixdtf_string(DisplayCalendar::Auto))
    }
}

impl PlainYearMonth {
    /// Creates an unvalidated `YearMonth`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    /// [`9.5.8 AddDurationToYearMonth(operation, yearMonth, temporalDurationLike, options)`][spec]
    ///
    /// Internal addition method for adding `Duration` to a `PlainYearMonth`
    ///
    /// [spec]: <https://tc39.es/proposal-temporal/#sec-temporal-adddurationtoyearmonth>
    ///
    // spec(2025-06-23): https://github.com/tc39/proposal-temporal/tree/ed49b0b482981119c9b5e28b0686d877d4a9bae0
    pub(crate) fn add_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        // NOTE: The following are engine specific:
        //    SKIP: 1. Let duration be ? ToTemporalDuration(temporalDurationLike).

        // NOTE: The following operation has been moved to the caller.
        //    MOVE: 2. If operation is subtract, set duration to CreateNegatedTemporalDuration(duration).

        // NOTE: The following are engine specific:
        //    SKIP: 3. Let resolvedOptions be ? GetOptionsObject(options).
        //    SKIP: 4. Let overflow be ? GetTemporalOverflowOption(resolvedOptions).

        // 5. Let sign be DurationSign(duration).
        let sign = duration.sign();

        // 6. Let calendar be yearMonth.[[Calendar]].
        let calendar = self.calendar();

        // 7. Let fields be ISODateToFields(calendar, yearMonth.[[ISODate]], year-month).
        let fields = PartialDate::from(&PartialYearMonth::try_from_year_month(self)?);

        // 8. Set fields.[[Day]] to 1.
        let fields = fields.with_day(Some(1));

        // 9. Let intermediateDate be ? CalendarDateFromFields(calendar, fields, constrain).
        let intermediate_date = calendar.date_from_partial(&fields, overflow)?;

        // 10. If sign < 0, then
        let date = if sign.as_sign_multiplier() < 0 {
            // a. Let oneMonthDuration be ! CreateDateDurationRecord(0, 1, 0, 0).
            let one_month_duration = DateDuration::new_unchecked(0, 1, 0, 0);

            // b. Let nextMonth be ? CalendarDateAdd(calendar, intermediateDate, oneMonthDuration, constrain).
            let next_month = calendar.date_add(
                &intermediate_date.iso,
                &Duration::from(one_month_duration),
                ArithmeticOverflow::Constrain,
            )?;

            // c. Let date be BalanceISODate(nextMonth.[[Year]], nextMonth.[[Month]], nextMonth.[[Day]] - 1).
            let date = IsoDate::balance(
                next_month.year(),
                i32::from(next_month.month()),
                i32::from(next_month.day())
                    .checked_sub(1)
                    .temporal_unwrap()?,
            );

            // d. Assert: ISODateWithinLimits(date) is true.
            temporal_assert!(date.is_valid());

            date
        } else {
            // 11. Else,
            //    a. Let date be intermediateDate.
            intermediate_date.iso
        };

        // 12. Let durationToAdd be ToDateDurationRecordWithoutTime(duration).
        let duration_to_add = duration.to_date_duration_record_without_time()?;

        // 13. Let addedDate be ? CalendarDateAdd(calendar, date, durationToAdd, overflow).
        let added_date = calendar.date_add(&date, &Duration::from(duration_to_add), overflow)?;

        // 14. Let addedDateFields be ISODateToFields(calendar, addedDate, year-month).
        let added_date_fields = PartialYearMonth::from(
            &PartialDate::default().with_fallback_date(&added_date, overflow)?,
        );

        // 15. Let isoDate be ? CalendarYearMonthFromFields(calendar, addedDateFields, overflow).
        let iso_date = calendar.year_month_from_partial(&added_date_fields, overflow)?;

        // 16. Return ! CreateTemporalYearMonth(isoDate, calendar).
        Ok(iso_date)
    }

    /// The internal difference operation of `PlainYearMonth`.
    pub(crate) fn diff(
        &self,
        op: DifferenceOperation,
        other: &Self,
        settings: DifferenceSettings,
    ) -> TemporalResult<Duration> {
        // 1. Set other to ? ToTemporalYearMonth(other).
        // 2. Let calendar be yearMonth.[[Calendar]].
        // 3. If CalendarEquals(calendar, other.[[Calendar]]) is false, throw a RangeError exception.
        if self.calendar().identifier() != other.calendar().identifier() {
            return Err(TemporalError::range()
                .with_message("Calendars for difference operation are not the same."));
        }

        // Check if weeks or days are disallowed in this operation
        if matches!(settings.largest_unit, Some(Unit::Week) | Some(Unit::Day))
            || matches!(settings.smallest_unit, Some(Unit::Week) | Some(Unit::Day))
        {
            return Err(TemporalError::range()
                .with_message("Weeks and days are not allowed in this operation."));
        }

        // 4. Let resolvedOptions be ? GetOptionsObject(options).
        // 5. Let settings be ? GetDifferenceSettings(operation, resolvedOptions, date, « week, day », month, year).
        let resolved = ResolvedRoundingOptions::from_diff_settings(
            settings,
            op,
            UnitGroup::Date,
            Unit::Year,
            Unit::Month,
        )?;

        // 6. If CompareISODate(yearMonth.[[ISODate]], other.[[ISODate]]) = 0, then
        if self.iso == other.iso {
            // a. Return ! CreateTemporalDuration(0, 0, 0, 0, 0, 0, 0, 0, 0, 0).
            return Ok(Duration::default());
        }

        // 7. Let thisFields be ISODateToFields(calendar, yearMonth.[[ISODate]], year-month).
        // 8. Set thisFields.[[Day]] to 1.
        // 9. Let thisDate be ? CalendarDateFromFields(calendar, thisFields, constrain).
        // 10. Let otherFields be ISODateToFields(calendar, other.[[ISODate]], year-month).
        // 11. Set otherFields.[[Day]] to 1.
        // 12. Let otherDate be ? CalendarDateFromFields(calendar, otherFields, constrain).
        // 13. Let dateDifference be CalendarDateUntil(calendar, thisDate, otherDate, settings.[[LargestUnit]]).
        // 14. Let yearsMonthsDifference be ! AdjustDateDurationRecord(dateDifference, 0, 0).
        let result = self
            .calendar()
            .date_until(&self.iso, &other.iso, resolved.largest_unit)?;

        // 15. Let duration be CombineDateAndTimeDuration(yearsMonthsDifference, 0).
        let mut duration = NormalizedDurationRecord::from_date_duration(*result.date())?;

        // 16. If settings.[[SmallestUnit]] is not month or settings.[[RoundingIncrement]] ≠ 1, then
        if resolved.smallest_unit != Unit::Month || resolved.increment != RoundingIncrement::ONE {
            // a. Let isoDateTime be CombineISODateAndTimeRecord(thisDate, MidnightTimeRecord()).
            let iso_date_time = IsoDateTime::new_unchecked(self.iso, IsoTime::default());
            // b. Let isoDateTimeOther be CombineISODateAndTimeRecord(otherDate, MidnightTimeRecord()).
            let target_iso_date_time = IsoDateTime::new_unchecked(other.iso, IsoTime::default());
            // c. Let destEpochNs be GetUTCEpochNanoseconds(isoDateTimeOther).
            let dest_epoch_ns = target_iso_date_time.as_nanoseconds()?;
            // d. Set duration to ? RoundRelativeDuration(duration, destEpochNs, isoDateTime, unset, calendar, resolved.[[LargestUnit]], resolved.[[RoundingIncrement]], resolved.[[SmallestUnit]], resolved.[[RoundingMode]]).
            duration = duration.round_relative_duration(
                dest_epoch_ns.as_i128(),
                &PlainDateTime::new_unchecked(iso_date_time, self.calendar.clone()),
                Option::<(&TimeZone, &NeverProvider)>::None,
                resolved,
            )?;
        }

        // 17. Let result be ! TemporalDurationFromInternal(duration, day).
        let result = Duration::from_normalized(duration, Unit::Day)?;

        // 18. If operation is since, set result to CreateNegatedTemporalDuration(result).
        // 19. Return result.
        match op {
            DifferenceOperation::Since => Ok(result.negated()),
            DifferenceOperation::Until => Ok(result),
        }
    }
}

// ==== Public method implementations ====

impl PlainYearMonth {
    /// Creates a new `PlainYearMonth`, constraining any arguments that are invalid into a valid range.
    #[inline]
    pub fn new(
        year: i32,
        month: u8,
        reference_day: Option<u8>,
        calendar: Calendar,
    ) -> TemporalResult<Self> {
        Self::new_with_overflow(
            year,
            month,
            reference_day,
            calendar,
            ArithmeticOverflow::Constrain,
        )
    }

    /// Creates a new `PlainYearMonth`, rejecting any date that may be invalid.
    #[inline]
    pub fn try_new(
        year: i32,
        month: u8,
        reference_day: Option<u8>,
        calendar: Calendar,
    ) -> TemporalResult<Self> {
        Self::new_with_overflow(
            year,
            month,
            reference_day,
            calendar,
            ArithmeticOverflow::Reject,
        )
    }

    /// Creates a new `PlainYearMonth` with an ISO 8601 calendar, rejecting any date that may be invalid.
    #[inline]
    pub fn try_new_iso(year: i32, month: u8, reference_day: Option<u8>) -> TemporalResult<Self> {
        Self::try_new(year, month, reference_day, Calendar::default())
    }

    /// Creates a new `PlainYearMonth` with an ISO 8601 calendar, constraining any arguments
    /// that are invalid into a valid range.
    #[inline]
    pub fn new_iso(year: i32, month: u8, reference_day: Option<u8>) -> TemporalResult<Self> {
        Self::new(year, month, reference_day, Calendar::default())
    }

    /// Creates a new valid `YearMonth` with provided `ArithmeticOverflow` option.
    #[inline]
    pub fn new_with_overflow(
        year: i32,
        month: u8,
        reference_day: Option<u8>,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let day = reference_day.unwrap_or(1);
        let iso = IsoDate::regulate(year, month, day, overflow)?;
        if !year_month_within_limits(iso.year, iso.month) {
            return Err(TemporalError::range().with_message("Exceeded valid range."));
        }
        Ok(Self::new_unchecked(iso, calendar))
    }

    /// Create a `PlainYearMonth` from a `PartialYearMonth`
    pub fn from_partial(
        partial: PartialYearMonth,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        partial
            .calendar
            .year_month_from_partial(&partial, overflow.unwrap_or_default())
    }

    // Converts a UTF-8 encoded string into a `PlainYearMonth`.
    pub fn from_utf8(s: &[u8]) -> TemporalResult<Self> {
        let record = crate::parsers::parse_year_month(s)?;
        let calendar = record
            .calendar
            .map(Calendar::try_from_utf8)
            .transpose()?
            .unwrap_or_default();

        // ParseISODateTime
        // Step 4.a.ii.3
        // If goal is TemporalMonthDayString or TemporalYearMonthString, calendar is
        // not empty, and the ASCII-lowercase of calendar is not "iso8601", throw a
        // RangeError exception.
        if !calendar.is_iso() {
            return Err(TemporalError::range().with_message("non-ISO calendar not supported."));
        }

        let date = record.date.temporal_unwrap()?;

        // The below steps are from `ToTemporalYearMonth`
        // 10. Let isoDate be CreateISODateRecord(result.[[Year]], result.[[Month]], result.[[Day]]).
        let iso = IsoDate::new_unchecked(date.year, date.month, date.day);

        // 11. If ISOYearMonthWithinLimits(isoDate) is false, throw a RangeError exception.
        if !year_month_within_limits(iso.year, iso.month) {
            return Err(TemporalError::range().with_message("Exceeded valid range."));
        }

        let intermediate = Self::new_unchecked(iso, calendar);
        // 12. Set result to ISODateToFields(calendar, isoDate, year-month).
        let partial = PartialYearMonth::try_from_year_month(&intermediate)?;
        // 13. NOTE: The following operation is called with constrain regardless of the
        // value of overflow, in order for the calendar to store a canonical value in the
        // [[Day]] field of the [[ISODate]] internal slot of the result.
        // 14. Set isoDate to ? CalendarYearMonthFromFields(calendar, result, constrain).
        // 15. Return ! CreateTemporalYearMonth(isoDate, calendar).
        PlainYearMonth::from_partial(partial, Some(ArithmeticOverflow::Constrain))
    }

    /// Returns the iso year value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn iso_year(&self) -> i32 {
        self.iso.year
    }

    /// Returns the padded ISO year string
    #[inline]
    #[must_use]
    pub fn padded_iso_year_string(&self) -> String {
        pad_iso_year(self.iso.year)
    }

    /// Returns the iso month value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn iso_month(&self) -> u8 {
        self.iso.month
    }

    /// Returns the calendar era of the current `PlainYearMonth`
    pub fn era(&self) -> Option<TinyAsciiStr<16>> {
        self.calendar().era(&self.iso)
    }

    /// Returns the calendar era year of the current `PlainYearMonth`
    pub fn era_year(&self) -> Option<i32> {
        self.calendar().era_year(&self.iso)
    }

    /// Returns the calendar year of the current `PlainYearMonth`
    pub fn year(&self) -> i32 {
        self.calendar().year(&self.iso)
    }

    /// Returns the calendar month of the current `PlainYearMonth`
    pub fn month(&self) -> u8 {
        self.calendar().month(&self.iso)
    }

    /// Returns the calendar month code of the current `PlainYearMonth`
    pub fn month_code(&self) -> MonthCode {
        self.calendar().month_code(&self.iso)
    }

    /// Returns the days in the calendar year of the current `PlainYearMonth`.
    pub fn days_in_year(&self) -> u16 {
        self.calendar().days_in_year(&self.iso)
    }

    /// Returns the days in the calendar month of the current `PlainYearMonth`.
    pub fn days_in_month(&self) -> u16 {
        self.calendar().days_in_month(&self.iso)
    }

    /// Returns the months in the calendar year of the current `PlainYearMonth`.
    pub fn months_in_year(&self) -> u16 {
        self.calendar().months_in_year(&self.iso)
    }

    #[inline]
    #[must_use]
    /// Returns a boolean representing whether the current `PlainYearMonth` is in a leap year.
    pub fn in_leap_year(&self) -> bool {
        self.calendar().in_leap_year(&self.iso)
    }
}

impl PlainYearMonth {
    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    /// Returns the string identifier for the current calendar used.
    #[inline]
    #[must_use]
    pub fn calendar_id(&self) -> &'static str {
        self.calendar.identifier()
    }

    /// Creates a `PlainYearMonth` using the fields provided from a [`PartialYearMonth`]
    pub fn with(
        &self,
        partial: PartialYearMonth,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        // 1. Let yearMonth be the this value.
        // 2. Perform ? RequireInternalSlot(yearMonth, [[InitializedTemporalYearMonth]]).
        // 3. If ? IsPartialTemporalObject(temporalYearMonthLike) is false, throw a TypeError exception.
        // 4. Let calendar be yearMonth.[[Calendar]].
        // 5. Let fields be ISODateToFields(calendar, yearMonth.[[ISODate]], year-month).
        // 6. Let partialYearMonth be ? PrepareCalendarFields(calendar, temporalYearMonthLike, « year, month, month-code », « », partial).
        // 7. Set fields to CalendarMergeFields(calendar, fields, partialYearMonth).
        // 8. Let resolvedOptions be ? GetOptionsObject(options).
        // 9. Let overflow be ? GetTemporalOverflowOption(resolvedOptions).
        // 10. Let isoDate be ? CalendarYearMonthFromFields(calendar, fields, overflow).
        // 11. Return ! CreateTemporalYearMonth(isoDate, calendar).
        let overflow = overflow.unwrap_or(ArithmeticOverflow::Constrain);
        self.calendar
            .year_month_from_partial(&partial.with_fallback_year_month(self, overflow)?, overflow)
    }

    /// Compares one `PlainYearMonth` to another `PlainYearMonth` using their
    /// `IsoDate` representation.
    ///
    /// # Note on Ordering.
    ///
    /// `temporal_rs` does not implement `PartialOrd`/`Ord` as `PlainYearMonth` does
    /// not fulfill all the conditions required to implement the traits. However,
    /// it is possible to compare `PlainDate`'s as their `IsoDate` representation.
    #[inline]
    #[must_use]
    pub fn compare_iso(&self, other: &Self) -> Ordering {
        self.iso.cmp(&other.iso)
    }

    /// Adds a [`Duration`] from the current `PlainYearMonth`.
    #[inline]
    pub fn add(&self, duration: &Duration, overflow: ArithmeticOverflow) -> TemporalResult<Self> {
        self.add_duration(duration, overflow)
    }

    /// Subtracts a [`Duration`] from the current `PlainYearMonth`.
    #[inline]
    pub fn subtract(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        self.add_duration(&duration.negated(), overflow)
    }

    /// Returns a `Duration` representing the period of time from this `PlainYearMonth` until the other `PlainYearMonth`.
    #[inline]
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff(DifferenceOperation::Until, other, settings)
    }

    /// Returns a `Duration` representing the period of time from this `PlainYearMonth` since the other `PlainYearMonth`.
    #[inline]
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.diff(DifferenceOperation::Since, other, settings)
    }

    pub fn to_plain_date(&self, day: Option<PartialDate>) -> TemporalResult<PlainDate> {
        let day_value = match &day {
            Some(partial) => partial.day.ok_or_else(|| {
                TemporalError::r#type().with_message("PartialDate must contain a day field")
            })?,
            None => return Err(TemporalError::r#type().with_message("Day must be provided")),
        };

        let partial_date = PartialDate::new()
            .with_year(Some(self.year()))
            .with_month_code(Some(self.month_code()))
            .with_day(Some(day_value))
            .with_calendar(self.calendar.clone());

        self.calendar
            .date_from_partial(&partial_date, ArithmeticOverflow::Reject)
    }

    /// Gets the epochMilliseconds represented by this YearMonth in the given timezone
    /// (using the reference year, and noon time)
    ///
    // Useful for implementing HandleDateTimeTemporalYearMonth
    pub fn epoch_ns_for_with_provider(
        &self,
        time_zone: &TimeZone,
        provider: &impl TimeZoneProvider,
    ) -> TemporalResult<EpochNanoseconds> {
        // 2. Let isoDateTime be CombineISODateAndTimeRecord(temporalYearMonth.[[ISODate]], NoonTimeRecord()).
        let iso = IsoDateTime::new(self.iso, IsoTime::noon())?;
        // 3. Let epochNs be ? GetEpochNanosecondsFor(dateTimeFormat.[[TimeZone]], isoDateTime, compatible).
        time_zone.get_epoch_nanoseconds_for(iso, Disambiguation::Compatible, provider)
    }

    /// Returns a RFC9557 IXDTF string for the current `PlainYearMonth`
    #[inline]
    pub fn to_ixdtf_string(&self, display_calendar: DisplayCalendar) -> String {
        self.to_ixdtf_writeable(display_calendar)
            .write_to_string()
            .into()
    }

    /// Returns a RFC9557 IXDTF string for the current `PlainYearMonth` as a Writeable
    #[inline]
    pub fn to_ixdtf_writeable(&self, display_calendar: DisplayCalendar) -> impl Writeable + '_ {
        let ixdtf = FormattableYearMonth {
            date: FormattableDate(self.iso_year(), self.iso_month(), self.iso.day),
            calendar: FormattableCalendar {
                show: display_calendar,
                calendar: self.calendar().identifier(),
            },
        };
        ixdtf
    }
}

impl FromStr for PlainYearMonth {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_utf8(s.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use core::str::FromStr;

    use super::PlainYearMonth;

    use tinystr::tinystr;

    use super::*;

    #[test]
    fn plain_year_month_since_until_diff_tests() {
        // Equal year-months
        {
            let earlier = PlainYearMonth::from_str("2024-03").unwrap();
            let later = PlainYearMonth::from_str("2024-03").unwrap();
            let settings = DifferenceSettings {
                smallest_unit: Some(Unit::Month),
                ..Default::default()
            };

            let until = earlier.until(&later, settings).unwrap();
            let since = earlier.since(&later, settings).unwrap();

            assert_eq!(until.days(), 0);
            assert_eq!(until.months(), 0);
            assert_eq!(until.years(), 0);

            assert_eq!(since.days(), 0);
            assert_eq!(since.months(), 0);
            assert_eq!(since.years(), 0);
        }

        // One month apart
        {
            let earlier = PlainYearMonth::from_str("2023-01").unwrap();
            let later = PlainYearMonth::from_str("2023-02").unwrap();
            let settings = DifferenceSettings {
                smallest_unit: Some(Unit::Month),
                ..Default::default()
            };

            let until = earlier.until(&later, settings).unwrap();
            let since = earlier.since(&later, settings).unwrap();

            assert_eq!(until.months(), 1);
            assert_eq!(until.years(), 0);

            assert_eq!(since.months(), -1);
            assert_eq!(since.years(), 0);
        }

        // Crossing year boundary
        {
            let earlier = PlainYearMonth::from_str("2022-11").unwrap();
            let later = PlainYearMonth::from_str("2023-02").unwrap();
            let settings = DifferenceSettings {
                smallest_unit: Some(Unit::Month),
                ..Default::default()
            };

            let until = earlier.until(&later, settings).unwrap();
            let since = earlier.since(&later, settings).unwrap();

            assert_eq!(until.months(), 3);
            assert_eq!(until.years(), 0);

            assert_eq!(since.months(), -3);
            assert_eq!(since.years(), 0);
        }

        // One year and one month
        {
            let earlier = PlainYearMonth::from_str("2002-05").unwrap();
            let later = PlainYearMonth::from_str("2003-06").unwrap();
            let settings = DifferenceSettings {
                smallest_unit: Some(Unit::Month),
                ..Default::default()
            };

            let until = earlier.until(&later, settings).unwrap();
            let since = earlier.since(&later, settings).unwrap();

            assert_eq!(until.years(), 1);
            assert_eq!(until.months(), 1);
            assert_eq!(until.days(), 0);

            assert_eq!(since.years(), -1);
            assert_eq!(since.months(), -1);
            assert_eq!(since.days(), 0);
        }

        // One year apart with unit = Year
        {
            let earlier = PlainYearMonth::from_str("2022-06").unwrap();
            let later = PlainYearMonth::from_str("2023-06").unwrap();
            let settings = DifferenceSettings {
                smallest_unit: Some(Unit::Year),
                ..Default::default()
            };

            let until = earlier.until(&later, settings).unwrap();
            let since = earlier.since(&later, settings).unwrap();

            assert_eq!(until.years(), 1);
            assert_eq!(until.months(), 0);

            assert_eq!(since.years(), -1);
            assert_eq!(since.months(), 0);
        }

        // Large year gap
        {
            let earlier = PlainYearMonth::from_str("1000-01").unwrap();
            let later = PlainYearMonth::from_str("2000-01").unwrap();
            let settings = DifferenceSettings {
                smallest_unit: Some(Unit::Year),
                ..Default::default()
            };

            let until = earlier.until(&later, settings).unwrap();
            let since = earlier.since(&later, settings).unwrap();

            assert_eq!(until.years(), 1000);
            assert_eq!(since.years(), -1000);
        }

        // Lower ISO limit
        {
            let earlier = PlainYearMonth::from_str("-271821-04").unwrap();
            let later = PlainYearMonth::from_str("-271820-04").unwrap();
            let settings = DifferenceSettings {
                smallest_unit: Some(Unit::Year),
                ..Default::default()
            };

            let until = earlier.until(&later, settings).unwrap();
            let since = earlier.since(&later, settings).unwrap();

            assert_eq!(until.years(), 1);
            assert_eq!(since.years(), -1);
        }
    }
    #[test]
    fn test_diff_with_different_calendars() {
        let ym1 = PlainYearMonth::new_with_overflow(
            2021,
            1,
            None,
            Calendar::from_str("islamic").unwrap(),
            ArithmeticOverflow::Reject,
        )
        .unwrap();

        let ym2 = PlainYearMonth::new_with_overflow(
            2021,
            1,
            None,
            Calendar::from_str("hebrew").unwrap(),
            ArithmeticOverflow::Reject,
        )
        .unwrap();

        let settings = DifferenceSettings {
            smallest_unit: Some(Unit::Month),
            ..Default::default()
        };

        let diff = ym1.until(&ym2, settings);
        assert!(
            diff.is_err(),
            "Expected an error when comparing dates from different calendars"
        );
    }
    #[test]
    fn test_diff_setting() {
        let ym1 = PlainYearMonth::from_str("2021-01").unwrap();
        let ym2 = PlainYearMonth::from_str("2023-02").unwrap();

        let settings = DifferenceSettings {
            smallest_unit: Some(Unit::Month),
            increment: Some(RoundingIncrement::ONE),
            ..Default::default()
        };

        let diff = ym1.until(&ym2, settings).unwrap();
        assert_eq!(diff.months(), 1);
        assert_eq!(diff.years(), 2);
    }
    #[test]
    fn test_diff_with_smallest_unit_year() {
        let ym1 = PlainYearMonth::from_str("2021-01").unwrap();
        let ym2 = PlainYearMonth::from_str("2023-02").unwrap();

        let settings = DifferenceSettings {
            smallest_unit: Some(Unit::Year),
            ..Default::default()
        };

        let diff = ym1.until(&ym2, settings).unwrap();
        assert_eq!(diff.years(), 2); // Rounded to the nearest year
        assert_eq!(diff.months(), 0); // Months are ignored
    }

    #[test]
    fn test_diff_with_smallest_unit_day() {
        let ym1 = PlainYearMonth::from_str("2021-01").unwrap();
        let ym2 = PlainYearMonth::from_str("2023-02").unwrap();

        let settings = DifferenceSettings {
            smallest_unit: Some(Unit::Day),
            ..Default::default()
        };

        let diff = ym1.until(&ym2, settings);
        assert!(
            diff.is_err(),
            "Expected an error when smallest_unit is set to Day"
        );
    }

    #[test]
    fn test_diff_with_smallest_unit_week() {
        let ym1 = PlainYearMonth::from_str("2021-01").unwrap();
        let ym2 = PlainYearMonth::from_str("2023-02").unwrap();

        let settings = DifferenceSettings {
            smallest_unit: Some(Unit::Week),
            ..Default::default()
        };

        let diff = ym1.until(&ym2, settings);
        assert!(
            diff.is_err(),
            "Expected an error when smallest_unit is set to Week"
        );
    }

    #[test]
    fn test_diff_with_no_rounding_increment() {
        let ym1 = PlainYearMonth::from_str("2021-01").unwrap();
        let ym2 = PlainYearMonth::from_str("2023-02").unwrap();

        let settings = DifferenceSettings {
            smallest_unit: Some(Unit::Month),
            increment: None, // No rounding increment
            ..Default::default()
        };

        let diff = ym1.until(&ym2, settings).unwrap();
        assert_eq!(diff.months(), 1); // Exact difference in months
        assert_eq!(diff.years(), 2); // Exact difference in years
    }

    #[test]
    fn test_plain_year_month_with() {
        let base = PlainYearMonth::new_with_overflow(
            2025,
            3,
            None,
            Calendar::default(),
            ArithmeticOverflow::Reject,
        )
        .unwrap();

        // Year
        let partial = PartialYearMonth {
            year: Some(2001),
            ..Default::default()
        };

        let with_year = base.with(partial, None).unwrap();
        assert_eq!(with_year.iso_year(), 2001); // year is changed
        assert_eq!(with_year.iso_month(), 3); // month is not changed
        assert_eq!(with_year.month_code(), MonthCode::from_str("M03").unwrap()); // assert month code has been initialized correctly

        // Month
        let partial = PartialYearMonth {
            month: Some(2),
            ..Default::default()
        };
        let with_month = base.with(partial, None).unwrap();
        assert_eq!(with_month.iso_year(), 2025); // year is not changed
        assert_eq!(with_month.iso_month(), 2); // month is changed
        assert_eq!(with_month.month_code(), MonthCode::from_str("M02").unwrap()); // assert month code has changed as well as month

        // Month Code
        let partial = PartialYearMonth {
            month_code: Some(MonthCode(tinystr!(4, "M05"))), // change month to May (5)
            ..Default::default()
        };
        let with_month_code = base.with(partial, None).unwrap();
        assert_eq!(with_month_code.iso_year(), 2025); // year is not changed
        assert_eq!(
            with_month_code.month_code(),
            MonthCode::from_str("M05").unwrap()
        ); // assert month code has changed
        assert_eq!(with_month_code.iso_month(), 5); // month is changed as well

        // Day
        let partial = PartialYearMonth::new();
        let with_day = base.with(partial, None).unwrap();
        assert_eq!(with_day.iso_year(), 2025); // year is not changed
        assert_eq!(with_day.iso_month(), 3); // month is not changed
        assert_eq!(with_day.iso.day, 1); // day is ignored

        // All
        let partial = PartialYearMonth {
            year: Some(2001),
            month: Some(2),
            ..Default::default()
        };
        let with_all = base.with(partial, None).unwrap();
        assert_eq!(with_all.iso_year(), 2001); // year is changed
        assert_eq!(with_all.iso_month(), 2); // month is changed
        assert_eq!(with_all.iso.day, 1); // day is ignored
    }

    #[test]
    fn basic_from_str() {
        let valid_strings = [
            "-271821-04",
            "-271821-04-01",
            "-271821-04-01T00:00",
            "+275760-09",
            "+275760-09-30",
            "+275760-09-30T23:59:59.999999999",
        ];

        for valid_case in valid_strings {
            let ym = PlainYearMonth::from_str(valid_case);
            assert!(ym.is_ok());
        }
    }

    #[test]
    fn invalid_from_str() {
        let invalid_strings = [
            "-271821-03-31",
            "-271821-03-31T23:59:59.999999999",
            "+275760-10",
            "+275760-10-01",
            "+275760-10-01T00:00",
            "1976-11[u-ca=hebrew]",
        ];

        for invalid_case in invalid_strings {
            let err = PlainYearMonth::from_str(invalid_case);
            assert!(err.is_err());
        }

        let invalid_strings = ["2019-10-01T09:00:00Z", "2019-10-01T09:00:00Z[UTC]"];

        for invalid_case in invalid_strings {
            let err = PlainYearMonth::from_str(invalid_case);
            assert!(err.is_err());
        }
    }

    #[test]
    fn test_to_plain_date() {
        let year_month = PlainYearMonth::new_with_overflow(
            2023, // year
            5,    // month
            None, // reference_day
            Calendar::default(),
            ArithmeticOverflow::Reject,
        )
        .unwrap();

        let partial_date = PartialDate::new().with_day(Some(3));
        let plain_date = year_month.to_plain_date(Some(partial_date)).unwrap();
        assert_eq!(plain_date.iso_year(), 2023);
        assert_eq!(plain_date.iso_month(), 5);
        assert_eq!(plain_date.iso_day(), 3);
    }

    #[test]
    fn test_partial_year_month_try_from_plain() {
        let ym = PlainYearMonth::from_str("2024-05").unwrap();
        let partial = PartialYearMonth::try_from_year_month(&ym).unwrap();
        assert_eq!(partial.year, Some(2024));
        assert_eq!(partial.month, Some(5));
        assert_eq!(
            partial.month_code,
            Some(MonthCode::from_str("M05").unwrap())
        );
        assert_eq!(partial.era, None);
        assert_eq!(partial.era_year, None);
        assert_eq!(partial.calendar, ym.calendar().clone());
    }

    #[test]
    fn test_partial_year_month_date_round_trip() {
        let partial = PartialYearMonth::new()
            .with_year(Some(1999))
            .with_month(Some(12));
        let pd: PartialDate = (&partial).into();
        let reconstructed: PartialYearMonth = (&pd).into();
        assert_eq!(partial, reconstructed);
    }

    #[test]
    fn test_partial_year_month_builder_methods() {
        let calendar = Calendar::from_str("gregory").unwrap();
        let partial = PartialYearMonth::new()
            .with_year(Some(2020))
            .with_month(Some(7))
            .with_month_code(Some(MonthCode::from_str("M07").unwrap()))
            .with_era(Some(tinystr!(19, "ce")))
            .with_era_year(Some(2020))
            .with_calendar(calendar.clone());

        assert_eq!(partial.year, Some(2020));
        assert_eq!(partial.month, Some(7));
        assert_eq!(
            partial.month_code,
            Some(MonthCode::from_str("M07").unwrap())
        );
        assert_eq!(partial.era, Some(tinystr!(19, "ce")));
        assert_eq!(partial.era_year, Some(2020));
        assert_eq!(partial.calendar, calendar);
    }
}
