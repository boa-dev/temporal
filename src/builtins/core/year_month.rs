//! This module implements `YearMonth` and any directly related algorithms.

use alloc::string::String;
use core::str::FromStr;

use tinystr::TinyAsciiStr;

use crate::{
    iso::IsoDate,
    options::{ArithmeticOverflow, DisplayCalendar},
    parsers::{FormattableCalendar, FormattableDate, FormattableYearMonth},
    utils::pad_iso_year,
    Calendar, TemporalError, TemporalResult, TemporalUnwrap,
};

use super::{Duration, PartialDate};

/// The native Rust implementation of `Temporal.YearMonth`.
#[non_exhaustive]
#[derive(Debug, Default, Clone)]
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

    /// Creates a new valid `YearMonth`.
    #[inline]
    pub fn new_with_overflow(
        year: i32,
        month: u8,
        reference_day: Option<u8>,
        calendar: Calendar,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let day = reference_day.unwrap_or(1);
        let iso = IsoDate::new_with_overflow(year, month, day, overflow)?;
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
        self.calendar().year(&self.iso)
    }

    pub fn month(&self) -> TemporalResult<u8> {
        self.calendar().month(&self.iso)
    }

    pub fn month_code(&self) -> TemporalResult<TinyAsciiStr<4>> {
        self.calendar().month_code(&self.iso)
    }

    #[inline]
    #[must_use]
    pub fn in_leap_year(&self) -> bool {
        self.calendar()
            .in_leap_year(&self.iso)
            .is_ok_and(|is_leap_year| is_leap_year)
    }

    pub fn get_days_in_year(&self) -> TemporalResult<u16> {
        self.calendar().days_in_year(&self.iso)
    }

    pub fn get_days_in_month(&self) -> TemporalResult<u16> {
        self.calendar().days_in_month(&self.iso)
    }

    pub fn get_months_in_year(&self) -> TemporalResult<u16> {
        self.calendar().months_in_year(&self.iso)
    }

    pub fn era(&self) -> TemporalResult<Option<TinyAsciiStr<16>>> {
        self.calendar().era(&self.iso)
    }

    pub fn era_year(&self) -> TemporalResult<Option<i32>> {
        self.calendar().era_year(&self.iso)
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

    pub fn add_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        self.add_or_subtract_duration(duration, overflow)
    }

    pub fn subtract_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        self.add_or_subtract_duration(&duration.negated(), overflow)
    }

    pub(crate) fn add_or_subtract_duration(
        &self,
        duration: &Duration,
        overflow: ArithmeticOverflow,
    ) -> TemporalResult<Self> {
        let partial = PartialDate::try_from_year_month(self)?;

        let mut intermediate_date = self.calendar().date_from_partial(&partial, overflow)?;

        intermediate_date = intermediate_date.add_date(duration, Some(overflow))?;

        let result_fields = PartialDate::default().with_fallback_date(&intermediate_date)?;

        self.calendar()
            .year_month_from_partial(&result_fields, overflow)
    }

    pub fn to_ixdtf_string(&self, display_calendar: DisplayCalendar) -> String {
        let ixdtf = FormattableYearMonth {
            date: FormattableDate(self.iso_year(), self.iso_month(), self.iso.day),
            calendar: FormattableCalendar {
                show: display_calendar,
                calendar: self.calendar().identifier(),
            },
        };
        ixdtf.to_string()
    }
}

impl FromStr for PlainYearMonth {
    type Err = TemporalError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let record = crate::parsers::parse_year_month(s)?;

        let calendar = record
            .calendar
            .map(Calendar::from_utf8)
            .transpose()?
            .unwrap_or_default();

        let date = record.date.temporal_unwrap()?;

        Self::new_with_overflow(
            date.year,
            date.month,
            None,
            calendar,
            ArithmeticOverflow::Reject,
        )
    }
}
