//! This module implements `YearMonth` and any directly related algorithms.

use std::str::FromStr;

use tinystr::TinyAsciiStr;

use crate::{
    components::calendar::Calendar,
    iso::{IsoDate, IsoDateSlots},
    options::ArithmeticOverflow,
    utils, TemporalError, TemporalResult, TemporalUnwrap,
};

use super::calendar::GetTemporalCalendar;

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

    /// Returns the `year` value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn year(&self) -> i32 {
        self.iso.year
    }

    /// Returns the `month` value for this `YearMonth`.
    #[inline]
    #[must_use]
    pub fn month(&self) -> u8 {
        self.iso.month
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn in_leap_year(&self) -> bool {
        utils::mathematical_in_leap_year(utils::epoch_time_for_year(self.iso.year)) == 1
    }

    /// Returns the calendar month code value with provided context.
    pub fn contextual_month_code(
        this: &C::YearMonth,
        context: &mut C::Context,
    ) -> TemporalResult<TinyAsciiStr<4>> {
        this.get_calendar()
            .month_code(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    /// Returns the Calendar value.
    #[inline]
    #[must_use]
    pub fn calendar(&self) -> &Calendar {
        &self.calendar
    }

    pub fn contextual_get_months_in_year(
        this: &C::YearMonth,
        context: &mut C::Context,
    ) -> TemporalResult<u16> {
        this.get_calendar()
            .months_in_year(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    #[inline]
    pub fn contextual_add(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        self.add_or_subtract_duration(duration, overflow)
    }

    pub fn get_days_in_year(this: &C::YearMonth, context: &mut C::Context) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_year(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    pub fn get_days_in_month(this: &C::YearMonth, context: &mut C::Context) -> TemporalResult<u16> {
        this.get_calendar()
            .days_in_month(&CalendarDateLike::YearMonth(this.clone()), context)
    }

    fn add_or_subtract_duration(
        &self,
        duration: &Duration,
        overflow: Option<ArithmeticOverflow>,
    ) -> TemporalResult<Self> {
        let new_date = self.iso.add_date_duration(
            duration.date(),
            overflow.unwrap_or(ArithmeticOverflow::Constrain),
        )?;

        Ok(Self::new_unchecked(new_date, self.calendar.clone()))
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
