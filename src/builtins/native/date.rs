use crate::builtins::native::PlainTime;
use crate::{
    builtins::core,
    options::{ArithmeticOverflow, DifferenceSettings, DisplayCalendar},
    Calendar, TemporalResult,
};
use alloc::string::String;

use super::{duration::Duration, PartialDate, PlainDateTime, PlainMonthDay, PlainYearMonth};
use tinystr::TinyAsciiStr;

#[derive(Debug, Clone)]
pub struct PlainDate(pub(crate) core::PlainDate);

impl From<core::PlainDate> for PlainDate {
    fn from(value: core::PlainDate) -> Self {
        Self(value)
    }
}

impl PlainDate {
    /// Creates a new `PlainDate` automatically constraining any values that may be invalid.
    pub fn new(year: i32, month: u8, day: u8, calendar: Calendar) -> TemporalResult<Self> {
        core::PlainDate::new(year, month, day, calendar).map(Into::into)
    }

    /// Creates a new `PlainDate` rejecting any date that may be invalid.
    pub fn try_new(year: i32, month: u8, day: u8, calendar: Calendar) -> TemporalResult<Self> {
        core::PlainDate::try_new(year, month, day, calendar).map(Into::into)
    }

    /// Creates a new `PlainDate` with the specified overflow.
    ///
    /// This operation is the public facing API to Temporal's `RegulateIsoDate`
    #[inline]
    pub fn new_with_overflow(
        year: i32,
        month: u8,
        day: u8,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        core::PlainDate::new_with_overflow(year, month, day, calendar, overflow).map(Into::into)
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
    /// let date = PlainDate::from_partial(partial, None).unwrap();
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
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        core::PlainDate::from_partial(partial, overflow).map(Into::into)
    }

    /// Creates a date time with values from a `PartialDate`.
    pub fn with(
        &self,
        partial: PartialDate,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.0.with(partial, overflow).map(Into::into)
    }

    /// Creates a new `Date` from the current `Date` and the provided calendar.
    pub fn with_calendar(&self, calendar: Calendar) -> TemporalResult<Self> {
        self.0.with_calendar(calendar).map(Into::into)
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO year value.
    pub const fn iso_year(&self) -> i32 {
        self.0.iso_year()
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO month value.
    pub const fn iso_month(&self) -> u8 {
        self.0.iso_month()
    }

    #[inline]
    #[must_use]
    /// Returns this `Date`'s ISO day value.
    pub const fn iso_day(&self) -> u8 {
        self.0.iso_day()
    }

    #[inline]
    #[must_use]
    /// Returns a reference to this `Date`'s calendar slot.
    pub fn calendar(&self) -> &Calendar {
        self.0.calendar()
    }

    /// 3.5.7 `IsValidISODate`
    ///
    /// Checks if the current date is a valid `ISODate`.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.0.is_valid()
    }

    /// `DaysUntil`
    ///
    /// Calculates the epoch days between two `Date`s
    #[inline]
    #[must_use]
    pub fn days_until(&self, other: &Self) -> i32 {
        self.0.days_until(&other.0)
    }

    #[inline]
    /// Adds a `Duration` to the current `Date`
    pub fn add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.0.add(&duration.0, overflow).map(Into::into)
    }

    #[inline]
    /// Subtracts a `Duration` to the current `Date`
    pub fn subtract(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.0.subtract(&duration.0, overflow).map(Into::into)
    }

    #[inline]
    /// Returns a `Duration` representing the time from this `Date` until the other `Date`.
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.until(&other.0, settings).map(Into::into)
    }

    #[inline]
    /// Returns a `Duration` representing the time passed from this `Date` since the other `Date`.
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.since(&other.0, settings).map(Into::into)
    }
}

// ==== Calendar-derived Public API ====

impl PlainDate {
    /// Returns the calendar year value.
    pub fn year(&self) -> TemporalResult<i32> {
        self.0.year()
    }

    /// Returns the calendar month value.
    pub fn month(&self) -> TemporalResult<u8> {
        self.0.month()
    }

    /// Returns the calendar month code value.
    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.0.month_code()
    }

    /// Returns the calendar day value.
    pub fn day(&self) -> TemporalResult<u8> {
        self.0.day()
    }

    /// Returns the calendar day of week value.
    pub fn day_of_week(&self) -> TemporalResult<u16> {
        self.0.day_of_week()
    }

    /// Returns the calendar day of year value.
    pub fn day_of_year(&self) -> TemporalResult<u16> {
        self.0.day_of_year()
    }

    /// Returns the calendar week of year value.
    pub fn week_of_year(&self) -> TemporalResult<Option<u16>> {
        self.0.week_of_year()
    }

    /// Returns the calendar year of week value.
    pub fn year_of_week(&self) -> TemporalResult<Option<i32>> {
        self.0.year_of_week()
    }

    /// Returns the calendar days in week value.
    pub fn days_in_week(&self) -> TemporalResult<u16> {
        self.0.days_in_week()
    }

    /// Returns the calendar days in month value.
    pub fn days_in_month(&self) -> TemporalResult<u16> {
        self.0.days_in_month()
    }

    /// Returns the calendar days in year value.
    pub fn days_in_year(&self) -> TemporalResult<u16> {
        self.0.days_in_year()
    }

    /// Returns the calendar months in year value.
    pub fn months_in_year(&self) -> TemporalResult<u16> {
        self.0.months_in_year()
    }

    /// Returns returns whether the date in a leap year for the given calendar.
    pub fn in_leap_year(&self) -> TemporalResult<bool> {
        self.0.in_leap_year()
    }

    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        self.0.era()
    }

    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        self.0.era_year()
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
        self.0.to_date_time(time.map(|t| t.0)).map(Into::into)
    }

    /// Converts the current `Date<C>` into a `PlainYearMonth`
    #[inline]
    pub fn to_year_month(&self) -> TemporalResult<PlainYearMonth> {
        self.0.to_year_month()
    }

    /// Converts the current `Date<C>` into a `PlainMonthDay`
    #[inline]
    pub fn to_month_day(&self) -> TemporalResult<PlainMonthDay> {
        self.0.to_month_day()
    }

    #[inline]
    pub fn to_ixdtf_string(&self, display_calendar: DisplayCalendar) -> String {
        self.0.to_ixdtf_string(display_calendar)
    }
}
