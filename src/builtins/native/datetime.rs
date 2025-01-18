use crate::{
    builtins::core,
    options::{
        ArithmeticOverflow, DifferenceSettings, DisplayCalendar, RoundingOptions,
        ToStringRoundingOptions,
    },
    Calendar, TemporalResult,
};
use alloc::string::String;
use tinystr::TinyAsciiStr;

use super::{Duration, PartialDateTime, PlainDate, PlainTime};
pub struct PlainDateTime(pub(crate) core::PlainDateTime);

impl From<core::PlainDateTime> for PlainDateTime {
    fn from(value: core::PlainDateTime) -> Self {
        Self(value)
    }
}

impl PlainDateTime {
    /// Creates a new `DateTime`, constraining any arguments that into a valid range.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
        calendar: Calendar,
    ) -> TemporalResult<Self> {
        core::PlainDateTime::new(
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
        )
        .map(Into::into)
    }

    /// Creates a new `DateTime`, rejecting any arguments that are not in a valid range.
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        year: i32,
        month: u8,
        day: u8,
        hour: u8,
        minute: u8,
        second: u8,
        millisecond: u16,
        microsecond: u16,
        nanosecond: u16,
        calendar: Calendar,
    ) -> TemporalResult<Self> {
        core::PlainDateTime::try_new(
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
        )
        .map(Into::into)
    }

    /// Create a `DateTime` from a `Date` and a `Time`.
    pub fn from_date_and_time(date: PlainDate, time: PlainTime) -> TemporalResult<Self> {
        core::PlainDateTime::from_date_and_time(date.0, time.0).map(Into::into)
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
    /// let date = PlainDateTime::from_partial(partial, None).unwrap();
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
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        core::PlainDateTime::from_partial(partial, overflow).map(Into::into)
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
        self.0.with(partial_datetime, overflow).map(Into::into)
    }

    /// Creates a new `DateTime` from the current `DateTime` and the provided `Time`.
    pub fn with_time(&self, time: PlainTime) -> TemporalResult<Self> {
        self.0.with_time(time.0).map(Into::into)
    }

    /// Creates a new `DateTime` from the current `DateTime` and a provided `Calendar`.
    pub fn with_calendar(&self, calendar: Calendar) -> TemporalResult<Self> {
        self.0.with_calendar(calendar).map(Into::into)
    }

    /// Returns this `Date`'s ISO year value.
    #[inline]
    #[must_use]
    pub const fn iso_year(&self) -> i32 {
        self.0.iso_year()
    }

    /// Returns this `Date`'s ISO month value.
    #[inline]
    #[must_use]
    pub const fn iso_month(&self) -> u8 {
        self.0.iso_month()
    }

    /// Returns this `Date`'s ISO day value.
    #[inline]
    #[must_use]
    pub const fn iso_day(&self) -> u8 {
        self.0.iso_day()
    }

    /// Returns the hour value
    #[inline]
    #[must_use]
    pub fn hour(&self) -> u8 {
        self.0.hour()
    }

    /// Returns the minute value
    #[inline]
    #[must_use]
    pub fn minute(&self) -> u8 {
        self.0.minute()
    }

    /// Returns the second value
    #[inline]
    #[must_use]
    pub fn second(&self) -> u8 {
        self.0.second()
    }

    /// Returns the `millisecond` value
    #[inline]
    #[must_use]
    pub fn millisecond(&self) -> u16 {
        self.0.millisecond()
    }

    /// Returns the `microsecond` value
    #[inline]
    #[must_use]
    pub fn microsecond(&self) -> u16 {
        self.0.microsecond()
    }

    /// Returns the `nanosecond` value
    #[inline]
    #[must_use]
    pub fn nanosecond(&self) -> u16 {
        self.0.nanosecond()
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        self.0.calendar()
    }
}

// ==== Calendar-derived public API ====

impl PlainDateTime {
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

impl PlainDateTime {
    #[inline]
    /// Adds a `Duration` to the current `DateTime`.
    pub fn add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.0.add(&duration.0, overflow).map(Into::into)
    }

    #[inline]
    /// Subtracts a `Duration` to the current `DateTime`.
    pub fn subtract(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.0.subtract(&duration.0, overflow).map(Into::into)
    }

    #[inline]
    /// Returns a `Duration` representing the period of time from this `DateTime` until the other `DateTime`.
    pub fn until(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.until(&other.0, settings).map(Into::into)
    }

    #[inline]
    /// Returns a `Duration` representing the period of time from this `DateTime` since the other `DateTime`.
    pub fn since(&self, other: &Self, settings: DifferenceSettings) -> TemporalResult<Duration> {
        self.0.since(&other.0, settings).map(Into::into)
    }

    /// Rounds the current datetime based on provided options.
    pub fn round(&self, options: RoundingOptions) -> TemporalResult<Self> {
        self.0.round(options).map(Into::into)
    }

    pub fn to_ixdtf_string(
        &self,
        options: ToStringRoundingOptions,
        display_calendar: DisplayCalendar,
    ) -> TemporalResult<String> {
        self.0.to_ixdtf_string(options, display_calendar)
    }
}
