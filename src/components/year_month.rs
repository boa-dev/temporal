//! This module implements `YearMonth` and any directly related algorithms.

use std::str::FromStr;

use tinystr::TinyAsciiStr;

use crate::{
    components::calendar::Calendar,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    utils::pad_iso_year,
    TemporalError, TemporalResult, TemporalUnwrap,
};

use super::{
    calendar::{CalendarDateLike, GetTemporalCalendar},
    calendar_types::CalendarFields,
    Duration,
};

// Subset of `TemporalFields` representing just the  `YearMonthFields`
pub struct YearMonthFields(pub i32, pub u8);

/// The native Rust implementation of `Temporal.YearMonth`.
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
pub struct YearMonth {
    iso: IsoDate,
    calendar: Calendar,
}

impl YearMonth {
    /// Creates an unvalidated `YearMonth`.
    #[inline]
    #[must_use]
    pub(crate) fn new_unchecked(iso: IsoDate, calendar: Calendar) -> Self {
        Self { iso, calendar }
    }

    /// Creates a new valid `YearMonth`.
    #[inline]
    pub fn new(
        year: i32,
        month: i32,
        reference_day: Option<i32>,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let day = reference_day.unwrap_or(1);
        let iso = IsoDate::new(year, month, day, overflow)?;
        Ok(Self::new_unchecked(iso, calendar))
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

    pub fn year(&self) -> TemporalResult<i32> {
        self.calendar().year(&CalendarDateLike::YearMonth(self))
    }

    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar().month(&CalendarDateLike::YearMonth(self))
    }

    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.get_calendar()
            .month_code(&CalendarDateLike::YearMonth(self))
    }

    #[inline]
    #[must_use]
    pub fn in_leap_year(&self) -> bool {
        self.get_calendar()
            .in_leap_year(&CalendarDateLike::YearMonth(self))
            .is_ok_and(|is_leap_year| is_leap_year)
    }

    pub fn get_days_in_year(&self) -> TemporalResult<u16> {
        self.get_calendar()
            .days_in_year(&CalendarDateLike::YearMonth(self))
    }

    pub fn get_days_in_month(&self) -> TemporalResult<u16> {
        self.get_calendar()
            .days_in_month(&CalendarDateLike::YearMonth(self))
    }

    pub fn get_months_in_year(&self) -> TemporalResult<u16> {
        self.get_calendar()
            .months_in_year(&CalendarDateLike::YearMonth(self))
    }

    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        self.calendar().era(&CalendarDateLike::YearMonth(self))
    }

    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        self.calendar().era_year(&CalendarDateLike::YearMonth(self))
    }
}

impl YearMonth {
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

    pub fn add_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<YearMonth> {
        self.add_or_subtract_duration(duration, overflow)
    }

    pub fn subtract_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<YearMonth> {
        self.add_or_subtract_duration(&duration.negated(), overflow)
    }

    pub(crate) fn add_or_subtract_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<YearMonth> {
        let fields = CalendarFields::try_from_year_month(self.calendar(), self)?;

        let mut intermediate_date = self.get_calendar().date_from_fields(&fields, overflow)?;

        intermediate_date = intermediate_date.add_date(duration, Some(overflow))?;

        let mut result_fields =
            YearMonthFields(intermediate_date.iso_year(), intermediate_date.iso_month()).into();

        self.get_calendar()
            .year_month_from_fields(&mut result_fields, overflow)
    }
}

impl GetTemporalCalendar for YearMonth {
    /// Returns a reference to `YearMonth`'s `CalendarSlot`
    fn get_calendar(&self) -> Calendar {
        self.calendar.clone()
    }
}

impl IsoDateSlots for YearMonth {
    #[inline]
    /// Returns this `YearMonth`'s `IsoDate`
    fn iso_date(&self) -> IsoDate {
        self.iso
    }
}

impl FromStr for YearMonth {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let record = crate::parsers::parse_year_month(s)?;

        let calendar = record.calendar.unwrap_or("iso8601");

        let date = record.date.temporal_unwrap()?;

        Self::new(
            date.year,
            date.month.into(),
            None,
            Calendar::from_str(calendar)?,
            ArithmeticOverflow::Reject,
        )
    }
}
